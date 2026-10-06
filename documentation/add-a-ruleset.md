# Adding a ruleset or rule

An embedded ruleset is a JSON file in `rustigc/src/score/rules/rulesets/`, listed in the `rulesets!`
call in `rules/ruleset.rs` — with its rulebook's URL as a comment there, since JSON takes none. Adding
one is writing that file; no Rust changes unless it needs a geometry that does not exist yet. A
caller can also score against its own file without rebuilding, through `Ruleset::from_json`.

A rule never touches the search. It says which geometry to fly and what a distance and a closing gap
are worth.

---

## The file

Three levels: a ruleset lists rules, a rule prices one geometry through one or more variants, and a
variant states how its geometry closes and what it requires, alongside its name and multiplier.

```json
{
  "name": "cfd",
  "rules": [
    { "shape": { "type": "closed_circuit", "turnpoints": 3 },
      "variants": [
        { "name": "triangle plat", "multiplier": 1.2,
          "closing": { "type": "distance", "distance_m": 3000.0, "penalized": false },
          "constraints": [
            { "type": "min_score", "points": 15000.0 }
          ] }
      ] }
  ]
}
```

Anything the file gets wrong — a misspelled key, an unknown shape, a variant disagreeing with its
geometry — is a panic at load for an embedded file, and `rulesets_all_load` forces it so CI is what
fails. `Ruleset::from_json` reports the same defects as an error.

## Closing

A variant over a circuit states one `closing`; one over an open shape states none. The loader
refuses a variant that disagrees with its shape.

| `type` | | |
|---|---|---|
| `distance` | `distance_m`, `penalized` | closes out to a gap of `distance_m` metres |
| `ratio` | `ratio`, `penalized` | closes out to a gap of `ratio` times the distance |

`penalized` is required: `true` charges the gap its own length, `false` waives it. A gap beyond what
it reaches does not close at all.

A rulebook pricing one gap two ways — CFD waives the first 3 km, then charges in full out to 5 % —
is two variants of the same rule, each with its own closing.

## Constraints

Optional. A variant requires **everything** it lists.

| `type` | | |
|---|---|---|
| `min_score` | `points` | earns nothing below this score |
| `min_distance` | `meters` | earns nothing below this scoring distance |

## Variants

Listed **best rate first**. `pick` takes the first that earns anything, so a better rate further down
would never be reached; the loader refuses a list whose multiplier rises. Two variants may share a
name and a rate and differ only in how they close, which is how CFD's two closings are
written.

What the report calls the threshold is the closing limit of the variant that won, not of the rule.

## Shapes

`shape` names one of the monomorphisations listed in `shapes/table.rs`, by the parameters that pick
it — not by its Rust type, and not by the cardinality the search derives from it.

| `type` | | Note |
|---|---|---|
| `open_polyline` | `points` | **ends included**: a "3 turnpoint" rule is `"points": 5` |
| `closed_circuit` | `turnpoints` | 2 turnpoints is an out-and-return |
| `balanced_circuit` | `turnpoints`, `min_side` | shortest leg ≥ `min_side` of the total, per mille — `280` is the FAI 28 % rule |

Minimum is 2 either way: a straight line, or an out-and-return. Needing a geometry the table does
not carry is a `shapes/` change: add the type to the `shapes!` list, or write the geometry first.

## Units

Everything is **unrounded metres**, and a multiplier applies to metres. Rounding to kilometres and
points happens once, at report time, so `min_score` is on the metre scale too — 15 points is
`15000.0`. Get this wrong and a threshold is off by 1000×, silently. A `distance` closing's `distance_m`
is metres as well; `ratio` is a fraction.

State a threshold in the unit its rulebook uses and never convert: `min_score` for a rulebook stating
points, `min_distance` for one stating a distance. The same band written the other way is the
distance times the rate, and that product is not the round number a rulebook means — `25_000.0 *
2.3` is `57499.99999999999`, so a ×2.3 band written in points refuses a 25.000 km triangle.

## The constraint that bites

**`score` must be non-decreasing in `distance` and non-increasing in `gap`.** That is what makes
"largest distance, smallest gap" a valid upper bound on a subtree. A multiplier that improves as the
gap grows, or worsens as distance grows, breaks pruning silently — the search drops correct answers
with no error to point at. Variants preserve it as long as they run best rate first: a threshold is
then a step up, which is still non-decreasing.

A variant's closing is measured against the **raw distance, before the penalty**. Its score and
its thresholds are measured on the distance net of the penalty.

## Naming

Rule strings are identity, not decoration. They match igc-xc-score's own names, case-insensitively
(`"triangle plat"` here is `"Triangle plat"` there) — `xc-score-compare` folds case so a corpus
comparison can still pair rules across the two implementations by name. Renaming one breaks that
pairing silently.
