# rustigc

Parsing and cross-country scoring of IGC flight recorder files, for free-flying sports.

## Usage

```sh
cargo add rustigc
```

```rust
use rustigc::{FlightDetection, FlightSelection, Log, Ruleset};

let log = Log::new(&std::fs::read("flight.igc")?)?;

println!("{} — {} fixes", log.headers["PLT"].text, log.track.len());

let flights = log.track.flights();
let flight = flights.longest().expect("no flight detected");

let xcontest = Ruleset::preset("xcontest")?;
if let Some(result) = log.score(&xcontest, flight.start, flight.stop)? {
    println!("{}: {} points over {} km", result.description, result.score, result.distance_km);
}
```

`Log` is the entry point; `RawLog` is the record-level view, which borrows from the input and
prints back valid IGC. The API is documented on [docs.rs](https://docs.rs/rustigc).

## Features

- `geojson` — drawing a flight and its score, through `Log::describe` and `Log::export`.

## Rulesets

`rulesets/presets/` holds the rulesets built into the library, reached with `Ruleset::preset`.
`rulesets/samples/` holds more that are not built in; they ship in the crate for packagers to
install, under `/usr/share/rustigc/rulesets/` for instance, and are read with `Ruleset::from_json`
or `rustigc-xc-score --ruleset-file`.

## Documentation

Beyond the API reference, the [`documentation/`](../documentation) directory covers the internals:
[how scoring works](../documentation/scoring.md), [adding a
ruleset](../documentation/add-a-ruleset.md), [where we differ from
igc-xc-score](../documentation/scoring.md#8-technical-differences-from-igc-xc-score), and [the IGC
spec violations found in the wild](../documentation/igc-spec-errors.md).

## Build & Test

```bash
cargo build
cargo test
```

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
