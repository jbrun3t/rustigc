#!/usr/bin/env node
// SPDX-License-Identifier: GPL-2.0-or-later
//
// Example CLI over the wasm bindings, mirroring `rustigc-xc-score`. Reads IGC on stdin.

"use strict";

const { readFileSync } = require("node:fs");
const { parseArgs } = require("node:util");

const { Log } = require("../pkg/rustigcjs.js");

/// One fix as `HH:MM:SS - [lat,lon] - @index`, dropping the time when the log has no date.
function disp(log, index) {
	const { lat, lon } = log.fix(index);
	const place = `[${lat.toFixed(4)},${lon.toFixed(4)}] - @${index}`;
	const when = log.fix_datetime(index);

	return when ? `${when.time} - ${place}` : place;
}

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
	const flight = log.longest_flight();

	if (!flight) {
		console.error("No flight detected");
		return;
	}

	const when = log.fix_datetime(flight.start);
	console.log(when ? `Flight on ${when.date} ${when.zone}` : "Flight has no date !");
	console.log(`Takeoff: ${disp(log, flight.start)}`);
	console.log(`Landing: ${disp(log, flight.stop)}`);
}

main()
