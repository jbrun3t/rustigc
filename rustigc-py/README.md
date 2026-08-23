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
print(f"Origin: {log.datetime}")  # 2022-08-05 01:00:00+01:00, what fix timestamps count from

# Flight detection, cached after the first call
flight = log.flights().longest   # None when nothing was detected
print(f"Sections: {len(log.flights())}")
print(f"Takeoff: {flight.takeoff}")  # Returns Fix object
print(f"Landing: {flight.landing}")  # Returns Fix object

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

# Editing the track: work on a copy, then push it back
edited = track._data.copy()
edited['baro_altitude'] += 10   # or trim: track._data[1000:]
log.push(edited)                # fix count may change, timestamps stay increasing

# Vectorized numpy operations (fast / intended for internal usage mostly)
mean_lat = track._latitude.mean()
max_alt = track._baro_altitude.max()
lats = track._latitude
first_100 = track._data[0:100]  # numpy slice

# Scoring, over the detected flight or an explicit window
from rustigcpy import league_names

print(league_names())
score = log.score("xcontest")
score = log.score("xcontest", (125, 25457))                  # fix indices
score = log.score("xcontest", (flight.takeoff, flight.landing))  # or fixes

if score:
    print(f"{score.description}: {score.score} points over {score.distance} km")
    print(f"Turnpoints: {[tp.index for tp in score.turnpoints]}")

# GeoJSON, either picked for you or chosen layer by layer
open("flight.geojson", "w").write(log.describe("xcontest"))

geojson = log.export([flight, score])          # in the order given
task_only = log.export([score], track=False)   # without the flown line
```

Both return the GeoJSON as a string. `describe` takes the longest flight detected and what it scored;
`export` draws the flights and scores handed to it, and nothing else. See
[the core's role table](../rustigc/README.md#geojson-export) for what the features carry.

### Architecture

- Track data is copied from Rust into Python numpy array on first `.track` access
- All subsequent operations are local Python (no FFI calls)
- Track is cached for efficient repeated access
- Flight detection is cached too, `reset()` drops it
- The track is read-only, so it always matches what Rust holds.
- `push()` is the way to change it. it drops the caches so everything derived is computed again.

## Low level bindings

Low-level Python bindings for the rustigc IGC file parser.

### (Discouraged) Usage

```python
import rustigcpy._bindings as rib
import json
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

# Flight detection, one opaque handle per section, each dumping itself as JSON
sections = [json.loads(f.json()) for f in log.flights()]
print(f"Sections: {sections}")  # [{"start": 125, "stop": 25425}]

# Access track data as numpy array
track = numpy.frombuffer(log.track_bytes, dtype=rustigcpy.FIX_DTYPE)
print(f"Fixes: {len(track)}")

# Access fields
print(f"Timestamps: {track['timestamp']}")
print(f"Latitudes: {track['latitude']}")
print(f"Longitudes: {track['longitude']}")
print(f"Altitudes: {track['baro_altitude']}")
```

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

`GPL-2.0-or-later WITH Classpath-exception-2.0`
