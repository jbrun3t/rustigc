# Adding a league or rule

Everything lives in `rustigc/src/score/rules/`, one file per league. A rule never touches the
search: it only says which geometry to fly and what a distance and a closing gap are worth.

---

## Rule Traits

Rules are supposed to implement 2 traits:
* `RuleGeometry`
* `RuleDescription`

`RuleShape`, `RuleScore` and `RuleReport` are blanket-implemented in `rules/mod.rs`
rules should generally not touch those.

Override `minimum` on `RuleDescription` only when this one rule diverges from its league's numbers.
Most don't.

## Closing

`variant` answers with a `Closing` alongside the multiplier and the name: how large the closing gap
may get, and what it costs. A league states its law as an inherent const (`Cfd::CLOSING`) and its
rules name it; a rule over an open shape names `Closing::NONE`.

Both halves of the law — the free allowance and the largest gap that still closes once charged — are
a `Limit`, so each can be absolute (`Limit::fixed(3000.0)`), relative (`Limit::ratio(0.05)`) or the
larger of both. A free gap always closes, so the free half also floors what `limit` reports.

## Shapes

`type Shape` fixes the geometry *and* the turnpoint count — a rule cannot disagree with it. Needing a
geometry that isn't here is a `shapes/` change, not a `rules/` one.

| Shape | Flies | Note |
|---|---|---|
| `OpenPolyline<N>` | free distance over `N` points | **ends included**: a "3 turnpoint" rule is `<5>` |
| `ClosedCircuit<N>` | `N` turnpoints, back to the first | `<2>` is an out-and-return |
| `BalancedCircuit<N, MINSIDE>` | closed circuit, shortest leg ≥ `MINSIDE` of the total | per mille — `280` is the FAI 28 % rule |

Minimum is 2 either way: a straight line, or an out-and-return.

## Units

Everything is **unrounded metres**, and a multiplier applies to metres. Rounding to kilometres and
points happens once, at report time, so `minimum()` is on the metre scale too — 15 points is
`15_000.0`. Get this wrong and a minimum is off by 1000×, silently. A `Closing`'s `free` is metres
as well.

## Two constraints that bite

**`score` must be non-decreasing in `distance` and non-increasing in `gap`.** That is what makes
"largest distance, smallest gap" a valid upper bound on a subtree. A multiplier that improves as the
gap grows, or worsens as distance grows, breaks pruning silently — the search drops correct answers
with no error to point at. The blanket impl preserves this as long as `penalty` and `minimum` behave.

**`variant` is called with the raw distance, before the penalty**, by both `score` and `scored`, and
its `Closing` is measured against that same raw distance. Derive a multiplier or a limit from
anything else and the reported numbers stop explaining the reported score.

## Naming

Rule strings are identity, not decoration. They match igc-xc-score's own names, case-insensitively
(`"triangle plat"` here is `"Triangle plat"` there) — `xc-score-compare` folds case so a corpus
comparison can still pair rules across the two implementations by name. Renaming one breaks that
pairing silently.
