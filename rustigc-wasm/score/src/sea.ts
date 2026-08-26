// SPDX-License-Identifier: GPL-2.0-or-later
//
// Entry for the single-executable build, where the wasm rides along as an asset.
//
// A single executable's `require` reaches built-in modules only, so nothing here may look at the
// file system. `getRawAsset` hands over the bundled bytes without copying them.

import { getRawAsset } from "node:sea";

import { main } from "./cli.js";

main(getRawAsset("wasm"));
