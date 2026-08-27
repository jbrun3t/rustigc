import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { initSync, Log, league_names } from "../pkg/rustigc.js";
import { encode, fixes as decodeFixes } from "../utils/dist/track.js";

initSync({
  module: readFileSync(new URL("../pkg/rustigc_bg.wasm", import.meta.url)),
});

const fixturePath = new URL("../../test_data/real/fai-01.igc", import.meta.url);

test("league_names returns expected leagues", () => {
  const names = league_names();
  assert.ok(names.includes("xcontest"));
  assert.ok(names.includes("cfd"));
});

test("Log constructor parses IGC fixture", () => {
  const igcBuffer = readFileSync(fixturePath);
  const log = new Log(igcBuffer);

  assert.equal(log.fix_count, 25459);
  assert.ok(log.header("PLT"));
  const score = log.score("xcontest");
  assert.ok(score);
  assert.equal(score.description, "Closed FAI Triangle");
});

test("Log.from_track constructs log from JS fixes array", () => {
  const igcBuffer = readFileSync(fixturePath);
  const parsedLog = new Log(igcBuffer);
  const trackFixes = decodeFixes(parsedLog.track_bytes);

  const log = Log.from_track(trackFixes);
  assert.equal(log.fix_count, 25459);

  const score = log.score("xcontest");
  assert.ok(score);
  assert.equal(score.description, "Closed FAI Triangle");
});

test("Log.from_track_bytes constructs log from encoded track bytes", () => {
  const igcBuffer = readFileSync(fixturePath);
  const parsedLog = new Log(igcBuffer);

  const log = Log.from_track_bytes(parsedLog.track_bytes);
  assert.equal(log.fix_count, 25459);

  const score = log.score("xcontest");
  assert.ok(score);
  assert.equal(score.description, "Closed FAI Triangle");
});

test("Log.from_track supports undefined baro_alt and gnss_alt", () => {
  const fixes = [
    { timestamp: 100, lat: 45.0, lon: 6.0 },
    { timestamp: 101, lat: 45.1, lon: 6.1, baro_alt: undefined, gnss_alt: undefined },
    { timestamp: 102, lat: 45.2, lon: 6.2, baro_alt: 1200 },
  ];

  const log = Log.from_track(fixes);
  assert.equal(log.fix_count, 3);
  assert.equal(log.fix(0).baro_alt, 0);
  assert.equal(log.fix(0).gnss_alt, 0);
  assert.equal(log.fix(1).baro_alt, 0);
  assert.equal(log.fix(1).gnss_alt, 0);
  assert.equal(log.fix(2).baro_alt, 1200);
  assert.equal(log.fix(2).gnss_alt, 0);
});

