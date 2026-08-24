#!/usr/bin/env node
// SPDX-License-Identifier: GPL-2.0-or-later
//
// Example CLI over the wasm bindings, mirroring `rustigc-xc-score`. Reads IGC on stdin.

"use strict";

const { readFileSync } = require("node:fs");
const { parseArgs } = require("node:util");

const { Log, league_names } = require("../pkg/rustigcjs.js");

const OPTIONS = {
	league: { type: "string", default: "xcontest" },
	window: { type: "string" },
	help: { type: "boolean", short: "h" },
};

function usage() {
	console.error(
		[
			"Score IGC Files",
			"",
			"Usage: rustigc-js-score [OPTIONS]",
			"",
			"Options:",
			`      --league <LEAGUE>  Scoring league [default: xcontest] [possible values: ${league_names().join(", ")}]`,
			"      --window <WINDOW>  Explicit `start,stop` fix range to score, bypassing flight detection",
			"  -h, --help             Print help",
		].join("\n"),
	);
}

/// `start,stop` as the `{start, stop}` window the binding takes.
function parseWindow(text) {
	const cut = text.indexOf(",");
	if (cut < 0) {
		throw new Error('expected "start,stop"');
	}

	const parse = (raw, what) => {
		const value = Number(raw.trim());
		if (!Number.isInteger(value) || value < 0) {
			throw new Error(`invalid ${what}: ${raw}`);
		}
		return value;
	};

	return {
		start: parse(text.slice(0, cut), "start"),
		stop: parse(text.slice(cut + 1), "stop"),
	};
}

/// One fix as `HH:MM:SS - [lat,lon] - @index`, dropping the time when the log has no date.
function disp(log, index) {
	const { lat, lon } = log.fix(index);
	const place = `[${lat.toFixed(4)},${lon.toFixed(4)}] - @${index}`;
	const when = log.fix_datetime(index);

	return when ? `${when.time} - ${place}` : place;
}

function humanOutput(log, result) {
	const when = log.fix_datetime(result.entry);
	console.log(when ? `Flight on ${when.date} ${when.zone}` : "Flight has no date !");

	console.log(`Takeoff: ${disp(log, result.takeoff)}`);
	console.log(` Entry : ${disp(log, result.entry)}`);
	result.turnpoints.forEach((tp, i) => console.log(`  TP${i}  : ${disp(log, tp)}`));
	console.log(` Exit  : ${disp(log, result.exit)}`);
	console.log(`Landing: ${disp(log, result.landing)}`);

	let report = `${result.description} ${result.score} points, ${result.distance} km`;
	if (result.multiplier !== 1) {
		report += ` (×${result.multiplier})`;
	}
	if (result.circuit) {
		report += ` [ closing distance: ${result.gap} km ]`;
	}
	console.log(report);
}

function main() {
	let args;
	try {
		({ values: args } = parseArgs({ options: OPTIONS }));
		args.window = args.window === undefined ? undefined : parseWindow(args.window);
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
	const scored = log.score(args.league, args.window);

	if (!scored) {
		console.error("Could not score");
		return;
	}

	humanOutput(log, scored);
}

main()
