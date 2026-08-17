# Comparison with igc-xc-score

`rustigc`'s scoring started as Rust re-implementation of [igc-xc-score](https://github.com/mmomtchev/igc-xc-score)
ideas. It would not exist without it, and the general design idea is entirely igc-xc-score's.

This document records where the two differ and why, for anyone comparing them. Reference: igc-xc-score
v1.8.0.

## Shared design

- Palkovský branch-and-bound over one fix-index range per turnpoint, split at the midpoint, bounded by
  the longest path over the ranges' bounding-box vertices.
- Left-first ordering, so permutations collapse into combinations: each range is trimmed against its
  neighbours, making any pick chronological.
- All rules of a league run in one shared pass — one queue, one incumbent, shared caches.
- FCC distance during the search (igc-xc-score can be configured otherwise).
- The closing search is the closest pair between the entry-side and exit-side ranges, found via a spatial
  index built over one side and queried with the other. The details differ — see below.
- Neither is multi-threaded (yet).

## Turnpoint count

igc-xc-score is built around a fixed cardinality 3. In `rustigc` the count is a property of the
geometry (`ShapeKind::CARDINALITY`), so rules may have any number of turnpoints.
The limit is computational power, memory and runtime required to solve the problem.

## Termination and incremental solving

igc-xc-score pops the greatest bound from a sorted set and stops once `current.bound <= best.score`.
Its solver is a generator: `maxcycle`/`maxloop` yield a best-so-far result carrying
an `optimal` flag, so a caller can time-box the search and resume it.

`rustigc` breaks on the first leaf popped, valid because the heap is ordered on bounds and a leaf's bound
is its score. It has no incremental mode yet.

## FAI cylinders

`igc-xc-score` offers FAI cylinder adjustment (400 m, `adjustFAICylinders`). `rustigc` does not.

## Chain maximisation: enumeration vs dynamic program

igc-xc-score enumerates vertex combinations directly — `maxDistance3Rectangles` is a triple loop,
`maxDistancePath` a recursion over the filtered vertex sets. `rustigc` runs one or more dynamic
programs over the layers.

## Entry and exit of an open distance

igc-xc-score's `boundDistance3Points` bounds the entry leg with one point:

```js
const pin = geom.findFurthestPointInSegment(opt.launch, ranges[0].start, boxes[0], opt);
geom.maxDistanceNRectangles([pin, boxes[0], boxes[1], boxes[2], pout])
```

`findFurthestPointInSegment` computes the furthest prefix fix for each of the four vertices of
`boxes[0]`, then returns only the fix belonging to the globally best vertex; the other three values
are discarded. The resulting bound can come out below what its own subtree reaches, which puts
optimality at risk.

`rustigc` keeps all four. `Furthest::terminals` returns one cost per turnpoint-box vertex —
`max(furthest, box diagonal)` — and the DP folds them into its first and last layer. Same number of
cache queries, without the collapse to a single vertex.

## Vertex reduction heuristic

`maxDistanceNRectangles` and `maxDistance3Rectangles` keep, per box, only the vertices lying on an
extreme of the global bounding box. igc-xc-score's own comment presents this as an addition to
Palkovský's method for three rectangles. However, the open-distance path applies it to a
five-element chain, not three. The path to maximize starts at the entry, not the first turnpoint, and it
may go through one of the eliminated vertices. This also puts optimality at risk and compounds the
problem presented above.

## Furthest-point search: flat projection instead of FCC per fix

For an open flight the bound needs the furthest track point from each vertex of the first and last
turnpoint box — up to 8 scans, each over a large slice of the track. igc-xc-score computes a full
`distanceEarth` per fix, cached where it can be reused.

`rustigc` keeps a similar cache but scans on a flat projection locally compensated for Earth curvature
(`CheapProjection`), ranking by plain Euclidean distance and computing FCC only for the winner.
igc-xc-score already uses this idea for its closing search, scaling longitude by `cos(latitude)` before
building the Flatbush index; `rustigc` applies it to both searches, without the cosine computation.

The flat ranking picks the same fix as geodesic in the large majority of cases. When it does not,
the two points are equally valid as far as the search is concerned — the geodesic distance difference
is insignificant.

## Closing search

Both find the closest pair between the two ends by indexing one side and querying with the other, and
both cache the answer the same way (above). Everything inside that differs.

| | igc-xc-score (`geom.js`) | `rustigc` (`score/cache/closing.rs`) |
|---|---|---|
| index | packed Hilbert R-tree (`Flatbush`) | `rstar::RTree` |
| indexed side | Entry side, `[launch..p1]` | the **shorter** of the two |
| query | `rtree.neighbors(x, y, 1)` — a plain nearest neighbour, restarting from scratch for every exit fix | `locate_within_distance` with **radius = the best pair found so far**, so a fix that cannot improve is refuted at the root |
| distance per iterated fix | full `distanceEarth` (FCC) **plus the rule's rounding**, once per fix | none — ranked on squared distance in the projected plane; one `Fcc::distance` for the winner only |
| projection | longitude only, scaled by `cos(lat at p1)` | `CheapProjection` about the merged bounding box: separate `kx`/`ky`, both scaled |

The query line is important. Asking for a plain nearest neighbour means the traversal must
keep every branch that might hold something closer than *its own* best, and here the query point sits
outside the cloud it searches, so almost nothing prunes. Carrying the caller's running best in as a
radius refutes the vast majority of queries immediately. On a circuit the closing search is the bulk
of scoring runtime.

The side choice follows from what the structure costs: bulk-loading an R-tree sorts, so with the radius
in place building dominates querying and the smaller side should be the one indexed. igc-xc-score's
fixed choice of the entry side is not equivalent — the two sides can differ by an order of magnitude in
length.

**One thing igc-xc-score does that `rustigc` does not:** it bounds the exit-side scan with `lastUnknown`,
taken from an already-cached solution, so it stops iterating where a previous search already has an
answer (`geom.js:205-211`). `rustigc` always scans the full exit range and relies on the radius to make
each query cheap instead.

## Reported metric

igc-xc-score computes FCC everywhere by default, including the reported result; a `hp` option swaps
`Point.prototype.distanceEarth` for Vincenty. `rustigc` uses FCC inside the search and always reports
geodesic.

When a solution is on the limit with FCC distance (closing distance or FAI ratio), it may be invalidated
when recomputed with geodesic distance. `rustigc` accounts for this and continues the search for a valid
solution.

This is one reason why `igc-xc-score` sometimes finds a better optimum than `rustigc`: it may report
solutions that, once recomputed over WGS84, turn out to be invalid, by an extremely small margin.

## Rounding

Both `igc-xc-score` and `rustigc` round to nearest.
`igc-xc-score` rounds each leg to 0.01 km before summing, then applies a final rounding — `round2` for
FFVL, `round1` for the FAI/XContest configs — a deliberate choice explained in its documentation. It
does mean a rounded total can exceed the real one, so a constraint tested against it may pass on a
triangle that the unrounded measure rejects (see the minimum side test below).

`rustigc` keeps unrounded metres throughout and rounds only for presentation, so per-leg rounding
slightly inflates igc-xc-score's total on long flights.

### Minimum side test

`scoreTriangle` and `scoreOpenTriangle` compare `minSide` against the `round2`-ed legs and the rounded
total (`src/scoring.js:156-159`, `240-243`). `rustigc` tests the ratio on unrounded metres, in the search
(`BalancedCircuit::peek`) and again geodesically before reporting (`Circuit::admissible`).

The two therefore admit different sets of triangles. On `fai-03` FFVL over the same window, each side's
own turnpoints re-measured with WGS84:

| | turnpoints | total, FCC | total, geodesic | min leg / total, geodesic |
|---|---|---|---|---|
| `rustigc` | 372, 7852, 10462 | 66122.610 | 66122.405 | 28.0004 % |
| igc-xc-score | 370, 7856, 10460 | 66138.756 | 66138.550 | 27.9966 % |

igc-xc-score's triangle is 16.1 m longer under both metrics. Its shortest leg is 18516.5 m against a
required 18518.8 m unrounded; rounded to km per leg it reads `29.10 + 18.52 + 18.52 = 66.14` against
18.5192 km, so it passes by 0.8 m. Running the same test with `hp=true` yields the same turnpoints, so
the rounding introduces this problem.

`fai-05` FFVL behaves the same way: `rustigc` reports 102630.069 m and 28.0009 %;
igc-xc-score reports 102635.877 m and 27.9979 %, and with `hp=true` it reports 102635.165 m and
27.9984 %.

## Bounding-box caching

`new Box(this.ranges[r], opt.flight)` runs in every `Solution` constructor and scans the whole range, so
every node rebuilds all its boxes — including the two of three a split leaves unchanged. `rustigc` caches
boxes by `(start, end)`. The gain is small, but the fix is simple.

