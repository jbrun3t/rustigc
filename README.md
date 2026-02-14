# Rustigc

[![CI](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml/badge.svg)](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml)

Fast IGC file parser for aviation sports (gliding, paragliding, hang gliding) written in Rust.

## Components

```
rustigc/
├── rustigc/             - Core Rust library
├── rustigc-py/          - Python bindings (low-level)
├── rustigc-py-wrapper/  - Python wrapper (high-level API)
└── rustigc-tools/       - CLI tool
```

## Features

- ✅ Parse IGC files to structured records (RawLog)
- ✅ Extract position fixes (lat/lon/altitude/time) into tracklog (Log)
- ✅ Access flight metadata (pilot, glider, date, recorder info)
- ✅ Roundtrip parse/write valid IGC
- ✅ Serde support for JSON serialization
- ✅ Takeoff/landing detection (basic - average speed ~15km/h)
- ✅ Python bindings with numpy integration
- ⏳ CLI tool (currently just JSON dump)

## Quick Start

### Rust Library

```rust
use rustigc::Log;

let content = std::fs::read("flight.igc")?;
let log = Log::new(&content)?;

println!("Pilot: {}", log.headers["PLT"].text);
println!("Fixes: {}", log.track.len());
```

See [rustigc/README.md](rustigc/README.md)

### Python

```python
from rustigcpy_wrapper import Log

log = Log.from_file("flight.igc")

print(f"Pilot: {log.pilot_name}")
print(f"Glider: {log.glider_type}")
print(f"Fixes: {len(log.track)}")

# Flight phases
print(f"Takeoff: {log.takeoff}")
print(f"Landing: {log.landing}")
```

See [rustigc-py-wrapper/README.md](rustigc-py-wrapper/README.md) for high-level API
See [rustigc-py/README.md](rustigc-py/README.md) for low-level bindings

### CLI Tool

```bash
# Parse IGC and output JSON
rustigc < flight.igc

# Or with pipe
cat flight.igc | rustigc > flight.json

# Quiet mode for profiling (no output)
rustigc --quiet < flight.igc
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
