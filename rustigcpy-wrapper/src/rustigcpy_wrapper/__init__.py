"""High-level Python wrapper for rustigcpy IGC parser

This package provides a convenient interface to the fast rustigcpy
parser with automatic numpy conversion and caching.

Example:
    >>> from rustigcpy_wrapper import Log
    >>> log = Log.from_file("flight.igc")
    >>> log.track.latitude.mean()  # numpy operations
    52.1234
"""

from .log import Log
from .track import Track
from .fix import Fix
from rustigcpy import FIX_DTYPE

__version__ = "0.1.0"
__all__ = ["Log", "Track", "Fix", "FIX_DTYPE"]
