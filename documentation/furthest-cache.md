# The furthest-point cache

This describes how `scoring/cache/furthest.rs` works, and why it is shaped the way it is. An **open
distance** has to know how far out its two ends reach — the leg before the first turnpoint and the
leg after the last.

---

## TL;DR

* The primitive: find the furthest fix from an anchor point, over a range running from a turnpoint to
  one end of the window.
* A cached entry records three things: **the fix found**, **the distance**, and **how far the scan
  looked**. The third is what lets a later, wider request resume instead of restarting.
* A resumed scan only sees a *segment*, so its result is not an answer. `insert` completes it by
  **adopting the previous winner** when the new segment turned up nothing better.
* Three caches in two layers. Layer 1 is keyed on the anchor point's raw bits; the two layer-2 caches
  above it are keyed on what their own caller actually asked:
  * `terminals` — one entry per **range**, bundling a whole box's vertices into one cached vector.
    Its anchors are box corners, not track fixes.
  * `end` — one slot per **turnpoint index**. Its anchor *is* a track fix, so `tp` alone identifies
    the query and no hashing is needed at all.

---

## 1. Furthest distance

An open-distance flight is a polyline: a start point, some turnpoints, an end point.

```
   Leg::Entry: argmax over [0 ..= tp]              "how far back does the entry leg reach?"
   Leg::Exit:  argmax over [tp ..= last]           "how far on does the exit leg reach?"

   0 ─────────────────► TP1 ... TPn ─────────────────────► last
   └── Entry searches ──┘         └─── Exit searches ──────┘
```

Both ranges are **anchored at a window end**.

### The anchors are not fixes

This matters. `OpenPolyline::bound` works on *bounding boxes* of index ranges, and it needs the reach
from **each corner of the box** — synthesised lon/lat points that do not exist in the track. (This is
true of `terminals`; `end`'s anchor is a real fix — see §2.)

`BBox::vertices` returns **one, two or four** points. Each vertex is keyed and cached **independently**
at this layer. §2 covers the layer above, which bundles them.

## 2. Users

Only `OpenPolyline` uses this, and only when `TERMINALS` is true (`POINTS > 3`, so `<4>` and `<5>` in
the current rule set, plus `<6>` behind the `crazy-test` feature). `OpenPolyline<3>` and `<2>` search
their end legs as two extra branch-and-bound dimensions instead, and never touch this cache. At low
cardinality the pure B&B is faster.

There are 3 furthest caches. `Caches` only forwards the two public ones; layer 1 is private to the
module.

```
  request: range R
       │
       ▼
  Furthest::terminals ── hit ──► return the cached Terminals vector        (layer 2, key = R)
       │ miss
       ▼
  the caller's box and vertices, its diagonal, tp = leg.tp(R) ── one call per vertex (≤ 4) ──►
       │
       ▼
  FurthestCache ×(≤4)  ── each call: furthest fix from that vertex         (layer 1, key = vertex point)
       │
       ▼
  bundle the ≤4 distances into one Terminals vector, cache it under R, return it
```

The layers exist because a cache pays off when it is keyed on the question its *caller* asks, not on
the parameters it happens to pass down. Layer 1 already hits well on its own; the layers above still
win, for two different reasons.

`Furthest::terminals` asks something coarser than "furthest from this point" — it asks about one
range. Keying at that level is worth orders of magnitude in call count, because one lookup replaces
up to four layer-1 lookups plus the box and its diagonal. It wraps layer 1, it does not replace it.

`Furthest::end`'s question is as fine as layer 1's, so there is nothing to collapse. The win is that
its anchor is a track fix, `track[tp]`, which makes `tp` alone a unique key — hashing the point's raw
bits unnecessary. `end` is  a plain `Vec`, one slot per fix: no key to hash and no list to
search, bypassing layer 1's map entirely.

## 3. Layer 1 cache answers

### The entry

```rust
struct FurthestCacheEntry {
    start: usize,     // the furthest fix it FOUND
    stop: usize,      // the bound it COVERED
    distance: f64,
}
```
Same shape of claim as the closing cache — *found* on one side, *searched* on
the other — but in one dimension, so it is an interval rather than a rectangle:

```
  0                        start                 stop                        len-1
  ├──────────────────────────●────────────────────┤ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
                        found here          looked this far      never looked at

   request tp ...
     inside [start, stop]  ──► COVERED   reuse the answer, no scan
     beyond stop           ──► PARTIAL   resume the scan at `stop`
     before start          ──► COLD      the winner is not in range; scan from scratch
```

### Covered: the plain hit

`stop_requested <= entry.stop` (nothing new to look at) **and** `entry.start <= stop_requested` (the
winner is still inside the requested range). Identical logic to the closing cache's rectangle, one axis
short. Return the cached fix and distance untouched.

### Partial

The request reaches beyond what was searched. The entry cannot answer, but its **bound** is still worth
having: everything up to `entry.stop` has already been examined, so the new scan only needs
`[entry.stop ..= stop_requested]`.

That leaves a hole. The segment scan does not see the fixes before `entry.stop`, so **its result is not
the answer** — the real furthest may be the old winner. `insert` closes the hole.

Then the entry with that `start` has its `stop` stretched over the range just scanned, so the two merge
into one. **`insert` is where a partial result becomes a complete one, and it is the only place that
happens** — `furthest` returns `insert`'s value, never the raw scan result.

If the new segment *did* turn up something better, a **new** entry is added and the old one is kept.

### Cold — no usable entry

`get` only considers entries whose found fix is at or before the requested `stop`. If the request is
*narrower* than every stored winner's position, they are all filtered out, and with them any resume
bound — so the scan runs the whole range from 0.

## 4. Scan

The scan is a plain linear one, with two tricks:
* **Projection centred on the anchor**, so the anchor sits at the origin and the distance is just
  `x² + y²`.
* **Ranked on the square.** An argmax is invariant under a monotone map, so the `sqrt` is waste for
  every fix but the winner.

The ranking metric and the reported metric are different. Same trade as the closing search.

## 5. The diagonal floor

This is `terminals`' own correctness subtlety:

```rust
vertices.iter().map(|v| self.furthest(scorer, leg, v, tp).1.max(diagonal)).collect()
```

1. **One value per vertex, folded into the DP's first and last layer.** Not one value for the box.
   Collapsing the four to a single point is what igc-xc-score does and it under-bounds.
2. **The diagonal floor is critical.** `tp` is `leg.tp(range)` — `range[0]` for the entry leg,
   `range[1]` for the exit one — so each leg scans *away* from the turnpoint range and never visits a
   fix inside it. The floor is what covers those: any fix within is no further from any other than the
   box diagonal.
