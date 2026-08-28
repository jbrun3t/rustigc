# rustigc-wasm

WASM bindings for [rustigc](../rustigc), as the `rustigc-wasm` npm package.

| directory | package | |
|---|---|---|
| `pkg/` | `rustigc-wasm` | the bindings: wasm, ESM glue and types, all generated |
| `utils/` | `rustigc-utils` | optional helpers: `Log` with local times, `track_bytes` decoder |
| `score/` | — | example CLI, not published |

## Prerequisites

| | Debian package |
|---|---|
| `wasm32-unknown-unknown` target | `libstd-rust-dev-wasm32` |
| `wasm-pack` | `wasm-pack` |
| `wasm-bindgen` | `wasm-bindgen` |
| `wasm-opt` | `binaryen` |

## Build

```sh
wasm-pack build rustigc-wasm --target web
npm install     # links the three packages together; needs pkg/ to exist
npm run build   # tsc over utils/ and score/
```

`--target web` hands the wasm bytes to the standard `WebAssembly` API instead of importing the
`.wasm` as an ES module. Node still marks that import experimental — stability 1.1, one warning
per run — so this is the target that serves browsers, node and the single-file binary from one
artifact.

## Usage

The wasm has to be instantiated before the first call. In a browser, `init` fetches it from
beside the module:

```ts
import init, { Log, league_names } from "rustigc-wasm";

await init();
```

Then the API is fairly similar to the Rust one:

```ts
const log = new Log(igc);                       // Uint8Array of IGC

log.fix_count                                   // 25459
log.header("PLT")                               // { text: "Mike Young", origin: "flightrecorder" }
log.longest_flight()                            // { start: 125, stop: 25425 }
log.score("xcontest")                           // over the longest detected flight
log.describe("xcontest")                        // GeoJSON: detects, scores and draws

league_names()                                  // what score and describe accept
```

Documentation is in `pkg/rustigc.d.ts`, generated from the doc comments in
`src/lib.rs` along with interfaces for `Fix`, `Flight`, `Header` and `Score`.

GeoJSON crosses as a string rather than an object: a track would otherwise become as many arrays,
and a caller either prints it or hands it to a map. `JSON.parse` it for objects.

`describe` detects, scores and draws in one call. `export` draws the `flights`/`score` results
handed to it and searches for nothing itself — use it when you already have them.

## Bulk track access

`track` builds one object per fix through `serde_wasm_bindgen`; `track_bytes` hands over the
`#[repr(C)] Fix` array as a `Uint8Array` and leaves the decoding to the caller. Measured on
`fai-01`, 25 459 fixes:

| | time | memory |
|---|---|---|
| `track` — serde to objects | 10 ms | 3.40 MB heap |
| `track_bytes` — copy only | 0.3 ms | 0.78 MB, exactly the raw track |
| `track_bytes` + a `DataView` loop to the *same* objects | 1.05 ms | 2.5 MB heap |

Decoding the bytes yourself is **10x faster than serde for identical objects**. Both are a single
call — the difference is what happens underneath. `serde_wasm_bindgen` builds the array from inside
wasm and calls back out per fix: `Object::new` and an array append each time, against one buffer
copy for `track_bytes`.

`track` needs nothing of the caller; `track_bytes` is for a caller who will decode, and
`rustigc-utils` ships a decoder so that caller does not have to write one:

```ts
import { fixes } from "rustigc-utils";

log.track                // Fix[], straight from the binding
fixes(log.track_bytes)   // the same Fix[], 10x faster
```

The layout is 32 bytes per fix, little-endian, matching `Fix`:

| offset | 0 | 4 | 8 | 16 | 24 | 28 |
|---|---|---|---|---|---|---|
| | `u32` timestamp | *pad* | `f64` lat | `f64` lon | `i32` baro_alt | `i32` gnss_alt |

## Scoring without a log

`Scorer` scores a table of coordinates, so a track that never came from an IGC file scores the same
way. It takes one `Float64Array` of interleaved latitude and longitude, decimal degrees, in flight
order — that is `[f64; 2]`'s own layout, so the buffer crosses once and is adopted rather than
converted.

```ts
const scorer = new Scorer(Float64Array.of(45.00, 6.00, 45.05, 6.10, 45.20, 6.30));

scorer.solve("xcontest")   // Score | undefined; every fix an index into the table
```

The whole table is the window, and at least two pairs of coordinates are required.

## Local times

`Log.datetime` is UTC and `Log.tzn` is whatever offset the recorder declared.

`TZN` is the pilot's instrument setting and can simply be stale — a caller holding a real tz
dataset should shift `Log.datetime` itself instead.

`rustigc-utils/datetime` does that shift: `LocalTime.of(log)`, then `.at(timestamp)` for a
`{ date, time }` and `.zone` for how it reads.

## A friendlier `Log`

`rustigc-utils/log` extends the binding's `Log` rather than wrapping it, so everything it already
does is still there. It adds text input, `datetimeAt(timestamp)` and `zone`. Text is converted one
character to one byte.

## Tests

`npm test` runs `test/binding.test.mjs` on node's own test runner — no dependency, and it needs
`pkg/` built and `npm install` run. It checks that each entry point crosses the boundary and that
the results agree with the blessed corpus, including a `Scorer` table against the same window's
`Log.score`. The corpus itself is `rustigc`'s to sweep, not this package's.

## Example CLI

`score/` mirrors `rustigc-xc-score`, minus its `json` format, reading IGC on stdin. It is the
example and smoke test for these bindings, not a supported tool, and will not be published.
`cli.ts` holds the work; `rustigc-wasm-score.ts` and `sea.ts` differ only in where they get
the wasm, from beside the bindings or from a single executable's assets.

```sh
node rustigc-wasm/score/dist/rustigc-wasm-score.js --league xcontest --format human < flight.igc
```

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
