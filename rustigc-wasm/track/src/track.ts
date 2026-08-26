// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

/**
 * Decoder for `Log.track_bytes`.
 *
 * Optional: the binding hands over the raw fix array and nothing obliges a caller to use this.
 * Doing it here rather than in Rust is 10x faster, because `serde_wasm_bindgen` crosses the FFI
 * once per field where this crosses once per track.
 *
 * @module
 */

import type { Fix } from "rustigc-wasm";

/** Bytes per fix, matching `#[repr(C)] Fix`. See the README for the layout. */
export const STRIDE = 32;

/**
 * Decodes a whole track, one object per fix, in track order.
 *
 * `bytes` is a whole number of `STRIDE`-byte fixes, as `Log.track_bytes` hands them over.
 */
export function fixes(bytes: Uint8Array): Fix[] {
	const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
	const count = bytes.byteLength / STRIDE;
	const out = new Array<Fix>(count);

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
