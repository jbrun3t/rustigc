# Changelog

## 0.2.0

### Breaking

- Scoring takes a `Ruleset` instead of a league name, in the library and the bindings.
- `league_names` renamed `preset_names`.
- `ScoringResult.league` renamed `ruleset`.
- `ScoreError::UnknownLeague` renamed `UnknownPreset`.
- `Scorer::solve` returns an `Option`, not a `Result`.
- `penalty` is in kilometres.
- GeoJSON leg `distance_m` is in metres.
- `geojson` is no longer a default feature.
- `cfd-preview` removed.
- `1tp`, `2tp`, `line` and `out-and-return` are no longer built in; see `rulesets/samples/`.
- CLI: `--league` renamed `--ruleset`.

### Added

- `Ruleset::from_json` to score against a custom ruleset.
- CLI: `--ruleset-file`.
