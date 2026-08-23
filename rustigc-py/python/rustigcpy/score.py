# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Score wrapper"""
import json
from typing import TYPE_CHECKING

from .track import Track

if TYPE_CHECKING:
    from .fix import Fix


class Score:
    """Scoring result of one league over one window"""

    def __init__(self, track: Track, handle):
        # The handle is the result as Rust still holds it, so `Log.export` can draw it without
        # scoring again. Its scalars are read from the JSON dump.
        self._handle = handle
        self._track = track
        self._data = json.loads(handle.json())

    def __getattr__(self, name):
        """Scalars of the result: description, distance, score, gap, penalty, multiplier, circuit"""
        try:
            return self._data[name]
        except KeyError:
            raise AttributeError(name) from None

    @property
    def takeoff(self) -> 'Fix':
        """Start of the scoring window"""
        return self._track[self._data["takeoff"]]

    @property
    def entry(self) -> 'Fix':
        """Start fix of the task scored"""
        return self._track[self._data["entry"]]

    @property
    def turnpoints(self) -> list['Fix']:
        """Turnpoints of the task"""
        return [self._track[i] for i in self._data["turnpoints"]]

    @property
    def exit(self) -> 'Fix':
        """Stop fix of the task scored"""
        return self._track[self._data["exit"]]

    @property
    def landing(self) -> 'Fix':
        """End of the scoring window"""
        return self._track[self._data["landing"]]

    def __repr__(self) -> str:
        return (f"Score({self.description!r}, score={self.score}, "
                f"distance={self.distance})")
