# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Scoring a table of coordinates, with no IGC file behind it."""
import numpy

import rustigcpy._bindings as rib

from .score import Score


class Scorer:
    """A table of coordinates, scorable against a league."""

    def __init__(self, points: numpy.ndarray):
        """Hold an `(N, 2)` array of `[latitude, longitude]`, degrees, in flight order.

        Args:
            points: The table. Converted to C-contiguous float64 if it is not already.

        Raises:
            ValueError: Not a 2-dimensional `(N, 2)` array, or it holds fewer than two points, or
                a coordinate is not a finite latitude/longitude in degrees.
        """
        table = numpy.ascontiguousarray(points, dtype=numpy.float64)
        if table.ndim != 2:
            raise ValueError(f"points must be a 2-dimensional array, got {table.ndim}")

        self._scorer = rib.RustScorer(table)
        self._points = len(table)

    def score(self, league: str) -> Score | None:
        """Score the table and report the best rule of `league`.

        There are no fixes behind the result, so `takeoff`, `entry`, `turnpoints`, `exit` and
        `landing` come back as plain indices into the table.

        Args:
            league: One of `league_names()`.

        Returns:
            The best scoring result, or None when the league is unknown or nothing scored.
        """
        handle = self._scorer.score(league)
        if handle is None:
            return None

        # A `Score` resolves a position by indexing what it is given, and `range` is the sequence
        # that hands an index straight back.
        return Score(range(self._points), handle)
