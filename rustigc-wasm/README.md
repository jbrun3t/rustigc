# rustigc-wasm

WASM bindings for [rustigc](../rustigc), as the `rustigcjs` JavaScript module.

## Prerequisites

The bindings are an optional part of the workspace: a plain `cargo build` does not need any of
this, and nothing else in the tree does either.

| | Debian package |
|---|---|
| `wasm32-unknown-unknown` target | `libstd-rust-dev-wasm32` |
| `wasm-pack` | `wasm-pack` |
| `wasm-bindgen` | `wasm-bindgen` |
| `wasm-opt` | `binaryen` |

## Build

```sh
wasm-pack build rustigc-wasm --target nodejs
```

Output lands in `pkg/`, `main` pointing at `rustigcjs.js`.

## Usage

```js
const { Log, league_names } = require("./pkg/rustigcjs.js");

const log = new Log(readFileSync("flight.igc"));   // Uint8Array

log.fix_count                            // 25459
log.header("PLT")                        // { text: "Mike Young", origin: "flightrecorder" }
log.longest_flight()                     // { start: 125, stop: 25425 }
log.score("xcontest")                    // over the longest detected flight
log.describe("xcontest")                 // GeoJSON: detects, scores and draws

league_names()                           // what score and describe accept
```

Every method is documented in `pkg/rustigcjs.d.ts`, generated from the doc comments in `src/lib.rs`
along with interfaces for `Fix`, `Flight`, `Header`, `DateTime` and `Score`.

GeoJSON crosses as a string rather than an object: a track would otherwise become as many JS
arrays, and a caller either prints it or hands it to a map. `JSON.parse` it for objects.

`describe` detects, scores and draws in one call. `export` draws the `flights`/`score` results
handed to it and searches for nothing itself — use it when you already have them.

## Bulk track access

`track` builds one JS object per fix through `serde_wasm_bindgen`; `track_bytes` hands over the
`#[repr(C)] Fix` array as a `Uint8Array` and leaves the decoding to the caller. Measured on
`fai-01`, 25 459 fixes:

| | time | memory |
|---|---|---|
| `track` — serde to objects | 10 ms | 3.40 MB heap |
| `track_bytes` — copy only | 0.3 ms | 0.78 MB, exactly the raw track |
| `track_bytes` + a `DataView` loop to the *same* objects | 1.05 ms | 2.5 MB heap |

Decoding the bytes in JS is **10x faster than serde for identical objects**. Both are a single
call from JS — the difference is what happens underneath. `serde_wasm_bindgen` builds the array
from inside wasm and calls back out to JS as it goes: `Object::new` and an array append per fix,
against one buffer copy for `track_bytes`.

`track` needs nothing on the JS side; `track_bytes` is for a caller who will decode, and
`js/track.js` ships a decoder so that caller does not have to write one:

```js
const { fixes } = require("./js/track.js");

log.track                // objects, straight from the binding
fixes(log.track_bytes)   // the same objects, 10x faster
```

Using it is optional — it is plain JS over a documented layout, not part of the binding.

The layout is 32 bytes per fix, little-endian, matching `Fix`:

| offset | 0 | 4 | 8 | 16 | 24 | 28 |
|---|---|---|---|---|---|---|
| | `u32` timestamp | *pad* | `f64` lat | `f64` lon | `i32` baro_alt | `i32` gnss_alt |

## Example CLI

`js/rustigc-js-score.js` mirrors `rustigc-xc-score`, minus its `json` format, reading IGC on
stdin. It is the example and smoke test for these bindings, not a supported tool.

```sh
node rustigc-wasm/js/rustigc-js-score.js --league xcontest --format human < flight.igc
```

Its `human` and `geojson` output is diffed against `rustigc-xc-score` over the whole of
`test_data/real/` — 52 comparisons, byte for byte.

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
