# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""The parsed IGC log and everything derived from it."""
from collections.abc import Iterable
from datetime import UTC, datetime, timedelta
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

import numpy

import rustigcpy._bindings as rib
from rustigcpy._bindings import FIX_DTYPE

from .fix import Fix
from .flight import Flight, Flights
from .score import Score
from .track import Track


def _handle(item: 'Flight | Score'):
    """The Rust-side layer behind a wrapper, which is what draws it"""
    handle = getattr(item, "_handle", None)
    if handle is None:
        raise TypeError(f"cannot draw a {type(item).__name__}")

    return handle


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

    Use `from_file` or `from_bytes` to build one. The track is copied into a numpy array on
    first `track` access and cached; reading it after that stays in Python.
    """

    def __init__(self, log: rib.RustLog):
        """Wrap an already-parsed log. Prefer `from_file` or `from_bytes`."""
        self._log = log
        self._track: Track | None = None
        self._flights: Flights | None = None

    def reset(self) -> None:
        """Drop the cached flight detection, so the next `flights()` runs it again."""
        self._flights = None

    def push(self, track: numpy.ndarray) -> None:
        """Replace the track, dropping everything cached from the old one.

        The track itself is read-only, so editing means working on a copy and pushing it back:

            edited = log.track._data.copy()
            edited["baro_altitude"] += 10
            log.push(edited)

        The fix count may change. A `Flight` or `Score` obtained before the push refers to the
        old track and can no longer be drawn against this log.

        Args:
            track: A `FIX_DTYPE` structured array, timestamps strictly increasing.

        Raises:
            TypeError: `track` is not a `FIX_DTYPE` array.
            ValueError: Its length is not a whole number of fixes, or its timestamps do not
                strictly increase.
        """
        if track.dtype != FIX_DTYPE:
            raise TypeError(f"track must be a {FIX_DTYPE} array, got {track.dtype}")

        self._log.set_track_bytes(track.tobytes())

        # Let the track come back up again from the Rust side
        self._track = None
        self.reset()

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

        # RFC 9557: the offset pins the instant, the bracket names the zone. `astimezone`, not
        # `replace`, so this attaches the zone rather than reinterpreting the wall clock against it.
        stamp, _, zone = origin.partition("[")
        stamped = datetime.fromisoformat(stamp)

        try:
            return stamped.astimezone(ZoneInfo(zone.rstrip("]")))
        except (ZoneInfoNotFoundError, ValueError):
            return stamped

    def datetime_at(self, timestamp: int) -> datetime | None:
        """When the fix carrying `timestamp` was recorded, as an aware datetime.

        Args:
            timestamp: A `Fix.timestamp`, in seconds.
        """
        origin = self.datetime
        if origin is None:
            return None

        return (origin.astimezone(UTC) + timedelta(seconds=timestamp)).astimezone(origin.tzinfo)

    def flights(self) -> Flights:
        """Flight sections detected in the track, cached until `reset()` or `push()`.

        Returns:
            A `Flights` list, empty when nothing was detected.
        """
        if self._flights is None:
            self._flights = Flights(
                Flight(self.track, handle) for handle in self._log.flights()
            )
        return self._flights

    def score(self, league: str,
              window: tuple[int, int] | tuple[Fix, Fix] | None = None) -> Score | None:
        """Score a window of the track against every rule of a league, reporting the best.

        Args:
            league: One of `league_names()`.
            window: The fixes to score, as a `(start, stop)` pair of indices or of `Fix`
                objects. Defaults to the longest detected flight.

        Returns:
            The best `Score`. None when the league is unknown, the window unusable, no flight
            was detected for the default window, or no rule could score.

        Raises:
            TypeError: One bound of `window` is a `Fix` and the other is not.
            ValueError: A `Fix` bound does not know its position in a track.
        """
        if window is None:
            flight = self.flights().longest
            if flight is None:
                return None
            start, stop = _window((flight.takeoff, flight.landing))
        else:
            start, stop = _window(window)

        handle = self._log.score(league, start, stop)
        if handle is None:
            return None
        return Score(self.track, handle)

    def describe(self, league: str) -> str:
        """Everything this log describes about itself, as a GeoJSON string.

        The longest flight detected in the track, and what it scored under `league`. Layers it
        cannot produce are left out: an unscorable league leaves just the track.

        Use `export` when the flight and score are already at hand.
        """
        return self._log.describe(league)

    def export(self, items: Iterable[Flight | Score] = (), track: bool = True) -> str:
        """This log and each of `items`, in the order given, as a GeoJSON string.

        Args:
            items: `Flight` and `Score` objects to draw, in drawing order.
            track: Whether to include the flown line.

        Returns:
            One GeoJSON FeatureCollection. Every feature declares a `role` — `track`, `marker`,
            `leg`, `closing`, `score` or `metadata`.

        Raises:
            TypeError: An item is neither a `Flight` nor a `Score`.

        Indices are taken on trust: an item detected or scored before a `push()` no longer
        refers to the track it came from.
        """
        return self._log.export([_handle(item) for item in items], track)

    def __repr__(self) -> str:
        return f"Log(fixes={len(self.track)}, pilot={self.pilot_name!r})"
