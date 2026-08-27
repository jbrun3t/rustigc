// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

/**
 * The binding's `Log`, taking text as well as bytes and reporting local times.
 *
 * @module
 */

import { Log as Binding } from "rustigc-wasm";

import { LocalTime, type WallClock } from "./datetime.js";

/**
 * IGC as bytes, one per character.
 *
 * A valid file is ASCII: printable `0x20`-`0x7D` less the reserved characters, plus the `CRLF`
 * ending (A6)
 */
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

	/** When the fix carrying `timestamp` was recorded, shifted by whatever `tzn` declares. */
	datetimeAt(timestamp: number): WallClock | undefined {
		return this.#local?.at(timestamp);
	}

	/** How those times read, `UTC+1` or `UTC` when the log declares no offset. */
	get zone(): string {
		return this.#local?.zone ?? "UTC";
	}
}
