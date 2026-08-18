# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""High-level Python API for rustigcpy IGC parser

This package provides a convenient interface to the fast rustigcpy
parser with automatic numpy conversion and caching.
"""

from rustigcpy._bindings import league_names

from .fix import Fix
from .log import Log
from .score import Score
from .track import Track

__version__ = "0.1.0"
__all__ = ["Log", "Track", "Fix", "Score", "league_names"]
