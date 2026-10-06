# rustigc-wasm-utils

Optional helpers for [`rustigc-wasm`](https://www.npmjs.com/package/rustigc-wasm):

- the package root decodes `Log.track_bytes` into `Fix` objects, about 10x faster than
  `Log.track`;
- `rustigc-wasm-utils/datetime` reads timestamps as local times from the log's `TZN` offset;
- `rustigc-wasm-utils/log` extends `Log` with text input and those local times.

```sh
npm install rustigc-wasm rustigc-wasm-utils
```

```ts
import { fixes } from "rustigc-wasm-utils";

log.track                // Fix[], straight from the binding
fixes(log.track_bytes)   // the same Fix[], 10x faster
```

The byte layout it decodes is documented with the binding that produces it, in the
[rustigc-wasm README](https://github.com/jbrun3t/rustigc/blob/main/rustigc-wasm/README.md).

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
