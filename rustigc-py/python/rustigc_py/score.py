# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""The result of scoring a flight."""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .fix import Fix
    from .track import Track


class Score:
    """The best result of a ruleset over one window.

    Obtained from `Log.score` or `Scorer.score`, and drawable by `Log.export`. `takeoff`,
    `entry`, `turnpoints`, `exit` and `landing` are `Fix` objects from `Log.score`, plain indices
    from `Scorer.score`.
    """

    def __init__(self, positions: "Track | range", data: dict):
        self._positions = positions
        self._data = data

    def __getattr__(self, name):
        """Scalars of the result.

        `description` names the winning variant and `score` is its points. `distance_km` and
        `distance_m` are the scored distance, net of `penalty`, the kilometers the gap took off.
        `gap_m` is a circuit's closing leg and `threshold_m` the largest gap the variant closes,
        in meters. `multiplier` is the rate it scored at, and `circuit` whether the task closes
        on itself.
        """
        try:
            return self._data[name]
        except KeyError:
            raise AttributeError(name) from None

    @property
    def takeoff(self) -> "Fix | int":
        """First fix of the scored window."""
        return self._positions[self._data["takeoff"]]

    @property
    def entry(self) -> "Fix | int":
        """First fix of the scored task."""
        return self._positions[self._data["entry"]]

    @property
    def turnpoints(self) -> list["Fix | int"]:
        """Turnpoints of the task, in order."""
        return [self._positions[i] for i in self._data["turnpoints"]]

    @property
    def exit(self) -> "Fix | int":
        """Last fix of the scored task."""
        return self._positions[self._data["exit"]]

    @property
    def landing(self) -> "Fix | int":
        """Last fix of the scored window."""
        return self._positions[self._data["landing"]]

    def __repr__(self) -> str:
        return f"Score({self.description!r}, score={self.score}, distance_km={self.distance_km})"
