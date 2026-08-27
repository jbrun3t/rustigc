# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Parsing and cross-country scoring of IGC flight recorder files.

`Log` is the entry point. It parses an IGC file, exposes its metadata and its `Track`, detects
the flights in it, scores them against a league, and draws the result as GeoJSON:

    from rustigcpy import Log

    log = Log.from_file("flight.igc")
    print(log.pilot_name, len(log.track))

    flight = log.flights().longest
    score = log.score("xcontest")
    if score:
        print(score.description, score.score, score.distance_km)

    open("flight.geojson", "w").write(log.export([flight, score]))

The track is a numpy structured array underneath, copied once from Rust and then read entirely in
Python. Parsing, detection and scoring release the GIL.

`rustigcpy._bindings` is the raw extension module this package is built on, not an interface to use
directly.
"""

from rustigcpy._bindings import league_names

from .fix import Fix
from .flight import Flight, Flights
from .log import Log
from .score import Score
from .track import Track

__version__ = "0.1.0"
__all__ = ["Log", "Track", "Fix", "Flight", "Flights", "Score", "league_names"]
