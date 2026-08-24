s// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0
//
// Decoder for `Log.track_bytes`. Optional: the binding hands over the raw fix array and nothing
// obliges a caller to use this. Doing it here rather than in Rust is 10x faster because
// `serde_wasm_bindgen` crosses the FFI once per field.

"use strict";

/// Bytes per fix, matching `#[repr(C)] Fix`: u32 timestamp, 4 pad, f64 lat, f64 lon,
/// i32 baro_alt, i32 gnss_alt.
const STRIDE = 32;

/// One object per fix
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
