# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Scoring a table of coordinates, with no IGC file behind it."""

import json

import numpy

import rustigc_py._bindings as rib

from .score import Score


class Scorer:
    """A table of coordinates, scorable against a ruleset."""

    def __init__(self, points: numpy.ndarray):
        """Hold an `(N, 2)` array of `[latitude, longitude]`, degrees, in flight order.

        Args:
            points: The table, made C-contiguous if it is not. Coordinates are float64.

        Raises:
            ValueError: Not a 2-dimensional `(N, 2)` array, or it holds fewer than two points, or
                a coordinate is not a finite latitude/longitude in degrees.
        """
        table = numpy.ascontiguousarray(points, dtype=numpy.float64)
        if table.ndim != 2:
            raise ValueError(f"points must be a 2-dimensional array, got {table.ndim}")

        self._scorer = rib.RustScorer(table)
        self._points = len(table)

    def score(self, ruleset: str) -> Score | None:
        """Score the table against every rule of `ruleset`, reporting the best.

        There are no fixes behind the result, so `takeoff`, `entry`, `turnpoints`, `exit` and
        `landing` come back as plain indices into the table.

        Args:
            ruleset: One of `ruleset_names()`.

        Returns:
            The best `Score`, or None when no rule scores the table.

        Raises:
            ValueError: `ruleset` is not one of `ruleset_names()`.
        """
        scored = self._scorer.score(ruleset)
        if scored is None:
            return None

        # Indexing a `range` hands the index back as the position.
        return Score(range(self._points), json.loads(scored))
