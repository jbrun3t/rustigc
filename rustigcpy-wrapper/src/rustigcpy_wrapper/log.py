"""Log wrapper - provides high-level API"""
import rustigcpy
from typing import Optional
from datetime import date, datetime
from .track import Track


class Log:
    """High-level IGC log wrapper

    Track data is copied into Python numpy array on first .track access.
    All subsequent operations are local Python (no FFI calls).
    """

    def __init__(self, raw_log: rustigcpy.Log):
        """Initialize from rustigcpy.Log instance"""
        self._log = raw_log
        self._track: Optional[Track] = None

    @classmethod
    def from_bytes(cls, content: bytes) -> 'Log':
        """Parse IGC file from bytes"""
        return cls(rustigcpy.Log.from_bytes(content))

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
    def pilot_name(self) -> Optional[str]:
        """Pilot name from headers"""
        return self._log.pilot_name()

    @property
    def glider_type(self) -> Optional[str]:
        """Glider type from headers"""
        return self._log.glider_type()

    @property
    def date(self) -> Optional[date]:
        """Parse flight date to datetime.date"""
        date_str = self._log.date()
        if date_str is None:
            return None
        # IGC format: DDMMYY,FF (flight number after comma)
        date_part = date_str.split(',')[0]
        return datetime.strptime(date_part, '%d%m%y').date()

    @property
    def takeoff(self) -> Optional[int]:
        """Takeoff fix index"""
        return self._log.takeoff

    @property
    def landing(self) -> Optional[int]:
        """Landing fix index"""
        return self._log.landing

    def __repr__(self) -> str:
        return f"Log(fixes={len(self.track)}, pilot={self.pilot_name!r})"
