# rustigc-wasm-utils

Decoder for the raw fix array [`rustigc-wasm`](https://www.npmjs.com/package/rustigc-wasm) hands
over as `Log.track_bytes`.

Optional, and independent at runtime: it reads a `Uint8Array` and returns plain objects, so it
imports nothing from the bindings but the `Fix` type. Decoding here is about 10x faster than
having `serde_wasm_bindgen` build the same objects from inside wasm, which crosses the FFI once
per field where this crosses once per track.

```sh
npm install rustigc-wasm rustigc-wasm-utils
```

```ts
import { fixes } from "rustigc-wasm-utils";

log.track                // Fix[], straight from the binding
fixes(log.track_bytes)   // the same Fix[], 10x faster
```

The byte layout it decodes is documented with the binding that produces it, in the
[rustigc-wasm README](https://github.com/jbrun3t/rustigc/blob/main/rustigc/rustigc-wasm/README.md).

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
