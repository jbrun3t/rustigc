#!/usr/bin/env node
// SPDX-License-Identifier: GPL-2.0-or-later
//
// The installed CLI, reading the wasm from beside the bindings.

import { readFileSync } from "node:fs";

import { main } from "./cli.js";

// `import.meta.resolve` names the package entry, never a subpath: a package that declares
// `exports` encapsulates its subpaths, and the bindings' manifest is generated, not ours to hold
// still. The wasm sits beside that entry either way.
main(readFileSync(new URL("rustigc_bg.wasm", import.meta.resolve("rustigc-wasm"))));
