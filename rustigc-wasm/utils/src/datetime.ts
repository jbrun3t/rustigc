// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

/**
 * Local wall-clock time for a track, from the offset the log's `TZN` declares, or UTC. There is
 * no zone lookup.
 *
 * @module
 */

import type { Log } from "rustigc-wasm";

/** Milliseconds in an hour, the unit `TZN` is declared in. */
const HOUR_MS = 3_600_000;

/**
 * Formats a shifted instant as a UTC wall clock. Built on first use: the first `Intl` formatter
 * loads ICU's data, ~10 ms of node 24 start-up that a caller reading no local time should not pay.
 */
let wallClock: Intl.DateTimeFormat | undefined;

function wallClockFormat(): Intl.DateTimeFormat {
	wallClock ??= new Intl.DateTimeFormat("en-GB", {
		timeZone: "UTC",
		year: "numeric",
		month: "2-digit",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
		second: "2-digit",
		hourCycle: "h23",
	});

	return wallClock;
}

/** A calendar day and a wall clock, `2022-08-05` and `10:09:32`. */
export interface WallClock {
	date: string;
	time: string;
}

/** The instant a track's timestamps count from, read against the offset the log declares. */
export class LocalTime {
	readonly #origin: number;
	readonly #tzn: number | undefined;

	private constructor(origin: number, tzn: number | undefined) {
		this.#origin = origin;
		this.#tzn = tzn;
	}

	/** `undefined` when the log has no date. */
	static of(log: Log): LocalTime | undefined {
		const origin = log.datetime();

		return origin === undefined ? undefined : new LocalTime(Date.parse(origin), log.tzn());
	}

	/** Wall clock at `timestamp`, a `Fix.timestamp` in milliseconds. */
	at(timestamp: number): WallClock {
		const shifted = new Date(this.#origin + timestamp + (this.#tzn ?? 0) * HOUR_MS);
		const part: Record<string, string> = {};

		for (const { type, value } of wallClockFormat().formatToParts(shifted)) {
			part[type] = value;
		}

		return {
			date: `${part.year}-${part.month}-${part.day}`,
			time: `${part.hour}:${part.minute}:${part.second}`,
		};
	}

	/** How the shift reads, `UTC+1` or `UTC` when the log declares none. */
	get zone(): string {
		return this.#tzn === undefined ? "UTC" : `UTC${this.#tzn >= 0 ? "+" : ""}${this.#tzn}`;
	}
}
