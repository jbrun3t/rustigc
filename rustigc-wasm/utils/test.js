import test from "node:test";
import assert from "node:assert/strict";
import { encode, fixes, STRIDE } from "./dist/track.js";

test("STRIDE is 32 bytes", () => {
  assert.equal(STRIDE, 32);
});

test("encode and fixes roundtrip", () => {
  const originalFixes = [
    {
      timestamp: 36000,
      lat: 45.1234567,
      lon: 5.9876543,
      baro_alt: 1500,
      gnss_alt: 1550,
    },
    {
      timestamp: 36001,
      lat: -12.3456789,
      lon: -45.6789012,
      baro_alt: -50,
      gnss_alt: -40,
    },
  ];

  const bytes = encode(originalFixes);
  assert.equal(bytes.byteLength, 2 * STRIDE);

  const decoded = fixes(bytes);
  assert.deepEqual(decoded, originalFixes);
});

test("encode handles undefined baro_alt and gnss_alt", () => {
  const fixesWithUndefined = [
    {
      timestamp: 100,
      lat: 45.0,
      lon: 6.0,
    },
    {
      timestamp: 101,
      lat: 45.1,
      lon: 6.1,
      baro_alt: undefined,
      gnss_alt: undefined,
    },
  ];

  const bytes = encode(fixesWithUndefined);
  const decoded = fixes(bytes);
  assert.equal(decoded[0].baro_alt, 0);
  assert.equal(decoded[0].gnss_alt, 0);
  assert.equal(decoded[1].baro_alt, 0);
  assert.equal(decoded[1].gnss_alt, 0);
});

