# rustigc-py - Python Bindings

Low-level Python bindings for the rustigc IGC file parser.

**Note:** For a high-level Python API with numpy integration, see `rustigc-py-wrapper`.

## Installation

From source:
```bash
pip install maturin
maturin develop
```

## Usage

```python
import rustigcpy
import numpy

# Parse IGC file
with open("flight.igc", "rb") as f:
    log = rustigcpy.Log.from_bytes(f.read())

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

## API

### `rustigcpy.Log`

**Methods:**
- `Log.from_bytes(content: bytes) -> Log` - Parse IGC file from bytes
- `get_header(key: str) -> tuple[str, str] | None` - Get header by 3-char key (e.g., "PLT", "GTY", "DTE")
  - Returns `(text, origin)` where origin is "Flight Recorder", "Observer", or "Pilot"
- `analyze() -> None` - Run flight phase analysis

**Properties:**
- `track_bytes: bytes` - Raw track data (32 bytes per fix)
- `takeoff: int | None` - Takeoff fix index
- `landing: int | None` - Landing fix index

### `rustigcpy.FIX_DTYPE`

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
