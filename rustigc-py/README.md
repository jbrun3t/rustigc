# Rustigc - Python Bindings

Python bindings for the rustigc IGC file parser.

## Installation

From PyPI (when published):
```bash
pip install rustigcpy
```

From source:
```bash
pip install maturin
maturin develop
```

## Usage

```python
import rustigcpy

# Read IGC file as bytes (pass content, not file path)
with open("flight.igc", "rb") as f:
    content = f.read()

# Parse
log = rustigcpy.Log.from_bytes(content)

# Access metadata
print(f"Pilot: {log.pilot_name()}")
print(f"Glider: {log.glider_type()}")
print(f"Date: {log.date()}")  # Returns raw DDMMYY string
print(f"Fixes: {len(log)}")

# Access individual fixes by index
first_fix = log[0]
last_fix = log[-1]  # Negative indexing supported

# Iterate over all fixes
for fix in log.fixes():
    print(f"{fix.timestamp}s - {fix.latitude:.5f}, {fix.longitude:.5f} @ {fix.gnss_altitude}m")
    # fix.timestamp: seconds since midnight (0-86399)
    # fix.latitude: decimal degrees
    # fix.longitude: decimal degrees
    # fix.gnss_altitude: meters
    # fix.baro_altitude: meters (barometric)

# Detect takeoff/landing (returns fix indices)
if log.takeoff is not None:
    takeoff_fix = log[log.takeoff]
    print(f"Takeoff at {takeoff_fix.timestamp}s")

if log.landing is not None:
    landing_fix = log[log.landing]
    print(f"Landing at {landing_fix.timestamp}s")
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

MIT
