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

## Build & Test

```bash
cargo build
cargo test
```

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
