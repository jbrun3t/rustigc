# Rustigc

[![CI](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml/badge.svg)](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml)

Fast IGC file parser for aviation sports (gliding, paragliding, hang gliding) written in Rust.

## Components

```
rustigc/
├── rustigc/          - Core Rust library
├── rustigc-py/       - Python bindings
└── rustigc-tools/    - CLI tool
```

## Features

- ✅ Parse IGC files to structured records (RawLog)
- ✅ Extract position fixes (lat/lon/altitude/time) into tracklog (Log)
- ✅ Access flight metadata (pilot, glider, date, recorder info)
- ✅ Roundtrip parse/write valid IGC
- ✅ Serde support for JSON serialization
- ⏳ Takeoff/landing detection (basic - average speed ~15km/h)
- ⏳ CLI tool (currently just JSON dump)
- ⏳ Python bindings (minimal, needs optimization)

## Quick Start

### Rust Library

```rust
use rustigc::Log;

let content = std::fs::read_to_string("flight.igc")?;
let log = Log::new(&content)?;

println!("Pilot: {}", log.headers["PLT"].text);
println!("Fixes: {}", log.track.len());
```

See [rustigc/README.md](rustigc/README.md)

### Python Bindings

```python
import rustigcpy

with open("flight.igc") as f:
    log = rustigcpy.Log.from_string(f.read())

print(f"Pilot: {log.pilot_name()}")
print(f"Fixes: {len(log)}")
```

See [rustigc-py/README.md](rustigc-py/README.md)

### CLI Tool

```bash
# Parse IGC and output JSON
rustigc < flight.igc

# Or with pipe
cat flight.igc | rustigc > flight.json
```

See [rustigc-tools/README.md](rustigc-tools/README.md)

## Build

```bash
# Build all components
cargo build --release

# Run tests
cargo test

# Build Python bindings
cd rustigc-py
pip install maturin
maturin develop
```

## License

MIT
