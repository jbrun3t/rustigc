# Rustigc - Core Library

## Build & Test

All as usual

```bash
cargo build
```

The rust core library comes with some unit tests
but is still lacking a proper benchmark ATM

```
cargo test
```

## Usage example


```rust
use rustigc::{Log, RawLog};

let content = std::fs::read_to_string("track.igc")?;

let raw = RawLog::new(&content)?;

// Printing a Rawlog outputs a valid IGC, for future comparison
print!("{}", raw);

// Create Log directly with Log::new()
let log:Log = raw.into();

println!("Pilot: {:?}", log.header["PLT"]);
println!("Fixes: {}", log.track.len());

for fix in &log.track {
    println!("{} - {:.5}, {:.5} @ {}m",
        fix.timestamp,  // u32: seconds since midnight
        fix.latitude,   // f64: decimal degrees
        fix.longitude,  // f64: decimal degrees
        fix.gnss_alt);  // i32: meters
}


```

## License

MIT
