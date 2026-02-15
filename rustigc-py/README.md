# rustigc-py

Python bindings for rustigcpy IGC parser.

## Installation

```bash
pip install rustigcpy
```

## High level Wrapper interface

### Usage

```python
from rustigcpy import Log

# Parse IGC file
log = Log.from_file("flight.igc")

# Access metadata
print(f"Pilot: {log.pilot_name}")
print(f"Glider: {log.glider_type}")
print(f"Date: {log.date}")

# Flight phase analysis
log.analyze()  # Optional: manually trigger analysis
print(f"Takeoff: {log.takeoff}")  # Returns Fix object
print(f"Landing: {log.landing}")  # Returns Fix object

# Access track data
track = log.track
print(f"Fixes: {len(track)}")

# Object-oriented access (convenient)
fix = track[0]  # Returns Fix object
print(f"Lat: {fix.latitude}, Lon: {fix.longitude}")

# Iterate over fixes
for fix in track:
    print(f"{fix.timestamp}s - {fix.latitude:.6f}, {fix.longitude:.6f}")
    break

# Vectorized numpy operations (fast / intended for internal usage mostly)
mean_lat = track._latitude.mean()
max_alt = track._baro_altitude.max()
lats = track._latitude
first_100 = track._data[0:100]  # numpy slice
```

### API

#### `Log`

**Class methods:**
- `Log.from_bytes(content: bytes) -> Log`
- `Log.from_file(path: str) -> Log`  (Consider Removing)

**Methods:**
- `analyze() -> None` - Run flight phase analysis

**Properties:**
- `pilot_name: str | None`
- `glider_type: str | None`
- `date: datetime.date | None`
- `takeoff: Fix | None`
- `landing: Fix | None`
- `track: Track`

#### `Track`

**Properties:**
- `_latitude: np.ndarray` - Array of latitudes (f64)
- `_longitude: np.ndarray` - Array of longitudes (f64)
- `_baro_altitude: np.ndarray` - Array of barometric altitudes (i32)
- `_gnss_altitude: np.ndarray` - Array of GNSS altitudes (i32)
- `_timestamp: np.ndarray` - Array of timestamps in seconds since midnight (u32)
- `_data: np.ndarray` - Full structured numpy array

#### `Fix`

Single position fix from IGC track.

**Properties:**
- `latitude: float` - Latitude in decimal degrees
- `longitude: float` - Longitude in decimal degrees
- `baro_altitude: int` - Barometric altitude in meters
- `gnss_altitude: int` - GNSS altitude in meters
- `timestamp: int` - Seconds since midnight

### Architecture

- Track data is copied from Rust into Python numpy array on first `.track` access
- All subsequent operations are local Python (no FFI calls)
- Track is cached for efficient repeated access


## Low level bindings

Low-level Python bindings for the rustigc IGC file parser.

### (Discouraged) Usage

```python
import rustigcpy._bindings as rib
import numpy

# Parse IGC file
with open("flight.igc", "rb") as f:
    log = rib.RustLog.from_bytes(f.read())

# Access metadata (returns tuple of text and origin)
pilot = log.get_header("PLT")
if pilot:
    text, origin = pilot
    print(f"Pilot: {text} (from {origin})")

glider = log.get_header("GTY")
if glider:
    print(f"Glider: {glider[0]}")

# Flight phases (run analysis first if needed)
log.analyze()
print(f"Takeoff index: {log.takeoff}")
print(f"Landing index: {log.landing}")

# Access track data as numpy array
track = numpy.frombuffer(log.track_bytes, dtype=rustigcpy.FIX_DTYPE)
print(f"Fixes: {len(track)}")

# Access fields
print(f"Timestamps: {track['timestamp']}")
print(f"Latitudes: {track['latitude']}")
print(f"Longitudes: {track['longitude']}")
print(f"Altitudes: {track['baro_altitude']}")
```

### API

### `rustigcpy._bindings.RustLog`

**Methods:**
- `RustLog.from_bytes(content: bytes) -> Log` - Parse IGC file from bytes
- `get_header(key: str) -> tuple[str, str] | None` - Get header by 3-char key (e.g., "PLT", "GTY", "DTE")
  - Returns `(text, origin)` where origin is "Flight Recorder", "Observer", or "Pilot"
- `analyze() -> None` - Run flight phase analysis

**Properties:**
- `track_bytes: bytes` - Raw track data (32 bytes per fix)
- `takeoff: int | None` - Takeoff fix index
- `landing: int | None` - Landing fix index

### `rustigcpy._bindings.FIX_DTYPE`

NumPy dtype for track data (32 bytes per fix):
- `timestamp: u32` - Seconds since midnight
- `_pad: u32` - Alignment padding
- `latitude: f64` - Decimal degrees
- `longitude: f64` - Decimal degrees
- `baro_altitude: i32` - Barometric altitude in meters
- `gnss_altitude: i32` - GNSS altitude in meters

## Development

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate

# Install development dependencies
pip install maturin pytest

# Build and install in development mode
maturin develop

# Run tests
python -m pytest -v
```

## License

MIT
