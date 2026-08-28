# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""What a league's rules made of a flight."""
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .fix import Fix
    from .track import Track


class Score:
    """Result over a `Log` from a league scored over one window.

    Obtained from `Log.score`, and drawable by `Log.export`.

    Positions are resolved by indexing `positions`, so what `takeoff` and the rest hand back is
    whatever that holds: a `Track` gives `Fix` objects, and `range(n)` — what `Scorer` passes,
    having only coordinates — gives the plain indices back.
    """

    def __init__(self, positions: 'Track | range', data: dict):
        self._positions = positions
        self._data = data

    def __getattr__(self, name):
        """Scalars of the result.

        `description` names the winning rule, `score` is the league points, `distance_km` the
        scored distance as the rule presents it and `distance_m` the same in meters. `gap_km` is
        a circuit's closing leg, `threshold_m` the largest gap the rule would still have accepted,
        `penalty` what the rule charged for the gap, `multiplier` the rate it scored at, and
        `circuit` whether the task closes on itself.
        """
        try:
            return self._data[name]
        except KeyError:
            raise AttributeError(name) from None

    @property
    def takeoff(self) -> 'Fix | int':
        """First fix of the scored window."""
        return self._positions[self._data["takeoff"]]

    @property
    def entry(self) -> 'Fix | int':
        """First fix of the scored task."""
        return self._positions[self._data["entry"]]

    @property
    def turnpoints(self) -> list['Fix | int']:
        """Turnpoints of the task, in order."""
        return [self._positions[i] for i in self._data["turnpoints"]]

    @property
    def exit(self) -> 'Fix | int':
        """Last fix of the scored task."""
        return self._positions[self._data["exit"]]

    @property
    def landing(self) -> 'Fix | int':
        """Last fix of the scored window."""
        return self._positions[self._data["landing"]]

    def __repr__(self) -> str:
        return (f"Score({self.description!r}, score={self.score}, "
                f"distance_km={self.distance_km})")
