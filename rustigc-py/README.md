# rustigc-py

Python bindings for [rustigc](../rustigc): parsing and cross-country scoring of IGC flight
recorder files.

## Installation

```bash
pip install rustigcpy
```

## Usage

```python
from rustigcpy import Log

log = Log.from_file("flight.igc")
print(log.pilot_name, log.glider_type, len(log.track))

flight = log.flights().longest
score = log.score("xcontest")            # or log.score("xcontest", (125, 25457))
if score:
    print(score.description, score.score, score.distance)

open("flight.geojson", "w").write(log.export([flight, score]))
```

`Log` is the entry point. The API is documented in the docstrings — `help(rustigcpy.Log)`, or hover
in any editor; the package ships `py.typed`, so type checkers see the annotations too.

A track is a numpy structured array underneath, copied once from Rust and then read entirely in
Python. `Track` hands out views over it for vectorized work:

```python
log.track._latitude.mean()
log.track._data[0:100]
```

The array is read-only, so it always matches what Rust holds; `Log.push` is the way to change a
track.

`rustigcpy._bindings` is the raw extension module this package is built on, not an interface to
use directly. `league_names()` lists what `score` accepts.

## Development

```bash
python -m venv venv && source venv/bin/activate
pip install maturin pytest
maturin develop
python -m pytest -v
```

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
