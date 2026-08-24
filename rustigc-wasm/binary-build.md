# Standalone binary notes

Node's [single executable applications](https://nodejs.org/api/single-executable-applications.html)
turn the CLI into one file that needs no node, no `pkg/`, and no `node_modules`. Two prerequisites
beyond the build ones:

| | |
|---|---|
| `postject` | injects the blob into the executable. `objcopy` cannot stand in: the section it appends is placed wrong and node segfaults reading it |
| `esbuild` | a SEA main must be one file, and the CLI pulls in the wasm-bindgen glue |

The host node must be an **official tarball**, not Debian's. Debian ships a 26 KB `/usr/bin/node`
stub against `libnode.so`, and SEA needs the fuse sentinel and the blob section in the *same* file.

```sh
npm install postject esbuild

# 1. one file. --minify is optional; without it the patch below matches on exact text
esbuild js/rustigc-js-score.js --bundle --platform=node --format=cjs --minify --outfile=bundle.js

# 2. the glue reads the wasm off disk — repoint it at the SEA asset. This is a patch on generated
#    code: it matches `${__dirname}/rustigcjs_bg.wasm` and the readFileSync beside it.
#      var X = Buffer.from(require("node:sea").getAsset("wasm"));

# 3. blob, with the wasm as an asset
cat > sea-config.json <<'JSON'
{ "main": "bundle.js", "output": "sea.blob",
  "assets": { "wasm": "pkg/rustigcjs_bg.wasm" },
  "disableExperimentalSEAWarning": true }
JSON
node --experimental-sea-config sea-config.json

# 4. strip BEFORE injecting — strip would drop the NODE_SEA_BLOB section afterwards
cp "$(command -v node)" rustigc-js-score && chmod +w rustigc-js-score && strip rustigc-js-score
postject rustigc-js-score NODE_SEA_BLOB sea.blob \
    --sentinel-fuse NODE_SEA_FUSE_fce680ab2cc467b6e072b8b5df1996b2
```

The result is ~109 MB, essentially all of it node: the blob is 1.1 MB and 1.1 MB of *that* is the
wasm.

The same four steps build a comparable `igc-xc-score` binary from
`igc-xc-score/dist/igc-xc-score.cjs`, minus step 2 since it has no wasm. Worth having: the shipped
`igc-xc-score-linux` v1.8.0  embeds node 12, so timing it against rustigc compares runtimes as much as
algorithms.
