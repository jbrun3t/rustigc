import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { initSync, Log, league_names } from "../pkg/rustigc.js";

initSync({
  module: readFileSync(new URL("../pkg/rustigc_bg.wasm", import.meta.url)),
});

const fixturePath = new URL("../../test_data/real/fai-01.igc", import.meta.url);

test("league_names returns expected leagues", () => {
  const names = league_names();
  assert.ok(names.includes("xcontest"));
  assert.ok(names.includes("cfd"));
});

test("Log constructor parses IGC fixture from Buffer", () => {
  const igcBuffer = readFileSync(fixturePath);
  const log = new Log(igcBuffer);

  assert.equal(log.fix_count, 25459);
  assert.ok(log.header("PLT"));
  const score = log.score("xcontest");
  assert.ok(score);
  assert.equal(score.description, "Closed FAI Triangle");
});

test("Log constructor parses IGC fixture from Uint8Array", () => {
  const igcBuffer = readFileSync(fixturePath);
  const uint8Array = new Uint8Array(igcBuffer.buffer, igcBuffer.byteOffset, igcBuffer.byteLength);
  const log = new Log(uint8Array);

  assert.equal(log.fix_count, 25459);
  assert.ok(log.header("PLT"));
  const score = log.score("xcontest");
  assert.ok(score);
  assert.equal(score.description, "Closed FAI Triangle");
});

test("Log constructor parses IGC fixture from ArrayBuffer", () => {
  const igcBuffer = readFileSync(fixturePath);
  const arrayBuffer = igcBuffer.buffer.slice(igcBuffer.byteOffset, igcBuffer.byteOffset + igcBuffer.byteLength);
  const log = new Log(arrayBuffer);

  assert.equal(log.fix_count, 25459);
  assert.ok(log.header("PLT"));
  const score = log.score("xcontest");
  assert.ok(score);
  assert.equal(score.description, "Closed FAI Triangle");
});

test("Log constructor parses IGC fixture from string", () => {
  const igcString = readFileSync(fixturePath, "utf-8");
  const log = new Log(igcString);

  assert.equal(log.fix_count, 25459);
  assert.ok(log.header("PLT"));
  assert.equal(log.header("PLT")?.text, "Mike Young");
  const score = log.score("xcontest");
  assert.ok(score);
  assert.equal(score.description, "Closed FAI Triangle");
});

test("Log constructor parses minimal IGC string", () => {
  const igc = "AXCT001\nHFDTE130826\nB1200004500000N00600000EA0100001200\n";
  const log = new Log(igc);

  assert.equal(log.fix_count, 1);
  assert.equal(log.fix(0).timestamp, 43200);
});

test("Log constructor throws on invalid IGC string", () => {
  assert.throws(
    () => new Log("not valid igc content"),
    (err) => err instanceof Error && err.message.includes("Failed to parse IGC file")
  );
});

test("Log constructor throws on invalid input type", () => {
  assert.throws(
    () => new Log(12345),
    (err) => err instanceof Error && err.message.includes("expected Uint8Array, Buffer, ArrayBuffer, or string")
  );
});

