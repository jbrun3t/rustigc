# rustigc-wasm

WASM bindings for the rustigc IGC parser, as the `rustigcjs` JavaScript module.

## Prerequisites

The bindings are an optional part of the workspace: a plain `cargo build` does not need any of
this, and nothing else in the tree does either.

| | Debian package |
|---|---|
| `wasm32-unknown-unknown` target | `libstd-rust-dev-wasm32` |
| `wasm-pack` | `wasm-pack` |
| `wasm-bindgen` | `wasm-bindgen` |
| `wasm-opt` | `binaryen` |

`wasm-pack` drives the other three. It falls back to downloading its own `wasm-bindgen` when the
one on `PATH` does not match this crate's `wasm-bindgen` dependency — which is why that dependency
is pinned to an exact version in `Cargo.toml`. Bump the pin when the distro package moves.

## Build

```sh
wasm-pack build rustigc-wasm --target nodejs
```

Output lands in `pkg/`, `main` pointing at `rustigcjs.js`.

## Usage

```js
const { Log, league_names } = require("./pkg/rustigcjs.js");

const log = new Log(readFileSync("flight.igc"));   // Uint8Array

log.fix_count            // 25459
log.header_keys          // ["PLT", "GTY", "DTE", ...]
log.header("PLT")        // { text: "Mike Young", origin: "flightrecorder" }
log.fix(288)             // { timestamp, lat, lon, baro_alt, gnss_alt }
log.track                // one object per fix

log.flights()            // [ { start: 125, stop: 25425 } ]
log.longest_flight()     // { start: 125, stop: 25425 } | undefined

log.datetime()           // origin of the fix timestamps, see below
log.fix_datetime(288)    // { date: "2022-08-05", time: "10:14:20",
                         //   iso: "2022-08-05T10:14:20+01:00", zone: "Europe/London" }

log.score("xcontest")                    // longest detected flight
log.score("xcontest", { start, stop })   // an explicit window

log.describe("xcontest")                 // GeoJSON: detects, scores and draws
log.export(window, scored)               // GeoJSON: draws what you hand it
log.export(window, scored, false)        // ... without the flown line

league_names()           // ["cfd", "xcontest", "1tp", "2tp", "line", "oar"]
```

Values cross as plain JS data — there is no handle to keep alive and nothing to free. Field names
come from the core's serde derives, so they are `snake_case` (`baro_alt`, `raw_distance`) and the
method names follow them.

Instants cross already split. `jiff` has resolved the zone, and re-deriving the parts in JS would
mean `Date` — which rejects RFC 9557's `[Europe/London]` suffix outright — and then `Intl`,
resolving the zone a second time against ICU's timezone database instead of the one that produced
the value. `iso` is the field `new Date()` accepts.

GeoJSON has two paths, mirroring the core's. `describe` is for a caller that wants nothing but the
drawing and has nothing to hand over. `export` draws the `flights`/`score` results given back to it
and searches for nothing itself, so a caller that already scored is not charged twice — over a
25 000 fix track that is 5.8 ms of drawing against 14.1 ms of scoring. Either layer may be left out.

It crosses as a string rather than an object: that track would otherwise become as many JS arrays,
and a caller either prints it or hands it to a map. `JSON.parse` it for objects.

Nothing crosses as a Rust map: `serde_wasm_bindgen` renders one as a JS `Map`, which
`JSON.stringify` prints as `{}`. Hence `header`, one key at a time, rather than a whole `headers`
object.

## Example CLI

`js/rustigc-js-score.js` mirrors `rustigc-xc-score`, reading IGC on stdin. It is the example and
smoke test for these bindings, not a supported tool.

```sh
node rustigc-wasm/js/rustigc-js-score.js --league xcontest --format human < flight.igc
```

Its `human` and `geojson` output is diffed against `rustigc-xc-score` over the whole of
`test_data/real/` — 104 comparisons, byte for byte.

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
