#!/usr/bin/env node
// SPDX-License-Identifier: GPL-2.0-or-later
//
// The installed CLI, reading the wasm from beside the bindings.

import { readFileSync } from "node:fs";

import { main } from "./cli.js";

// Resolved from the package entry: the generated manifest's `exports` hide its subpaths, and the
// wasm sits beside the entry.
main(readFileSync(new URL("rustigc_wasm_bg.wasm", import.meta.resolve("rustigc-wasm"))));
