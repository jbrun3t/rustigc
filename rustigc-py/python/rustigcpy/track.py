"""Track wrapper - copies data on creation, all access is local Python"""
import numpy

from rustigcpy._bindings import FIX_DTYPE

from .fix import Fix


class Track:
    """Track with numpy array (copied once on init, all access local)

    The track data is copied from Rust into a Python numpy array on creation.
    All subsequent operations are local Python (no FFI calls).
    """

    def __init__(self, track_bytes: bytes):
        """Copy track data from Rust bytes into Python numpy array"""
        # Single copy happens HERE - everything after is local Python
        self._npdata = numpy.frombuffer(track_bytes, dtype=FIX_DTYPE)

    @property
    def _data(self) -> numpy.ndarray:
        """Full structured array"""
        return self._npdata

    @property
    def _latitude(self) -> numpy.ndarray:
        """Latitudes in decimal degrees"""
        return self._npdata['latitude']

    @property
    def _longitude(self) -> numpy.ndarray:
        """Longitudes in decimal degrees"""
        return self._npdata['longitude']

    @property
    def _baro_altitude(self) -> numpy.ndarray:
        """Barometric altitudes in meters"""
        return self._npdata['baro_altitude']

    @property
    def _gnss_altitude(self) -> numpy.ndarray:
        """GNSS altitudes in meters"""
        return self._npdata['gnss_altitude']

    @property
    def _timestamp(self) -> numpy.ndarray:
        """Timestamps in seconds since midnight"""
        return self._npdata['timestamp']

    def __len__(self) -> int:
        return len(self._npdata)

    def __getitem__(self, idx: int) -> Fix:
        """Get a single fix as Fix object"""
        fix = self._npdata[idx]

        # Do not let a negative index slip through
        return Fix(fix, idx if idx >= 0 else idx + len(self._npdata))

    def __iter__(self):
        """Iterate over fixes"""
        for i in range(len(self._npdata)):
            yield Fix(self._npdata[i], i)

    def __repr__(self) -> str:
        return f"Track(fixes={len(self)})"
