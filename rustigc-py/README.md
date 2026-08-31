# rustigc-py

Python bindings for [rustigc](../rustigc): parsing and cross-country scoring of IGC flight
recorder files.

## Installation

```bash
pip install rustigc-py
```

## Usage

```python
from rustigc_py import Log

log = Log.from_file("flight.igc")
print(log.pilot_name, log.glider_type, len(log.track))

flight = log.flights().longest
score = log.score("xcontest")            # or log.score("xcontest", (125, 25457))
if score:
    print(score.description, score.score, score.distance_km)

open("flight.geojson", "w").write(log.export(flight, score))
```

`Log` is the entry point. The API is documented in the docstrings — `help(rustigc_py.Log)`, or hover
in any editor; the package ships `py.typed`, so type checkers see the annotations too.

A track is a numpy structured array underneath, copied once from Rust and then read entirely in
Python. `Track` hands out views over it for vectorized work:

```python
log.track._latitude.mean()
log.track._data[0:100]
```

The array is read-only, so it always matches what Rust holds. A `Log` never changes either:
`Log.with_track` hands back a new one over the track it is given.

## Scoring without a log

`Scorer` scores a table of coordinates, so points that do not come from an IGC file score the same
way. It takes an `(N, 2)` array of `[latitude, longitude]` in degrees, in flight order:

```python
from rustigc_py import Scorer

score = Scorer(points).score("xcontest")
```

The whole table is the scored window — there are no timestamps to detect a flight in — and there
are no fixes behind the result, so `takeoff`, `entry`, `turnpoints`, `exit` and `landing` come back
as plain indices into the table rather than `Fix` objects.

`league_names()` lists what `score` accepts.

## Development

```bash
python -m venv venv && source venv/bin/activate
pip install maturin pytest
maturin develop
python -m pytest -v
```

## License

`GPL-2.0-or-later WITH Classpath-exception-2.0`
