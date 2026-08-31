# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""The position fixes of a log."""

import numpy

from rustigc_py._bindings import FIX_DTYPE

from .fix import Fix


class Track:
    """A track of position fixes, as a read-only sequence of `Fix`.

    Indexing and iteration yield `Fix` objects. The underlying numpy structured array is exposed
    through the underscore properties, for vectorized work:

        track._latitude.mean()
        track._data[0:100]

    They expose the fix layout directly, hence the underscores. The array is read-only, so it
    always matches what Rust holds; `Log.with_track` hands back a log over a different one.
    """

    def __init__(self, track_bytes: bytes):
        """Wrap raw `FIX_DTYPE` bytes, copying them into a numpy array once."""
        # The one copy: everything after this is local Python
        self._npdata = numpy.frombuffer(track_bytes, dtype=FIX_DTYPE)

    @property
    def _data(self) -> numpy.ndarray:
        """The whole track, as a `FIX_DTYPE` structured array."""
        return self._npdata

    @property
    def _latitude(self) -> numpy.ndarray:
        """Latitudes in decimal degrees"""
        return self._npdata["latitude"]

    @property
    def _longitude(self) -> numpy.ndarray:
        """Longitudes in decimal degrees"""
        return self._npdata["longitude"]

    @property
    def _baro_altitude(self) -> numpy.ndarray:
        """Barometric altitudes in meters"""
        return self._npdata["baro_altitude"]

    @property
    def _gnss_altitude(self) -> numpy.ndarray:
        """GNSS altitudes in meters"""
        return self._npdata["gnss_altitude"]

    @property
    def _timestamp(self) -> numpy.ndarray:
        """Timestamps, in milliseconds from the instant `Log.datetime` reports."""
        return self._npdata["timestamp"]

    def __len__(self) -> int:
        return len(self._npdata)

    def __getitem__(self, idx: int) -> Fix:
        """The fix at `idx`, negative indices counted from the end."""
        fix = self._npdata[idx]

        # Do not let a negative index slip through
        return Fix(fix, idx if idx >= 0 else idx + len(self._npdata))

    def __iter__(self):
        """Every fix in order, each knowing its own index."""
        for i in range(len(self._npdata)):
            yield Fix(self._npdata[i], i)

    def __repr__(self) -> str:
        return f"Track(fixes={len(self)})"
