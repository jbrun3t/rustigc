# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Flight wrapper"""
from typing import TYPE_CHECKING

from .track import Track

if TYPE_CHECKING:
    from .fix import Fix


class Flight:
    """One flight section detected in a track"""

    def __init__(self, track: Track, data: dict):
        self._track = track
        self._data = data

    def __getattr__(self, name):
        """Scalars of the section, as detection reports them"""
        try:
            return self._data[name]
        except KeyError:
            raise AttributeError(name) from None

    @property
    def takeoff(self) -> 'Fix':
        """First fix of the section"""
        return self._track[self._data["start"]]

    @property
    def landing(self) -> 'Fix':
        """Last fix of the section"""
        return self._track[self._data["stop"]]

    def __repr__(self) -> str:
        return f"Flight(start={self._data['start']}, stop={self._data['stop']})"


class Flights(list):
    """Sections of one detection pass, empty when none was detected"""

    @property
    def longest(self) -> Flight | None:
        """Longest section by fix span"""
        return max(self, key=lambda f: f.stop - f.start, default=None)
