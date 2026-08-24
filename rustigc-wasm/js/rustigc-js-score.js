#!/usr/bin/env node
// SPDX-License-Identifier: GPL-2.0-or-later
//
// Example CLI over the wasm bindings, mirroring `rustigc-xc-score`. Reads IGC on stdin.

"use strict";

const { readFileSync } = require("node:fs");
const { parseArgs } = require("node:util");

const { Log } = require("../pkg/rustigcjs.js");

function usage() {
	console.error(
		[
			"Score IGC Files",
			"",
			"Usage: rustigc-js-score [OPTIONS]",
			"",
			"Options:",
			"  -h, --help  Print help",
		].join("\n"),
	);
}

function main() {
	let args;
	try {
		({ values: args } = parseArgs({ options: { help: { type: "boolean", short: "h" } } }));
	} catch (e) {
		console.error(e.message);
		usage();
		process.exit(2);
	}

	if (args.help) {
		usage();
		return;
	}

	// `/dev/stdin` rather than fd 0: reading the fd throws EAGAIN on a pipe node has not drained.
	const content = readFileSync("/dev/stdin");
	if (content.length === 0) {
		console.error("No input on stdin");
		process.exit(0);
	}

	const log = new Log(content);

	console.log(`Fixes: ${log.fix_count}`);
	for (const key of log.header_keys.sort()) {
		console.log(`${key}: ${log.header(key).text}`);
	}
}

main()
