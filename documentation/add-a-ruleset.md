# Adding a ruleset or rule

A ruleset is a JSON file, loaded at run time with `Ruleset::from_json`.

`rustigc/rulesets/samples/` holds example rulesets; they are not built in.

A rule only says which geometry was flown and what its distance and closing gap are worth.

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

## Closing

A variant over a circuit states one `closing`; one over an open shape states none. The loader
refuses a variant that disagrees with its shape.

| `type` | | |
|---|---|---|
| `distance` | `distance_m`, `penalized` | closes out to a gap of `distance_m` meters |
| `ratio` | `ratio`, `penalized` | closes out to a gap of `ratio` times the distance |

`penalized` is required: `true` charges the gap its own length, `false` waives it.

A gap beyond that limit does not close.

A rulebook pricing one gap two ways is two variants of the same rule, each with its own
closing. For example, the French CFD waives the first 3 km, then charges in full out to 5 %.

## Constraints

Optional. A variant requires **everything** it lists.

| `type` | | |
|---|---|---|
| `min_score` | `points` | earns nothing below this score |
| `min_distance` | `meters` | earns nothing below this scoring distance |

## Variants

Listed **best rate first**. The first valid one wins, so a better rate further down would not
be reached; the loader refuses a multiplier that rises down the list.

What the report calls the threshold is the closing limit of the variant that won, not of the rule.

## Shapes

`shape` picks a geometry by its `type` and parameters.

| `type` | | Note |
|---|---|---|
| `open_polyline` | `points` | **ends included**: a "3 turnpoint" rule is `"points": 5` |
| `closed_circuit` | `turnpoints` | 2 turnpoints is an out-and-return |
| `balanced_circuit` | `turnpoints`, `min_side` | shortest leg ≥ `min_side` of the total, per mille — `280` is the FAI 28 % rule |

Only the combinations built into the library exist; any other is refused as an unknown shape. Each
one is compiled on its own so the search can be optimized for it.

## Units

Everything is **unrounded meters**, and a multiplier applies to meters. Rounding to kilometers and
points happens once, at report time, so `min_score` is on the meter scale too — 15 points is
`15000.0`.

## Pitfalls

**A rate must never improve as the gap grows, nor worsen as distance grows.** Variants listed best
rate first keep it: a threshold is then a step up.
