# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Flight sections detected in a track."""
import json
from typing import TYPE_CHECKING

from .track import Track

if TYPE_CHECKING:
    from .fix import Fix


class Flight:
    """One flight section detected in a track.

    Obtained from `Log.flights()`, and drawable by `Log.export`. `start` and `stop` are indices
    into the track it was detected in; `takeoff` and `landing` are the fixes they point at.
    """

    def __init__(self, track: Track, handle):
        # The handle is the section as Rust still holds it, so `Log.export` can draw it without
        # detecting again. Its scalars are read from the JSON dump.
        self._handle = handle
        self._track = track
        self._data = json.loads(handle.json())

    def __getattr__(self, name):
        """Scalars of the section as detection reports them: `start`, `stop`."""
        try:
            return self._data[name]
        except KeyError:
            raise AttributeError(name) from None

    @property
    def takeoff(self) -> 'Fix':
        """First fix of the section."""
        return self._track[self._data["start"]]

    @property
    def landing(self) -> 'Fix':
        """Last fix of the section."""
        return self._track[self._data["stop"]]

    def __repr__(self) -> str:
        return f"Flight(start={self._data['start']}, stop={self._data['stop']})"


class Flights(list):
    """The sections of one detection pass, as a list, empty when none was detected."""

    @property
    def longest(self) -> Flight | None:
        """The longest section by fix span, None when the list is empty."""
        return max(self, key=lambda f: f.stop - f.start, default=None)
