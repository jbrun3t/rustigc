# Rustigc - CLI Tools

Command-line toolbox for IGC files

## Installation

```bash
cargo install --path .
```

## `rustigc-xc-score`: scoring CLI

Reads IGC content on stdin and writes the result to stdout.

```bash
> rustigc-xc-score < flight.igc
Flight on 2025-04-30 Europe/Paris
Takeoff: 12:12:16 - [45.3101,5.8907] - @0
 Entry : 12:27:10 - [45.2784,5.8467] - @894
  TP0  : 13:02:11 - [45.2290,5.7477] - @2995
  TP1  : 15:26:16 - [45.6716,5.8279] - @11639
  TP2  : 17:32:09 - [45.2574,6.0143] - @19192
 Exit  : 17:54:09 - [45.2678,5.8488] - @20512
Landing: 17:54:09 - [45.2678,5.8488] - @20512
closed free triangle 165.03 points, 117.88 km (×1.4) [ closing distance: 1.19 km ]
```

Or as JSON, for a chosen league:

```bash
> rustigc-xc-score --league xcontest --format json < flight.igc
{
  "league": "xcontest",
  "description": "closed free triangle",
  "distance_m": 117876.93,
  "distance_km": 117.88,
  "gap_km": 1.19,
  "penalty": 1.19,
  "score": 165.03,
  "multiplier": 1.4,
  "takeoff": 0,
  "entry": 894,
  "turnpoints": [
    2995,
    11639,
    19192
  ],
  "exit": 20512,
  "landing": 20512,
  "circuit": true
}
```

### Options

- `--league <name>` — ruleset to score against, `xcontest` by default. `--help` lists them.
- `--format <human|json|geojson>` — output format, `human` by default. `geojson` draws the track,
  the flight and the task.
- `--window <start,stop>` — score this fix range instead of the auto-detected flight. The detected
  flight is still what the report calls takeoff and landing.

## `rustigc-parse`: parser test tool

### Usage

Reads IGC content on stdin and writes JSON to stdout. It is not terribly useful and mostly there
to test and profile of the parser.

```bash
# Parse a file
rustigc-parse < flight.igc

# Or with pipe
cat flight.igc | rustigc-parse

# Save to file
rustigc-parse < flight.igc > flight.json

# Quiet (profiling) mode (no output)
rustigc-parse --quiet < flight.igc
```

### Output format

A JSON object with:
- `recorder`: the flight recorder that wrote the file
- `headers`: flight metadata, keyed by 3-letter code
- `track`: the position fixes
- `task`: the declared task, `null` when the file declares none

Example output, abridged:

```json
{
  "recorder": {
    "manufacturer": "XTR",
    "uid": "499BE7D1C91C",
    "data": null
  },
  "headers": {
    "PLT": { "text": "John Smith", "origin": "flightrecorder" },
    "GTY": { "text": "Ozone Delta 4", "origin": "flightrecorder" },
    "DTE": { "text": "201024", "origin": "flightrecorder" }
  },
  "track": [
    {
      "timestamp": 39695,
      "lat": 52.105,
      "lon": -0.103,
      "baro_alt": 587,
      "gnss_alt": 558
    }
  ],
  "task": null
}
```

## License

`GPL-2.0-or-later`
