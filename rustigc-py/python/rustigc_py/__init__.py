# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Parsing and cross-country scoring of IGC flight recorder files.

`Log` is the entry point. It parses an IGC file, exposes its metadata and its `Track`, detects
the flights in it, scores them against a `Ruleset`, and draws the result as GeoJSON:

    from rustigc_py import Log, Ruleset

    log = Log.from_file("flight.igc")
    print(log.pilot_name, len(log.track))

    flight = log.flights().longest
    score = log.score(Ruleset.preset("xcontest"))
    if score:
        print(score.description, score.score, score.distance_km)

    open("flight.geojson", "w").write(log.export(flight, score))

The track is a numpy structured array, copied once out of Rust. Parsing, detection and scoring
release the GIL.

`Scorer` scores a table of coordinates on its own, with no IGC file behind it:

    from rustigc_py import Ruleset, Scorer

    score = Scorer(points).score(Ruleset.preset("xcontest"))   # points: (N, 2) [lat, lon]

A ruleset is a preset, listed by `preset_names()`, or a caller's own JSON content, read with
`Ruleset.from_json`.

`rustigc_py._bindings` is the raw extension module this package is built on, not an interface to use
directly.
"""

from importlib.metadata import version

from rustigc_py._bindings import preset_names

from .fix import Fix
from .flight import Flight, Flights
from .log import Log
from .ruleset import Ruleset
from .score import Score
from .scorer import Scorer
from .track import Track

__version__ = version("rustigc-py")
__all__ = [
    "Log",
    "Scorer",
    "Ruleset",
    "Track",
    "Fix",
    "Flight",
    "Flights",
    "Score",
    "preset_names",
]
