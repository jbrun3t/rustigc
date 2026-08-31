# Standalone binary notes

Node's [single executable applications](https://nodejs.org/api/single-executable-applications.html)
turn the CLI into one file that needs no node, no `pkg/`, and no `node_modules`. Two prerequisites
beyond the build ones:

| | |
|---|---|
| `postject` | injects the blob into the executable. |
| `esbuild` | a SEA main must be one file. |

The node this builds on must be an **official tarball**, not Debian's: Debian ships a 26 KB
`/usr/bin/node` stub against `libnode.so`, and SEA needs the fuse sentinel and the blob in the
*same* file.

`score/src/sea.ts` is the entry for this build. It takes the wasm from the SEA asset where
`rustigc-wasm-score.ts` reads it from disk, and both hand it to the same `cli.ts`.

```sh
npm install postject esbuild

# Both the blob and the binary it goes into must come from the same official node: the blob is
# version-specific, and `command -v node` here finds Debian's stub
NODE=./node-v24.19.0-linux-x64/bin/node

# Create one bundle file
esbuild score/src/sea.ts --bundle --platform=node --format=cjs --minify --outfile=bundle.js

# Add the wasm lib to the configuration
cat > sea-config.json <<'JSON'
{ "main": "bundle.js", "output": "sea.blob",
  "assets": { "wasm": "pkg/rustigc_wasm_bg.wasm" },
  "disableExperimentalSEAWarning": true }
JSON
"$NODE" --experimental-sea-config sea-config.json

# Copy node and inject the blob — postject warns about a section name string offset, expected
cp "$NODE" rustigc-wasm-score
postject rustigc-wasm-score NODE_SEA_BLOB sea.blob \
    --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2
```

The sources stay ESM; esbuild emits CommonJS because a SEA main must be one until node 26, whose
`"mainFormat": "module"` accepts an ESM main. `sea.ts` reads the asset with `getRawAsset`, which
hands over the bundled bytes themselves — `getAsset` would copy the megabyte first.

The result is ~127 MB, essentially all of it node: the blob is 1.1 MB and 1.1 MB of *that* is the
wasm. Its output matches `rustigc-xc-score` over `rustigc-test-data/real/`, the same 104 comparisons the
CLI itself is held to.

The same recipe builds a comparable `igc-xc-score` binary from
`igc-xc-score/dist/igc-xc-score.cjs`, minus the wasm asset. Worth having: the shipped
`igc-xc-score-linux` v1.8.0 embeds node 12, so timing it against rustigc compares runtimes as much
as algorithms.
