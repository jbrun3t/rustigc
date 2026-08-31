# rustigc

Parsing and cross-country scoring of IGC flight recorder files, for free-flying sports.

## Usage

```sh
cargo add rustigc
```

```rust
use rustigc::{FlightDetection, FlightSelection, Log};

let log = Log::new(&std::fs::read("flight.igc")?)?;

println!("{} — {} fixes", log.headers["PLT"].text, log.track.len());

let flights = log.track.flights();
let flight = flights.longest().expect("no flight detected");

if let Some(result) = log.score("xcontest", flight.start, flight.stop) {
    println!("{}: {} points over {} km", result.description, result.score, result.distance_km);
}
```

`Log` is the entry point; `RawLog` is the record-level view, which borrows from the input and
prints back valid IGC. The API is documented on [docs.rs](https://docs.rs/rustigc).

## Features

- `serde` — `Serialize`/`Deserialize` on the parsed records and the scoring report.
- `geojson` — drawing a flight and its score, through `Log::describe` and `Log::export`.

## Documentation

Beyond the API reference, the [`documentation/`](../documentation) directory covers the internals:
[how scoring works](../documentation/scoring-overview.md), [adding a
league](../documentation/add-a-league.md), [where we differ from
igc-xc-score](../documentation/igc-xc-score-diff.md), and [the IGC spec violations found in the
wild](../documentation/igc-spec-errors.md).

## Build & Test

```bash
cargo build
cargo test
```

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
