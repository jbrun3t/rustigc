// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

/**
 * Local wall clock for a track.
 *
 * Mimic what Rust CLI and Python bindings do, minus tz lookup for now
 *
 * @module
 */

import type { Log } from "rustigc-wasm";

/** Milliseconds in an hour, the unit `TZN` is declared in. */
const HOUR_MS = 3_600_000;

/** Reads a shifted instant back out. */
const WALL_CLOCK = new Intl.DateTimeFormat("en-GB", {
	timeZone: "UTC",
	year: "numeric",
	month: "2-digit",
	day: "2-digit",
	hour: "2-digit",
	minute: "2-digit",
	second: "2-digit",
	hourCycle: "h23",
});

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

	/** `undefined` when the log states no date, so there is nothing to count from. */
	static of(log: Log): LocalTime | undefined {
		const origin = log.datetime();

		return origin === undefined ? undefined : new LocalTime(Date.parse(origin), log.tzn());
	}

	/** Wall clock at `timestamp`, a `Fix.timestamp` in milliseconds. */
	at(timestamp: number): WallClock {
		const shifted = new Date(this.#origin + timestamp + (this.#tzn ?? 0) * HOUR_MS);
		const part: Record<string, string> = {};

		for (const { type, value } of WALL_CLOCK.formatToParts(shifted)) {
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
