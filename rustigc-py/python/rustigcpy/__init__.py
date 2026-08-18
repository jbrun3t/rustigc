"""High-level Python API for rustigcpy IGC parser

This package provides a convenient interface to the fast rustigcpy
parser with automatic numpy conversion and caching.
"""

from .log import Log
from .track import Track
from .fix import Fix

__version__ = "0.1.0"
__all__ = ["Log", "Track", "Fix"]
