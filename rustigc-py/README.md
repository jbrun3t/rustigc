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
import numpy as np

# Parse IGC file
with open("flight.igc", "rb") as f:
    log = rustigcpy.Log.from_bytes(f.read())

# Access metadata
print(f"Pilot: {log.pilot_name()}")
print(f"Glider: {log.glider_type()}")
print(f"Date: {log.date()}")

# Flight phases
print(f"Takeoff: {log.takeoff}")
print(f"Landing: {log.landing}")

# Access track data as numpy array
track = np.frombuffer(log.track_bytes, dtype=rustigcpy.FIX_DTYPE)
print(f"Fixes: {len(track)}")

# Access fields
print(f"Latitudes: {track['latitude']}")
print(f"Longitudes: {track['longitude']}")
print(f"Altitudes: {track['baro_altitude']}")
print(f"Timestamps: {track['timestamp']}")
```

## API

### `rustigcpy.Log`

**Methods:**
- `Log.from_bytes(content: bytes) -> Log` - Parse IGC file from bytes
- `pilot_name() -> str | None` - Get pilot name from headers
- `glider_type() -> str | None` - Get glider type from headers
- `date() -> str | None` - Get date from headers (DDMMYY format)

**Properties:**
- `track_bytes: bytes` - Raw track data (32 bytes per fix)
- `takeoff: int | None` - Takeoff fix index
- `landing: int | None` - Landing fix index

### `rustigcpy.FIX_DTYPE`

NumPy dtype for track data (32 bytes per fix):
- `latitude: f64` - Decimal degrees
- `longitude: f64` - Decimal degrees
- `baro_altitude: i32` - Barometric altitude in meters
- `gnss_altitude: i32` - GNSS altitude in meters
- `timestamp: u32` - Seconds since midnight
- `_pad: u32` - Alignment padding

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
