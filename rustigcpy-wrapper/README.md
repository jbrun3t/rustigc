# rustigcpy-wrapper

High-level Python wrapper for rustigcpy IGC parser with numpy integration.

## Installation

```bash
pip install rustigcpy-wrapper
```

## Usage

```python
from rustigcpy_wrapper import Log

# Parse IGC file
log = Log.from_file("flight.igc")

# Access metadata
print(f"Pilot: {log.pilot_name}")
print(f"Glider: {log.glider_type}")
print(f"Date: {log.date}")
print(f"Takeoff: {log.takeoff}, Landing: {log.landing}")

# Access track data
track = log.track
print(f"Fixes: {len(track)}")

# Vectorized numpy operations (fast)
mean_lat = track._latitude.mean()
max_alt = track._baro_altitude.max()
lats = track._latitude  # numpy array

# Object-oriented access (convenient)
fix = track[0]  # Returns Fix object
print(f"Lat: {fix.latitude}, Lon: {fix.longitude}")

# Iterate over fixes
for fix in track:
    print(f"{fix.timestamp}s - {fix.latitude:.6f}, {fix.longitude:.6f}")
    break

# Expert API: direct numpy slicing
first_100 = track._data[0:100]  # numpy slice
```

## API

### `Log`

**Class methods:**
- `Log.from_bytes(content: bytes) -> Log`
- `Log.from_file(path: str) -> Log`

**Properties:**
- `pilot_name: str | None`
- `glider_type: str | None`
- `date: datetime.date | None`
- `takeoff: int | None`
- `landing: int | None`
- `track: Track`

### `Track`

**Properties:**
- `_latitude: np.ndarray` - Array of latitudes (f64)
- `_longitude: np.ndarray` - Array of longitudes (f64)
- `_baro_altitude: np.ndarray` - Array of barometric altitudes (i32)
- `_gnss_altitude: np.ndarray` - Array of GNSS altitudes (i32)
- `_timestamp: np.ndarray` - Array of timestamps in seconds since midnight (u32)
- `_data: np.ndarray` - Full structured numpy array

**Methods:**
- `__len__() -> int` - Number of fixes
- `__getitem__(index: int) -> Fix` - Get single Fix object
- `__iter__() -> Iterator[Fix]` - Iterate over Fix objects

### `Fix`

Single position fix from IGC track.

**Properties:**
- `latitude: float` - Latitude in decimal degrees
- `longitude: float` - Longitude in decimal degrees
- `baro_altitude: int` - Barometric altitude in meters
- `gnss_altitude: int` - GNSS altitude in meters
- `timestamp: int` - Seconds since midnight

## Architecture

- Track data is copied from Rust into Python numpy array on first `.track` access
- All subsequent operations are local Python (no FFI calls)
- Track is cached for efficient repeated access
