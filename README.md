# Rustigc

[![CI](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml/badge.svg)](https://github.com/jbrun3t/rustigc/actions/workflows/ci.yml)

An IGC toolbox for aviation sports written in Rust.

## Components

```
rustigc/
├── rustigc/             - Core Rust library
├── rustigc-py/          - Python bindings
├── rustigc-wasm/        - WebAssembly bindings
├── rustigc-tools/       - CLI tools
└── rustigc-test-data/   - Shared IGC test files and their scoring references
```

## Features

- ✅ Parse [IGC files](https://www.fai.org/page/igc-approved-flight-recorders) to structured records (RawLog)
  - ✅ Extract position fixes (lat/lon/altitude/time) into tracklog (Log)
  - ✅ Access flight metadata (pilot, glider, date, recorder info)
  - ✅ Roundtrip parse/write valid IGC
  - ⏳ Extension support:
    - `LOD`/`LAD`: Coordinate higher precision
    - `TDS`: Sub-second time division
- ✅ [Flight scoring](documentation/scoring.md):
  - Generic scoring over a configurable number of turnpoints
  - Optimized for fast searches
  - Supports XContest, FFVL's CFD, and more
- ✅ Python bindings with numpy support
- ✅ WASM bindings
- ✅ GeoJSON export
- ✅ CLI tools
- 🏗️ Takeoff/landing detection (currently basic - average speed ~15km/h)
- ⏳ Flight dynamics smoothing
- ⏳ Flight phases identification

## Quick Start

### Rust Library

```rust
use rustigc::{FlightDetection, FlightSelection, Log, Ruleset};

let log = Log::new(&std::fs::read("flight.igc")?)?;
println!("Pilot: {}", log.headers["PLT"].text);

let flights = log.track.flights();
let flight = flights.longest().expect("no flight detected");
if let Some(score) = log.score(&Ruleset::preset("xcontest")?, flight.start, flight.stop)? {
    println!("{}: {} points", score.description, score.score);
}
```

See [rustigc/README.md](rustigc/README.md)

### Python

```python
from rustigc_py import Log, Ruleset

with open("flight.igc", "rb") as f:
    log = Log.from_bytes(f.read())
print(f"Pilot: {log.pilot_name}")

score = log.score(Ruleset.preset("xcontest"))
if score:
    print(f"{score.description}: {score.score} points")
```

See [rustigc-py/README.md](rustigc-py/README.md)

### TypeScript

```ts
import { readFileSync } from "node:fs";
import { Log, Ruleset } from "rustigc-wasm";

const log = new Log(readFileSync("flight.igc"));
console.log(`Pilot: ${log.header("PLT")?.text}`);

const score = log.score(Ruleset.preset("xcontest"));
if (score) console.log(`${score.description}: ${score.score} points`);
```

See [rustigc-wasm/README.md](rustigc-wasm/README.md), which covers instantiating the wasm first.

### CLI tool

```bash
rustigc-xc-score --ruleset xcontest < flight.igc
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

## Status: Alpha

`rustigc` is still very early in its development cycle. It is a pet project I'm using to learn Rust.
I'm still exploring everything Rust has to offer, so there is a good chance some things in there are
not done correctly or could be improved. Public APIs are likely to change.

However, `rustigc` is tested on [more than 130 000 tracklogs](documentation/scoring.md),
[accounting for many oddities found in the real world](documentation/igc-spec-errors.md).

Its decoder and scoring modules are robust enough for production. If you have a tracklog that does
not parse or score correctly, please open an issue and share the tracklog.

The analysis module, especially takeoff and landing detection, is not on par yet.

## Credit

The following sources have been extremely helpful in building `rustigc`:

* [Ondřej Palkovský's paper on Paragliding Competition Tracklog Optimization](https://web.archive.org/web/20230320111732/http://www.penguin.cz/~ondrap/algorithm.pdf)
* [igc-xc-score](https://github.com/mmomtchev/igc-xc-score)
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
