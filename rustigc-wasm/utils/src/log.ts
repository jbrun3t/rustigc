// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

/**
 * The binding's `Log`, taking text as well as bytes and reporting local times.
 *
 * @module
 */

import { Log as Binding } from "rustigc-wasm";

import { LocalTime, type WallClock } from "./datetime.js";

/** A string as bytes, one per character masked to 7 bits: valid IGC is ASCII (A6). */
function ascii(source: string | Uint8Array): Uint8Array {
	return typeof source === "string"
		? Uint8Array.from(source, (char) => char.charCodeAt(0) & 0x7f)
		: source;
}

/** A parsed IGC log. */
export class Log extends Binding {
	readonly #local: LocalTime | undefined;

	constructor(source: string | Uint8Array) {
		super(ascii(source));
		this.#local = LocalTime.of(this);
	}

	/** Wall clock at `timestamp`, shifted by the log's `TZN` offset; `undefined` without a date. */
	datetimeAt(timestamp: number): WallClock | undefined {
		return this.#local?.at(timestamp);
	}

	/** How those times read, `UTC+1` or `UTC` when the log declares no offset. */
	get zone(): string {
		return this.#local?.zone ?? "UTC";
	}
}
