# Rustigc

Fast IGC library for aviation sports

## Usage

Add to `Cargo.toml`:
```toml
[dependencies]
rustigc = "0.1"
```

### Basic parsing

```rust
use rustigc::Log;

let content = std::fs::read("flight.igc")?;
let log = Log::new(&content)?;

// Access metadata
println!("Pilot: {}", log.headers["PLT"].text);
println!("Glider: {}", log.headers["GTY"].text);
println!("Date: {}", log.headers["DTE"].text);
println!("Recorder: {} - {}", log.recorder.manufacturer, log.recorder.uid);

// Access track points
println!("Track points: {}", log.track.len());

for fix in &log.track {
    println!("{} - {:.5}, {:.5} @ {}m/{}m",
        fix.timestamp,  // u32: seconds since midnight
        fix.lat,        // f64: decimal degrees
        fix.lon,        // f64: decimal degrees
        fix.gnss_alt,   // i32: GNSS altitude in meters
        fix.baro_alt);  // i32: barometric altitude in meters
}
```

### Parsing with RawLog

`RawLog` keeps references to the original bytes for minimal allocations:

```rust
use rustigc::{Log, RawLog};

let content = std::fs::read("flight.igc")?;
let raw = RawLog::new(&content)?;

// Print back valid IGC (roundtrip)
println!("{}", raw);

// Convert to Log when needed
let log: Log = raw.try_into()?;
```

### Error handling

```rust
use rustigc::Log;

match Log::new(&content) {
    Ok(log) => println!("Parsed {} fixes", log.track.len()),
    Err(e) => eprintln!("Parse error: {}", e),
}
```

### Scoring

`Log::score` runs every rule of a league in one search and reports the best. `league_names()` lists
what the `league` argument accepts.

```rust
use rustigc::{FlightDetection, FlightSelection, Log};

let log = Log::new(&content)?;

// Scoring works on a fix window; flight detection gives a sensible one
let flights = log.track.flights();
let flight = flights.longest().unwrap();

if let Some(result) = log.score("xcontest", flight.start, flight.stop) {
    println!("{}: {} points over {} km", result.description, result.score, result.distance);
    println!("turnpoints: {:?}", result.turnpoints);
}
```

See [`documentation/scoring-overview.md`](../documentation/scoring-overview.md) for how the search
works, and [`documentation/add-a-league.md`](../documentation/add-a-league.md) to add a league.

### GeoJSON export

Behind the `geojson` feature. A log renders itself and the layers it is given

```toml
rustigc = { version = "0.1", features = ["geojson"] }
```

Every feature declares a `role`:

| `role` | geometry | carries |
| --- | --- | --- |
| `track` | LineString, 3D | the whole flown line and its `coordTimes` |
| `marker` | Point, 3D | `name`, `fix`, `timestamp` |
| `leg` | LineString | a scored side: `name` (`leg0`…), `from`, `to`, `distance` |
| `closing` | LineString | a circuit's closing side, named `entry`, `exit` or `gap` |
| `metadata` | none | `datetime`: the RFC 9557 instant every `timestamp` counts from and IGC metadata as JSON |
| `score` | none | `rule`, `score`, `distance`, `raw_distance`, `gap`, `penalty`, `multiplier`, `circuit` |


Markers are named `takeoff`, `landing`, `entry`, `tp0`…`tp(n-1)` and `exit`, and a leg's `from`/`to` name
them.

An open task is all `leg`s, `leg0` running from its entry and the last to its exit;
a circuit's `leg`s close over its turnpoints alone, with the closing legs beside them.
A leg's `distance` is its geodesic length in kilometers, so the legs of a task do
not add up to the `score`'s `distance`, which is net of the penalty, if any

The time reference is UTC midnight of the flight's date read in the zone the track starts in, stated
once as RFC 9557 — `2022-08-05T01:00:00+01:00[Europe/London]`.

Every position over a fix — the track's line and each marker — is `[lon, lat, gnss_alt]`, trimmed to
eight decimals, past anything a fix records.

## Build & Test

```bash
cargo build
cargo test
```

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
