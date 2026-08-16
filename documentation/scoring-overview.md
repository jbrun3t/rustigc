# Scoring documentation

A description of the scoring algorithm, aimed at developers: how it works, what the challenges are,
what each step costs, and why the optimizations lowering those costs are shaped the way they are.

For per-module detail, follow the pointers in the last section.

---

## Problem difficulty

Scoring a flight means finding, among every way turnpoints could be placed on the track log, the one
that scores the most points under a league's rules.

Trying every combination of turnpoints is not practically possible. A flight log can hold tens of
thousands of GPS fixes, and a shape with `k` turnpoints picked from `n` fixes is `C(n, k)`
combinations before a single distance is measured. Ondřej Palkovský's paper on the problem
([*Paragliding Competition Tracklog Optimization*](https://web.archive.org/web/20230320111732/http://www.penguin.cz/~ondrap/algorithm.pdf))
puts the general case's worst-case complexity at `O(n⁵)`.

Each individual distance is expensive too. The Earth's surface is a curved, slightly flattened
ellipsoid, not a plane: even the cheap approximation used during the search (`Fcc::distance`) needs
a cosine of latitude and a square root per leg, and the exact ellipsoid distance has no closed-form
solution — it is solved iteratively, which is why it is only ever run once, on the winning leaf, and
never inside the search.

Brute force is therefore not viable. Most combinations of turnpoints are obviously bad long before
their distance is measured precisely — the search below is built to act on that, instead of measuring
everything.

## Branch and bound

Rather than enumerate fixes, the search works with **ranges** of fixes — one range per turnpoint,
initially the whole track. A `Candidate` holds one `[start, end]` range per turnpoint; the root
candidate spans the whole track for every one of them. The search narrows those ranges down until each
is a single fix — a **leaf**, a fully-specified flight.

Each candidate is asked two questions:

* **bound** — the best score any flight *inside these ranges* could reach, computed from the ranges'
  bounding-box vertices (see *Bound computation*, below) rather than the real track. It is an
  over-estimate: it may credit a distance no real fix combination inside the ranges can actually fly.

  Getting the direction of that estimate right is what makes the search correct: `bound` must never
  fall *below* what its own subtree can really reach, or the search may silently drop the branch
  holding the true optimum. Over-estimating stays correct — it only costs convergence speed — but
  that cost may be critical.

* **peek** — a real, achievable score, through one actual fix of each range. Ranges are kept
  chronologically ordered, so this path is always flyable — but it is not necessarily close to the
  best score the ranges could still yield. Unlike `bound`, `peek` feeds the floor used to prune
  candidates, so a weak `peek` costs pruning strength, not correctness.

All live candidates sit in one priority queue (a max-heap), ordered by `bound`. The search proceeds
with the following steps:

1. Pop the candidate with the highest bound.
2. If it's a leaf, it's a fully-specified flight — and since the heap is ordered by bound and a leaf's
   bound equals its real score, this is the global optimum. Stop.
3. Otherwise, split its widest range in two, producing two children with narrower ranges.
4. Score both children (bound and peek) and push them back onto the heap.
5. Keep a running **floor**: the best real score (`peek`) seen so far. Any candidate whose `bound`
   cannot beat the floor is dropped — it cannot contain the answer.

## Bound computation

`bound` turns a candidate's ranges into a small dynamic program over bounding-box vertices, not the
real track. This is the same idea as the vertex-path search in Palkovský's paper: route the flight
through one vertex of each turnpoint's box, and take whichever routing goes furthest. The length of
that routing is the bound.

Each turnpoint range becomes a bounding box. Between every pair of consecutive turnpoint boxes, a
`TransitionMatrix` holds the leg from every vertex of one box to every vertex of the next: at most
4×4 numbers.

The dynamic programming (DP) search then walks the boxes left to right, keeping one running best per
vertex of the *current* box only — at most 4 numbers, not the whole track. Reaching a vertex of the
next box costs whatever it took to reach some vertex of this one, plus the leg between them; each
vertex of the next box keeps the best of those, over every vertex it could have come from. The first
box starts from the real entry leg into each of its vertices, and the last one folds in the real exit
leg the same way — the largest total across its vertices is the `bound`.

`entry` and `exit` are the real entry and exit leg lengths for an open shape, from the furthest-point
caches described in *Runtime costs*, below.

Circuits close on themselves, so the closing leg's length depends on *which* vertex of TP1 the loop
actually started from — what a single left-to-right pass throws away. The circuit DP therefore runs
one DP per vertex of TP1, using the first and last `TransitionMatrix` as that run's entry and exit
costs, and keeps the best of those runs.

## Scoring every rule in one pass

A league is a set of rules (e.g. XContest's free distance and its two triangle variants). Rather than
run the search once per rule, every rule of a league shares one heap and one floor: a strong candidate
for one rule raises the floor for all of them, so weak rules get pruned early by a floor that candidates
of other rules set. `Scorer::solve` runs exactly one pass over the whole league — scoring the whole
ruleset costs close to what scoring its single best rule alone would; for XContest, scoring each rule
separately and comparing the results afterwards would run around 3× longer.

## Shape vs. rule

* A **shape** is the geometry — how many turnpoints, and whether the flight must close back on
  itself. `OpenPolyline<N>` (a free-distance flight of N points, ends included), `ClosedCircuit<N>`,
  `BalancedCircuit<N, MINSIDE>` (the FAI triangle, with its 28%-per-side rule). A shape only knows
  about ranges and distances — bound, peek, where the flight starts/ends, the closing gap if any.
* A **rule** is what a league pays for that geometry — the multiplier, the penalty charged for a
  closing gap, the minimum score, the name shown in the report. A rule never sees a fix index, only
  the distance and gap a shape hands it.

This split is why adding a scoring rule to an existing shape (a new multiplier scheme, say) does not
touch the search, and keeps the process of adding one simple — and why the same `ClosedCircuit`
geometry backs triangle rules for several leagues.

See [`add-a-league.md`](add-a-league.md) for a walkthrough of adding a new league or rule.

## Runtime costs

Every `TransitionMatrix` is a distance calculation per pair of box vertices, and every candidate the
search ever creates — survivors and dead ends alike — computes them. It stays cheap because it is
small and fixed-size, at most 4×4 legs, however wide the ranges are.

What actually dominates the runtime is two geometric questions whose cost grows with how much of the
track they have to look at:

* **Open shapes** need to know how far the entry leg (before the first turnpoint) and the exit leg
  (after the last) reach — a furthest-point search from each turnpoint's box.
* **Circuits** need to know how close the flight comes back to where it started — a nearest-pair
  search between the start and end of the window.

Because those operations are expensive, both are cached — the search revisits the same ranges from
many sibling candidates — and both are documented on their own:

* [`furthest-cache.md`](furthest-cache.md) — the open-distance side.
* [`closing-search.md`](closing-search.md) — the circuit side.

Bounding boxes over ranges are cached too, for the same reason: `eval` asks for the same box repeatedly
across sibling candidates.

Note that runtime is driven far more by how tight `bound` is than by how much the searches above
cost, or by how many fixes the log holds. A bound stays tight when real distances discriminate
between candidates — one range is clearly better than another, and the weaker one gets pruned. It
goes slack on flights that are close to degenerate: a track close to a straight line puts many
different turnpoint placements at nearly the same distance from each other, so their bounds tie and
the search cannot rule any of them out without splitting almost down to the leaf; a track close to a
perfect circle does the same to a circuit, since many different closings come back to nearly the
same gap. Neither case is about the number of fixes — the geometry itself gives the search nothing
to prune on.

| fixture | fai-02 | free-04 |
|---|---|---|
| Number of fixes | 39725 | 3886 |
| Rule | FAI Triangle | Free Distance |
| DP search per eval | 8 | 1 |
| Total evaluations | 411 | 1188825 |
| Heap maximum size | 148 | 176706 |
| Search time, normalized | 1 | 41 |

The table above is a rather extreme example. free-04 has ten times fewer fixes than fai-02 and, in
theory, runs a much simpler search, so it should be much faster. But the tracklog shape of free-04
does not let the search discriminate as well as it can on fai-02, and the search ends up 41× longer.

Even so, the B&B is doing its job: free-04's 1.2 million evaluations are 0.002 % of the 3886³
turnpoint triples a brute force would have to walk — and that is the *cheap* count, since the rule
scored here is a five-point polyline, whose real brute force is C(3886, 5).

## Keeping the search sound

The search itself runs on the cheap `Fcc` distance approximation, not the `Geodesic` one. The error
between the two grows monotonically with distance, so `Fcc` ranks candidates just as well and is fine
for exploring the tracklog. It is judging a *limit* that it cannot be trusted with — a closing ratio
or an FAI minimum side that `Fcc` passes, `Geodesic` may not. To keep the floor from resting on a
distance the exact re-measure would not confirm, every distance the search scores has a margin
subtracted, sized to the flight's latitude and to the distance itself.

This does not eliminate disagreement, only bounds it: if the exact re-measure on the winning leaf still
fails the shape's own constraints, that is not an error — the search takes the next-best leaf off the
heap instead, since the floor was never raised above what a re-measure can withdraw.

## Final Score

Once a leaf wins, the solution is recomputed once against the WGS84 reference: the exact geodesic distance
of every leg, which fixes are turnpoints vs. entry/exit, the closing gap. The rule then turns that
raw `(distance, gap)` into what gets reported — applies the closing penalty, the league's
multiplier, checks the minimum, and rounds.

## More details

* [`furthest-cache.md`](furthest-cache.md), [`closing-search.md`](closing-search.md) — the two caches
  behind the costly searches above.
* [`igc-xc-score-diff.md`](igc-xc-score-diff.md) — where this implementation's results differ from the
  igc-xc-score JS tool, and why.
* [`add-a-league.md`](add-a-league.md) — how to add a new league or rule.
