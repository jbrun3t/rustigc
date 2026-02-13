# Rustigc

Fast IGC file parser and analysis library for aviation sports (gliding, paragliding, hang gliding).

## Features

- Parse IGC files into structured data
- Roundtrip: parse and write back valid IGC
- Optional serde support for serialization

## Usage

Add to `Cargo.toml`:
```toml
rustigc = "0.1"
# or with serde support:
rustigc = { version = "0.1", features = ["serde"] }
```

### Basic parsing

```rust
use rustigc::Log;

let content = std::fs::read_to_string("flight.igc")?;
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

`RawLog` keeps references to the original string for minimal allocations:

```rust
use rustigc::RawLog;

let content = std::fs::read_to_string("flight.igc")?;
let raw = RawLog::new(&content)?;

// Print back valid IGC (roundtrip)
println!("{}", raw);

// Convert to Log when needed
let log: Log = raw.into();
```

### Error handling

```rust
use rustigc::Log;

match Log::new(&content) {
    Ok(log) => println!("Parsed {} fixes", log.track.len()),
    Err(e) => eprintln!("Parse error: {}", e),
}
```

## Build & Test

```bash
cargo build
cargo test
```

## License

MIT
