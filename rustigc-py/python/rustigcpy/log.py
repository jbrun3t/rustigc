# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Log wrapper - provides high-level API"""
import json
from datetime import UTC, datetime

import rustigcpy._bindings as rib

from .fix import Fix
from .flight import Flight, Flights
from .score import Score
from .track import Track


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
    """High-level IGC log wrapper

    Track data is copied into Python numpy array on first .track access.
    All subsequent operations are local Python (no FFI calls).
    """

    def __init__(self, log: rib.RustLog):
        """Initialize from rustigcpy.Log instance"""
        self._log = log
        self._track: Track | None = None
        self._flights: Flights | None = None

    def reset(self) -> None:
        """Drop the cached detection, so the next call runs it again"""
        self._flights = None

    @classmethod
    def from_bytes(cls, content: bytes) -> 'Log':
        """Parse IGC file from bytes"""
        return cls(rib.RustLog.from_bytes(content))

    @classmethod
    def from_file(cls, path: str) -> 'Log':
        """Parse IGC file from path"""
        with open(path, 'rb') as f:
            return cls.from_bytes(f.read())

    @property
    def track(self) -> Track:
        """Get track """
        # copied once on first access, then cached
        # This avoid going over the FFI for each fix which
        # would be slow
        if self._track is None:
            self._track = Track(self._log.track_bytes)
        return self._track

    @property
    def pilot_name(self) -> str | None:
        """Pilot name from headers"""
        header = self._log.get_header("PLT")
        return header[0] if header else None

    @property
    def glider_type(self) -> str | None:
        """Glider type from headers"""
        header = self._log.get_header("GTY")
        return header[0] if header else None

    @property
    def datetime(self) -> datetime | None:
        """Parse flight date to datetime"""
        header = self._log.get_header("DTE")
        if header is None:
            return None
        date_str = header[0]
        # IGC format: DDMMYY,FF (flight number after comma)
        date_part = date_str.split(',')[0]
        return datetime.strptime(date_part, '%d%m%y').replace(tzinfo=UTC)

    def flights(self) -> Flights:
        """Flight sections detected in the track, cached until `reset()`"""
        if self._flights is None:
            self._flights = Flights(
                Flight(self.track, data) for data in json.loads(self._log.flights())
            )
        return self._flights

    def score(self, league: str,
              window: tuple[int, int] | tuple[Fix, Fix] | None = None) -> Score | None:
        """Score against a `league`"""
        if window is None:
            flight = self.flights().longest
            if flight is None:
                return None
            start, stop = _window((flight.takeoff, flight.landing))
        else:
            start, stop = _window(window)

        raw = self._log.score(league, start, stop)
        if raw is None:
            return None
        return Score(self.track, json.loads(raw))

    def __repr__(self) -> str:
        return f"Log(fixes={len(self.track)}, pilot={self.pilot_name!r})"
