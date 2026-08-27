# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""What a league's rules made of a flight."""
import json
from typing import TYPE_CHECKING

from .track import Track

if TYPE_CHECKING:
    from .fix import Fix


class Score:
    """Result over a `Log` from a league scored over one window.

    Obtained from `Log.score`, and drawable by `Log.export`.
    """

    def __init__(self, track: Track, handle):
        # The handle is the result as Rust still holds it, so `Log.export` can draw it without
        # scoring again. Its scalars are read from the JSON dump.
        self._handle = handle
        self._track = track
        self._data = json.loads(handle.json())

    def __getattr__(self, name):
        """Scalars of the result.

        `description` names the winning rule, `score` is the league points, `distance_km` the
        scored distance as the rule presents it and `distance_m` the same in meters. `gap_km` is
        a circuit's closing leg, `penalty` what the rule charged for it, `multiplier` the rate
        it scored at, and `circuit` whether the task closes on itself.
        """
        try:
            return self._data[name]
        except KeyError:
            raise AttributeError(name) from None

    @property
    def takeoff(self) -> 'Fix':
        """First fix of the scored window."""
        return self._track[self._data["takeoff"]]

    @property
    def entry(self) -> 'Fix':
        """First fix of the scored task."""
        return self._track[self._data["entry"]]

    @property
    def turnpoints(self) -> list['Fix']:
        """Turnpoints of the task, in order."""
        return [self._track[i] for i in self._data["turnpoints"]]

    @property
    def exit(self) -> 'Fix':
        """Last fix of the scored task."""
        return self._track[self._data["exit"]]

    @property
    def landing(self) -> 'Fix':
        """Last fix of the scored window."""
        return self._track[self._data["landing"]]

    def __repr__(self) -> str:
        return (f"Score({self.description!r}, score={self.score}, "
                f"distance_km={self.distance_km})")
