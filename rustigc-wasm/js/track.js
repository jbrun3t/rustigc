// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"use strict";

/**
 * Decoder for `Log.track_bytes`.
 *
 * Optional: the binding hands over the raw fix array and nothing obliges a caller to use this.
 * Doing it here rather than in Rust is 10x faster, because `serde_wasm_bindgen` crosses the FFI
 * once per field where this crosses once per track.
 *
 * @module track
 */

/**
 * Bytes per fix, matching `#[repr(C)] Fix`. See the crate README for the layout.
 *
 * @type {number}
 */
const STRIDE = 32;

/**
 * One position fix.
 *
 * @typedef {object} Fix
 * @property {number} timestamp Seconds from the instant `Log.datetime` reports.
 * @property {number} lat Latitude in decimal degrees, north positive.
 * @property {number} lon Longitude in decimal degrees, east positive.
 * @property {number} baro_alt Pressure altitude in meters.
 * @property {number} gnss_alt GNSS altitude in meters.
 */

/**
 * Decodes a whole track, one object per fix.
 *
 * @param {Uint8Array} bytes `Log.track_bytes`, a whole number of `STRIDE`-byte fixes.
 * @returns {Fix[]} The fixes, in track order.
 */
function fixes(bytes) {
	const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
	const count = bytes.byteLength / STRIDE;
	const out = new Array(count);

	for (let i = 0, at = 0; i < count; i++, at += STRIDE) {
		out[i] = {
			timestamp: dv.getUint32(at, true),
			lat: dv.getFloat64(at + 8, true),
			lon: dv.getFloat64(at + 16, true),
			baro_alt: dv.getInt32(at + 24, true),
			gnss_alt: dv.getInt32(at + 28, true),
		};
	}

	return out;
}

module.exports = { STRIDE, fixes };
