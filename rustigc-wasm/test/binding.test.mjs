// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0
//
// Sanity test for the wasm bindings: that every entry point crosses the boundary and agrees with
// the blessed corpus. Not a corpus sweep — `rustigc`'s own tests own that.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { initSync, league_names, Log, Scorer } from "rustigc-wasm";
import { fixes, STRIDE } from "rustigc-utils";

initSync({ module: readFileSync(new URL("rustigc_bg.wasm", import.meta.resolve("rustigc-wasm"))) });

const CORPUS = new URL("../../test_data/real/", import.meta.url);

/** A blessed fixture, with the score `rustigc/tests/scoring.rs` pins for it. */
const FIXTURES = {
	"triangle-01": { description: "closed free triangle", score: 22.08, distance_km: 15.77 },
	// The same flight moved onto ±180, so it must score the same.
	"problem-antimeridian": { description: "closed free triangle", score: 26.8, distance_km: 19.14 },
};

function load(name) {
	return new Log(readFileSync(new URL(`${name}.igc`, CORPUS)));
}

/** The window's fixes as the interleaved [lat, lon] table `Scorer` takes. */
function table(log, { start, stop }) {
	const track = fixes(log.track_bytes).slice(start, stop + 1);
	const coords = new Float64Array(track.length * 2);

	track.forEach(({ lat, lon }, i) => {
		coords[2 * i] = lat;
		coords[2 * i + 1] = lon;
	});

	return coords;
}

test("league_names lists the real leagues", () => {
	const names = league_names();

	assert.ok(names.includes("xcontest"), `no xcontest in ${names}`);
	assert.ok(names.includes("cfd"), `no cfd in ${names}`);
});

test("Log parses a fixture", () => {
	const log = load("triangle-01");

	assert.equal(log.fix_count, 4647);
	assert.equal(log.header("PLT").origin, "flightrecorder");
	assert.match(log.datetime(), /^\d{4}-\d{2}-\d{2}T00:00:00/);

	const { start, stop } = log.longest_flight();
	assert.ok(start < stop && stop < log.fix_count, `window ${start}..${stop}`);
});

test("track_bytes decodes to what track builds", () => {
	const log = load("triangle-01");
	const bytes = log.track_bytes;

	assert.equal(bytes.length, log.fix_count * STRIDE);
	assert.deepEqual(fixes(bytes), log.track);
});

test("Log.score matches the blessed corpus", () => {
	for (const [name, want] of Object.entries(FIXTURES)) {
		const scored = load(name).score("xcontest");

		assert.equal(scored.description, want.description, name);
		assert.equal(scored.score, want.score, name);
		assert.equal(scored.distance_km, want.distance_km, name);
	}
});

test("Scorer over a coordinate table matches Log.score", () => {
	for (const name of Object.keys(FIXTURES)) {
		const log = load(name);
		const window = log.longest_flight();
		const expected = log.score("xcontest", window);

		const scored = new Scorer(table(log, window)).solve("xcontest");

		// A Scorer's window is the whole table, so its indices start at the window.
		assert.deepEqual(scored, {
			...expected,
			takeoff: expected.takeoff - window.start,
			entry: expected.entry - window.start,
			turnpoints: expected.turnpoints.map((tp) => tp - window.start),
			exit: expected.exit - window.start,
			landing: expected.landing - window.start,
		}, name);
	}
});

test("Scorer rejects an unusable table", () => {
	assert.throws(() => new Scorer(Float64Array.of(45.0, 6.0, 45.1)), /whole number/);
	assert.throws(() => new Scorer(Float64Array.of(45.0, 6.0)), /not scorable/);
	assert.throws(() => new Scorer(Float64Array.of(45.0, 6.0, 90.5, 6.1)), /not scorable/);
	assert.throws(() => new Scorer(Float64Array.of(45.0, 6.0, NaN, 6.1)), /not scorable/);
});

test("a layer the track does not hold throws", () => {
	const log = load("triangle-01");
	const window = log.longest_flight();

	assert.ok(log.export(window).length > 0);
	assert.throws(() => log.export({ start: 0, stop: log.fix_count }), /out of range/);
	assert.throws(() => log.export({ nope: 1 }), /missing field/);
});

test("an unknown league throws", () => {
	const log = load("triangle-01");

	assert.throws(() => log.score("nope"), /unknown league/);
	assert.throws(() => log.describe("nope"), /unknown league/);

	const scorer = new Scorer(Float64Array.of(45.0, 6.0, 45.1, 6.1));
	assert.throws(() => scorer.solve("nope"), /unknown league/);
});
