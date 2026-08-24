# Rustigc

[![CI](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml/badge.svg)](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml)

Fast IGC toolbox for aviation sports written in Rust.

## Components

```
rustigc/
├── rustigc/             - Core Rust library
├── rustigc-py/          - Python bindings
├── rustigc-wasm/        - WASM bindings
└── rustigc-tools/       - CLI tools
```

Each has its API documented in its own language's format: rustdoc for the core
([docs.rs](https://docs.rs/rustigc)), docstrings for Python (`help(rustigcpy.Log)`), and
`pkg/rustigcjs.d.ts` for JavaScript. The READMEs below cover installing and building.

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
  - Supports XContest, FFVL's CFD, and more
- ✅ Python bindings with numpy support
- ✅ WASM bindings
- ✅ GeoJSON export
- 🏗️ Takeoff/landing detection (currently basic - average speed ~15km/h)
- 🏗️ CLI tools
- ⏳ Flight dynamics smoothing
- ⏳ Flight phases identification

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
from rustigcpy import Logs

log = Log.from_file("flight.igc")

print(f"Pilot: {log.pilot_name}")
print(f"Glider: {log.glider_type}")
print(f"Fixes: {len(log.track)}")

# Flight sections
flight = log.flights().longest
print(f"Takeoff: {flight.takeoff}")
print(f"Landing: {flight.landing}")
```

See [rustigc-py/README.md](rustigc-py/README.md)

### JavaScript

```js
const { Log } = require("./pkg/rustigcjs.js");

const log = new Log(readFileSync("flight.igc"));

console.log(log.header("PLT"), log.fix_count);
console.log(log.score("xcontest"));
```

See [rustigc-wasm/README.md](rustigc-wasm/README.md)

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

## Status: Pre-alpha

`rustigc` is still very early in its development cycle. It is a pet project I'm using to learn Rust.
I'm still exploring everything Rust has to offer, so there is a good chance some things in there are
not done correctly or could be improved. Public APIs are likely to change.

However, `rustigc` is tested on thousands of tracklogs, [accounting for many oddities found in the
real world](documentation/igc-spec-errors.md).

If you have a tracklog that does not parse or score correctly, please open an issue and share the
tracklog.

## Credit

The following sources have been extremely helpful in building `rustigc` so far:

* [FAI IGC Specification](https://www.fai.org/page/igc-approved-flight-recorders)
* [Ondřej Palkovský's Paper on Paragliding Competition Tracklog Optimization](https://web.archive.org/web/20230320111732/http://www.penguin.cz/~ondrap/algorithm.pdf)
* [igc-xc-score](https://github.com/mmomtchev/igc-xc-score) (differences between `igc-xc-score` and `rustigc` scoring are documented [here](documentation/igc-xc-score-diff.md))
* [Python libigc](https://github.com/surajmandalcell/libigc)
* [Rust igc_parser](https://github.com/LWEdslev/igc_parser)
* [Rust igc-rs](https://github.com/Joey9801/igc-rs)

## License

* `GPL-2.0-or-later WITH Classpath-exception-2.0` for the library and its bindings
* `GPL-2.0-or-later` for `rustigc-tools`

The Classpath exception means linking against the library or its bindings — statically or
dynamically, from Rust, Python, JavaScript, or any other language — does not require your own code
to be GPL-licensed. Only modifications to `rustigc` itself stay under the GPL. See
[LICENSE](LICENSE) for the full text.
