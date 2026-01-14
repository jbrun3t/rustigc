# Rustigc - Python bindings

## Build & Test

```bash
# Create a venv (optional)
python -m venv venv
source venv/bin/activate

Install with pip
```bash
pip install --upgrade pip
pip install rustigc-py/
```

Or with maturin
```
pip install maturin
maturin develop
```

The python binding also come with unit tests
```
pip instal pytest
python -m pytest -v
```

## Usage example

```python
import rustigcpy

# Read content (not file path!)
with open("track.igc") as f:
    content = f.read()

# Parse
log = rustigcpy.Log.from_string(content)

# Access metadata
print(f"Pilot: {log.pilot_name()}")
print(f"Glider: {log.glider_type()}")
print(f"Date: {track.date()}")   # Return the raw DDMMYY
print(f"Fixes: {len(log)}")

# Get fixes (simple types - all standard units)
for fix in track.fixes():
    t = fix.timestamp  # number of second since the beginning of UTC day.
    lat = fix.latitude  # Decimal degrees
    lon = fix.longitude  # Decimal degrees
    alt = fix.gnss_altitude  # Meters
    print(f"{t}s - {lat:.5f}, {lon:.5f} @ {alt}m")

# Detect takeoff/landing
if track.takeoff:
    print(f"Takeoff at {track.takeoff.timestamp()}s")
if track.landing:
    print(f"Landing at {track.landing.timestamp()}s")
````

## License

MIT
