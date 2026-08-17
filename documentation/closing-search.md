# The closing search and its cache

This describes how `score/cache/closing.rs` works, and why it is shaped the way it is. It answers
one question, as fast as it can: for a circuit candidate, how far apart are the two ends of the
flight?

---

## TL;DR

* A request is **two numbers**, `entry_stop` and `exit_start`, because the ranges always reach the ends
  of the window. Everything else follows from that.
* The **search** finds the closest pair between `[0..entry_stop]` and `[exit_start..last]`, ranking in
  a flat projected plane and asking the tree *"anything closer than the best found so far?"*.
* The **cache** stores each answer as a **rectangle** in `(entry_stop, exit_start)` space, spanning
  *from the fixes it found* to *the bounds it searched*. Any request landing inside reuses it exactly.
* Landing outside means one of two things: **the request reaches fixes that were never searched**, or
  **the cached winner no longer lies in the requested range**
* It hits **75 to 99.9 %** of the time, and a pass generally runs **1 to 100** real searches (median 17
  over the corpus).

---

## 1. Closing distance

A circuit has to come back near where it started. Scoring one therefore needs the shortest distance
between a fix near the **start** of the flight and a fix near the **end**.

```
      entry range                                          exit range
 |<------------------->|                              |<--------------->|
 0 . . . . . . . . entry_stop                    exit_start . . . . . last
 └──────────────────── the flight, fix by fix ──────────────────────────┘

         find the closest pair, one fix from each range
```

* **The ranges reach the ends of the window.** The entry range always starts at fix 0 and the exit
  range always ends at the last fix.
* **A request consists of just two numbers**: `entry_stop` and `exit_start`.

## 2. Users

Two callers want two different halves of the same answer:

| Caller | Wants | Used for |
|---|---|---|
| `gap` | the **distance** | pruning: `rule.score(bound, gap)` gives the candidate's upper bound |
| `endpoints` | the **two fix indices** | the report: which fixes are shown as the closing pair |

That split matters for a subtlety in §4: the distance the search computes is a *fast approximation*,
and it is only ever used to prune. `report` takes the **indices** and re-measures them with the exact
geodesic. So the search's own distance never reaches a published score.

### Overlapping circuits

Before any of this, `Circuit::closing` checks `Candidate::first_last_overlap()` — whether the first
turnpoint's range still overlaps the last one's (`first[1] >= last[0]`). While they overlap, the
circuit is not pinned down enough to *have* two distinct ends.

In that case there is no search at all: it returns a gap of `0.0`, the most favourable gap possible,
so the bound stays valid.

## 3. Two spatial structures

The module uses two R-trees, for completely different purposes:

| | structure | indexes |
|---|---|---|
| **cache** | *index* space — fix numbers | which questions have been answered |
| **search** | *geographic* space — projected metres | where the fixes actually are |

## 4. Closing distance search

Given `entry_stop` and `exit_start`, find the closest pair as fast as possible:

1. Projection: Take a bounding box over both ranges, and build a `CheapProjection` centred on
   it. That turns (lon, lat) into flat metres, so distances become plain Pythagoras. It is an
   approximation, and it is deliberately anchored per call.
1. Pick the shortest side: The **shorter** range goes into the tree; the **longer** one is walked
   fix by fix. Bulk-loading an R-tree *sorts*, so building costs more per point than querying does
   once the radius of step 3 is refuting most queries at the root.
1. Walk, with a shrinking radius: This is **not** checking "who is nearest to fix `i`?" for every
   `i`. It is checking "is anything closer than the best pair found so far?".
   A plain nearest-neighbour query — one that always starts from an infinite radius — computes an
   exact answer for every single fix and then throws almost all of them away. On a test with
   `fai-02` only 3 of 19 862 queries improve on the best.
1. Compute the Fcc distance on the winning pair

## 5. Caching

### Key idea

A request is two numbers, `(entry_stop, exit_start)`, so plot every request as a **point on a 2-D
grid**. A completed search is then not a point but a **rectangle** — the whole set of future requests
whose answer it already knows.

Two facts about a finished search make that rectangle:

1. It looked at everything in `[0, E]` and `[X, last]` — call those the **bounds searched**.
2. It found a winning pair at fixes `(e, x)` — the **fixes found**. Necessarily `e ≤ E` and `x ≥ X`.

A later request `(E', X')` can reuse the answer when **both** of these hold:

* **its ranges are a subset of what was searched** — nothing new to look at:
  `E' ≤ E` and `X' ≥ X`
* **the old winner is still valid for it** — the answer is still reachable:
  `E' ≥ e` and `X' ≤ x`

Together they mean the cached optimum *is* the new request's optimum.

Those four inequalities are a rectangle:

```
   exit_start
   (larger = narrower exit range)
        ▲
        │
    x ──┼─ ─ ─ ─ ┌────────────┐        x = exit fix found
        │        │            │
        │        │    HIT     │        any request landing in here
        │        │            │        reuses the cached answer as-is
    X ──┼─ ─ ─ ─ └────────────┘        X = exit_start searched
        │        ╎            ╎
        └────────┴────────────┴──────► entry_stop
                 e            E         (larger = wider entry range)

          e = entry fix found     E = entry_stop searched
```

`ClosingCacheEntry::envelope` builds this box, from corners `(e, X)` and `(E, x)`.

### Cache misses

A request outside the rectangle is a miss, for one of **two** reasons depending on the edge it left
through:

| Leaves via | Condition | Meaning |
|---|---|---|
| right edge | `entry_stop > E` | the request reaches entry fixes that were never searched |
| bottom edge | `exit_start < X` | the request reaches exit fixes that were never searched |
| left edge | `entry_stop < e` | the entry range no longer contains the winning fix |
| top edge | `exit_start > x` | the exit range no longer contains the winning fix |

* **Right and bottom** = the request is *wider* than what was searched. There might be a better pair
  among the fixes never examined, so the search must run again.
* **Left and top** = the request is *narrower*, and narrow enough that the cached winner has fallen
  outside it. Something else is optimal now.

