"""Log wrapper - provides high-level API"""
from datetime import date, datetime
from typing import TYPE_CHECKING, Optional

import rustigcpy._bindings as rib

from .track import Track

if TYPE_CHECKING:
    from .fix import Fix


class Log:
    """High-level IGC log wrapper

    Track data is copied into Python numpy array on first .track access.
    All subsequent operations are local Python (no FFI calls).
    """

    def __init__(self, log: rib.RustLog):
        """Initialize from rustigcpy.Log instance"""
        self._log = log
        self._track: Track | None = None

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
    def date(self) -> date | None:
        """Parse flight date to datetime.date"""
        header = self._log.get_header("DTE")
        if header is None:
            return None
        date_str = header[0]
        # IGC format: DDMMYY,FF (flight number after comma)
        date_part = date_str.split(',')[0]
        return datetime.strptime(date_part, '%d%m%y').date()

    def analyze(self) -> None:
        """Run flight phase analysis"""
        self._log.analyze()

    @property
    def takeoff(self) -> Optional['Fix']:
        """Takeoff fix"""
        idx = self._log.takeoff
        if idx is None:
            return None
        return self.track[idx]

    @property
    def landing(self) -> Optional['Fix']:
        """Landing fix"""
        idx = self._log.landing
        if idx is None:
            return None
        return self.track[idx]

    def __repr__(self) -> str:
        return f"Log(fixes={len(self.track)}, pilot={self.pilot_name!r})"
