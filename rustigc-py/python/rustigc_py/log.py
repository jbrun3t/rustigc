# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""The parsed IGC log and everything derived from it."""
from __future__ import annotations

import json
from datetime import UTC, datetime, timedelta, timezone, tzinfo
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

_TIMEZONE_FINDER = None


def _finder():
    """The `timezonefinder` instance, built on first use and kept."""
    global _TIMEZONE_FINDER

    if _TIMEZONE_FINDER is None:
        from timezonefinder import TimezoneFinder

        _TIMEZONE_FINDER = TimezoneFinder(in_memory=True)

    return _TIMEZONE_FINDER


import numpy

import rustigc_py._bindings as rib
from rustigc_py._bindings import FIX_DTYPE

from .fix import Fix
from .flight import Flight, Flights
from .score import Score
from .track import Track


def _json(item) -> str | None:
    """A layer's scalars, for Rust to read back into the struct that draws it"""
    return None if item is None else json.dumps(item._data)


def _window(bounds: tuple[int, int] | tuple[Fix, Fix]) -> tuple[int, int]:
    """Fix indices of a scoring window, given as indices or as fixes"""
    start, stop = bounds

    if isinstance(start, Fix) != isinstance(stop, Fix):
        raise TypeError("window bounds must both be indices or both be fixes")
    if not isinstance(start, Fix):
        return start, stop
    if start.index is None or stop.index is None:
        raise ValueError("fix has no position in a track")

    return start.index, stop.index


class Log:
    """A parsed IGC file.

    A log never changes, so the track is copied into a numpy array on first `track` access
    and kept; reading it after that stays in Python.
    """

    def __init__(self, log: rib.RustLog):
        """Wrap an already-parsed log. Prefer `from_file` or `from_bytes`."""
        self._log = log
        self._track: Track | None = None

    def with_track(self, track: numpy.ndarray) -> 'Log':
        """A new log over `track`, carrying everything else this one holds.

        The track is read-only, so editing means working on a copy:

            edited = log.track._data.copy()
            edited["baro_altitude"] += 10
            log = log.with_track(edited)

        Args:
            track: A `FIX_DTYPE` structured array, timestamps strictly increasing.

        Raises:
            TypeError: `track` is not a `FIX_DTYPE` array.
            ValueError: Its length is not a whole number of fixes, or its timestamps do not
                strictly increase.
        """
        if track.dtype != FIX_DTYPE:
            raise TypeError(f"track must be a {FIX_DTYPE} array, got {track.dtype}")

        return Log(self._log.with_track_bytes(track.tobytes()))

    @classmethod
    def from_bytes(cls, content: bytes) -> 'Log':
        """Parse an IGC file held in memory.

        Args:
            content: The file, as bytes.

        Raises:
            ValueError: The bytes are not usable IGC.
        """
        return cls(rib.RustLog.from_bytes(content))

    @classmethod
    def from_file(cls, path: str) -> 'Log':
        """Parse the IGC file at `path`.

        Raises:
            OSError: The file could not be read.
            ValueError: Its content is not usable IGC.
        """
        with open(path, 'rb') as f:
            return cls.from_bytes(f.read())

    @property
    def track(self) -> Track:
        """The position fixes, copied once on first access then cached."""
        if self._track is None:
            self._track = Track(self._log.track_bytes)
        return self._track

    @property
    def pilot_name(self) -> str | None:
        """Pilot name, from the `PLT` header. None when entry is not present."""
        header = self._log.get_header("PLT")
        return header[0] if header else None

    @property
    def glider_type(self) -> str | None:
        """Glider type, from the `GTY` header. None when entry is not present."""
        header = self._log.get_header("GTY")
        return header[0] if header else None

    @property
    def datetime(self) -> datetime | None:
        """Instant this log's fix timestamps count from, as an aware datetime.

        UTC midnight of the flight's date, read in the zone the track starts in. West of
        Greenwich that puts its own local date on the day before the flight's. `datetime_at`
        dates a fix from it. None without a usable `HFDTE` header.
        """
        origin = self._log.datetime()
        if origin is None:
            return None

        stamped = datetime.fromisoformat(origin)
        zone = self._zone()

        return stamped.astimezone(zone) if zone else stamped

    def _zone(self) -> tzinfo | None:
        """Zone the track starts in, falling back to the offset the log declares.

        None leaves the origin in UTC.
        """
        track = self.track
        if len(track) > 0:
            first = track[0]
            name = _finder().timezone_at(lng=first.longitude, lat=first.latitude)

            if name is not None:
                try:
                    return ZoneInfo(name)
                except (ZoneInfoNotFoundError, ValueError):
                    pass

        tzn = self._log.tzn()

        return timezone(timedelta(hours=tzn)) if tzn is not None else None

    def datetime_at(self, timestamp: int) -> datetime | None:
        """When the fix carrying `timestamp` was recorded, as an aware datetime.

        Args:
            timestamp: A `Fix.timestamp`, in milliseconds.
        """
        origin = self.datetime
        if origin is None:
            return None

        return (origin.astimezone(UTC) + timedelta(milliseconds=timestamp)).astimezone(origin.tzinfo)

    def flights(self) -> Flights:
        """Detect the flight sections in the track.

        Each call detects again; hold the result to reuse it.

        Returns:
            A `Flights` list, empty when nothing was detected.
        """
        return Flights(
            Flight(self.track, data) for data in json.loads(self._log.flights())
        )

    def score(self, league: str,
              window: tuple[int, int] | tuple[Fix, Fix] | None = None) -> Score | None:
        """Score a window of the track against every rule of a league, reporting the best.

        Args:
            league: One of `league_names()`.
            window: The fixes to score, as a `(start, stop)` pair of indices or of `Fix`
                objects. Defaults to the longest detected flight.

        Returns:
            The best `Score`. None when no rule could score.

        Raises:
            TypeError: One bound of `window` is a `Fix` and the other is not.
            ValueError: `league` is not one of `league_names()`, the window is not one this
                track holds — including a default window with no flight to take it from — or a
                `Fix` bound does not know its position in a track.
        """
        if window is None:
            flight = self.flights().longest
            if flight is None:
                raise ValueError("no flight detected, pass an explicit window")
            start, stop = _window((flight.takeoff, flight.landing))
        else:
            start, stop = _window(window)

        scored = self._log.score(league, start, stop)
        if scored is None:
            return None
        return Score(self.track, json.loads(scored))

    def describe(self, league: str) -> str:
        """Everything this log describes about itself, as a GeoJSON string.

        The longest flight detected in the track, and what it scored under `league`. A flight
        that scores nothing leaves just the track.

        Use `export` when the flight and score are already at hand.

        Raises:
            ValueError: `league` is not one of `league_names()`.
        """
        return self._log.describe(league)

    def export(self, flight: Flight | None = None, score: Score | None = None,
               track: bool = True) -> str:
        """This log, `flight` and `score`, as a GeoJSON string.

        Args:
            flight: A detected section to draw, or None.
            score: A scored task to draw, or None.
            track: Whether to include the flown line.

        Returns:
            One GeoJSON FeatureCollection. Every feature declares a `role` — `track`, `marker`,
            `leg`, `closing`, `score` or `metadata`.

        Raises:
            ValueError: A layer is not the kind its slot draws. Rust reads each one back and
                names the field it is missing.
            TypeError: A layer is not a layer at all.

        Indices are taken on trust: a layer detected or scored in another log — what `with_track`
        hands back, for one — no longer refers to the track it came from.
        """
        return self._log.export(_json(flight), _json(score), track)

    def __repr__(self) -> str:
        return f"Log(fixes={len(self.track)}, pilot={self.pilot_name!r})"
