# Rustigc

[![CI](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml/badge.svg)](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml)

Fast IGC toolbox for aviation sports written in Rust.

## Components

```
rustigc/
├── rustigc/             - Core Rust library
├── rustigc-py/          - Python bindings
└── rustigc-tools/       - CLI tools
```

## Features

- ✅ Parse IGC files to structured records (RawLog)
  - ✅ Extract position fixes (lat/lon/altitude/time) into tracklog (Log)
  - ✅ Access flight metadata (pilot, glider, date, recorder info)
  - ✅ Roundtrip parse/write valid IGC
  - ⏳ Extension support:
    - `LOD`/`LAD`: Coordinate higher precision
    - `TDS`: Sub-second time division
- ✅ Flight scoring: see [dedicated documentation](documentation/scoring-overview.md)
  - Generic scoring over a configurable number of turnpoints
  - Optimized for fast searches
  - Supports Xcontest, FFVL's CFD, and more ...
- ✅ Python bindings with numpy support
- ⏳ WASM bindings
- 🏗️ Takeoff/landing detection (currently basic - average speed ~15km/h)
- 🏗️ CLI tools
  - ⏳ Geojson support
- ⏳ Flight dynamics smoothing
- ⏳ Flights phases identification

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
from rustigcpy import Log

log = Log.from_file("flight.igc")

print(f"Pilot: {log.pilot_name}")
print(f"Glider: {log.glider_type}")
print(f"Fixes: {len(log.track)}")

# Flight phases
print(f"Takeoff: {log.takeoff}")
print(f"Landing: {log.landing}")
```

See [rustigc-py/README.md](rustigc-py/README.md)

### CLI tool

```bash
rustigc-xc-score --league xcontest < flight.igc
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

## Status : Pre-alpha

`rustigc` is still very early in its development cycle. It is a pet project I'm using to learn Rust.
I'm still exploring everything Rust has to offer, so there is a good chance some things in there are
not done correctly or could be improved. Public APIs are likely to be reworked and change.

However, `rustigc` is tested on thousands of tracklogs, [accounting for many oddities found in the
real world](documentation/igc-spec-errors.md).

If you have a tracklog that does not parse or score correctly, please open an issue and share the
tracklog.

## Credit

The following sources of information have been extremely helpful to build `rustigc` so far:

* [FAI IGC Specificication](https://www.fai.org/page/igc-approved-flight-recorders)
* [Ondřej Palkovský's Paper on Paragliding Competition Tracklog Optimization](https://web.archive.org/web/20230320111732/http://www.penguin.cz/~ondrap/algorithm.pdf)
* [igc-xc-score](https://github.com/mmomtchev/igc-xc-score) (differences between `igc-xc-score` and `rustigc` scoring are documented [here](documentation/igc-xc-score-diff.md))
* [Python libigc](https://github.com/surajmandalcell/libigc)
* [Rust igc_parser](https://github.com/LWEdslev/igc_parser)
* [Rust igc-rs](https://github.com/Joey9801/igc-rs)

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0` for `rustigc` and `rustigc-py`, `GPL-2.0-or-later`
for `rustigc-tools`.

The Classpath exception means linking against `rustigc` or `rustigc-py` — statically or
dynamically, from Rust, Python, or any other language — does not require your own code to be
GPL-licensed. Only modifications to `rustigc`/`rustigc-py` themselves stay under the GPL. See
[LICENSE](LICENSE) for the full text.
