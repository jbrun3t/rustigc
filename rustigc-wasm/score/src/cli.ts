// SPDX-License-Identifier: GPL-2.0-or-later
//
// Example CLI over the wasm bindings, mirroring `rustigc-xc-score`. Reads IGC on stdin.

import { readFileSync } from "node:fs";
import { parseArgs } from "node:util";

import {
	initSync,
	preset_names,
	Ruleset,
	type Flight,
	type Score,
	type SyncInitInput,
} from "rustigc-wasm";
import { Log } from "rustigc-wasm-utils/log";

const FORMATS = ["human", "json", "geojson"];

const OPTIONS = {
	ruleset: { type: "string" },
	"ruleset-file": { type: "string" },
	format: { type: "string", default: "human" },
	window: { type: "string" },
	help: { type: "boolean", short: "h" },
} as const;

function usage(): void {
	console.error(
		[
			"Score IGC Files",
			"",
			"Usage: rustigc-wasm-score [OPTIONS]",
			"",
			"Options:",
			`      --ruleset <RULESET>    Scoring ruleset [default: xcontest] [possible values: ${preset_names().join(", ")}]`,
			"      --ruleset-file <PATH>  Scoring ruleset read from a JSON file, instead of --ruleset",
			`      --format <FORMAT>      Output format [default: human] [possible values: ${FORMATS.join(", ")}]`,
			"      --window <WINDOW>      Explicit `start,stop` fix range to score, bypassing flight detection",
			"  -h, --help                 Print help",
		].join("\n"),
	);
}

/// `start,stop` as the `{start, stop}` window the binding takes.
function parseWindow(text: string): Flight {
	const cut = text.indexOf(",");
	if (cut < 0) {
		throw new Error('expected "start,stop"');
	}

	const parse = (raw: string, what: string): number => {
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
function disp(log: Log, index: number): string {
	const { lat, lon, timestamp } = log.fix(index);
	const place = `[${lat.toFixed(4)},${lon.toFixed(4)}] - @${index}`;
	const when = log.datetimeAt(timestamp);

	return when ? `${when.time} - ${place}` : place;
}

function humanOutput(log: Log, result: Score): void {
	const when = log.datetimeAt(log.fix(result.entry).timestamp);
	console.log(when ? `Flight on ${when.date} ${log.zone}` : "Flight has no date !");

	console.log(`Takeoff: ${disp(log, result.takeoff)}`);
	console.log(` Entry : ${disp(log, result.entry)}`);
	result.turnpoints.forEach((tp, i) => console.log(`  TP${i}  : ${disp(log, tp)}`));
	console.log(` Exit  : ${disp(log, result.exit)}`);
	console.log(`Landing: ${disp(log, result.landing)}`);

	let report = `${result.description} ${result.score} points, ${result.distance_km} km`;
	if (result.multiplier !== 1) {
		report += ` (×${result.multiplier})`;
	}
	if (result.circuit) {
		const max = Math.round(result.threshold_m);
		const gap = Math.round(result.gap_m);
		report += ` [ closing distance: ${gap} / ${max} m ]`;
	}
	console.log(report);
}

/// Runs the CLI over `wasm`, the bindings' WebAssembly, which each entry point sources its own way.
export function main(wasm: SyncInitInput): void {
	// First: `usage` lists the rulesets, which calls into the wasm.
	initSync({ module: wasm });

	let preset: string | undefined;
	let rulesetFile: string | undefined;
	let format: string;
	let requested: Flight | undefined;
	let help: boolean | undefined;

	try {
		const { values } = parseArgs({ options: OPTIONS });
		({ ruleset: preset, "ruleset-file": rulesetFile, format, help } = values);
		if (preset !== undefined && rulesetFile !== undefined) {
			throw new Error("--ruleset cannot be used with --ruleset-file");
		}
		requested = values.window === undefined ? undefined : parseWindow(values.window);
		if (!FORMATS.includes(format)) {
			throw new Error(`invalid format: ${format}`);
		}
	} catch (e) {
		console.error((e as Error).message);
		usage();
		process.exit(2);
	}

	if (help) {
		usage();
		return;
	}

	// `/dev/stdin` rather than fd 0: reading the fd throws EAGAIN on a pipe node has not drained.
	const content = readFileSync("/dev/stdin");
	if (content.length === 0) {
		console.error("No input on stdin");
		process.exit(0);
	}

	let log: Log;
	try {
		log = new Log(content);
	} catch (error) {
		console.error(error instanceof Error ? error.message : `Could not parse: ${error}`);
		process.exit(1);
	}

	let ruleset: Ruleset;
	try {
		ruleset = rulesetFile === undefined
			? Ruleset.preset(preset ?? "xcontest")
			: Ruleset.from_json(readFileSync(rulesetFile, "utf8"));
	} catch (error) {
		console.error(`Could not read ruleset: ${error instanceof Error ? error.message : error}`);
		process.exit(1);
	}

	const window = requested ?? log.longest_flight();
	const scored = log.score(ruleset, window);

	if (!scored) {
		console.error("Could not score");
	}

	if (format === "json") {
		// The scoring report alone, `null` when nothing scored, as `rustigc-xc-score` prints it.
		console.log(JSON.stringify(scored ?? null, null, 2));
	} else if (format === "geojson") {
		console.log(log.export(window, scored));
	} else if (scored) {
		humanOutput(log, scored);
	}
}
