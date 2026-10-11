# rustigc-tools

Command-line toolbox for IGC files

## Installation

```bash
cargo install --path .
```

## `rustigc-xc-score`: scoring CLI

Reads IGC content on stdin and writes the result to stdout.

```bash
> rustigc-xc-score < fai-01.igc
Flight on 2022-06-21 -03
Takeoff: 06:11:37 - [51.1883,-41.0285] - @125
 Entry : 06:11:37 - [51.1883,-41.0285] - @125
  TP0  : 06:32:27 - [51.1489,-41.0439] - @1374
  TP1  : 09:26:12 - [52.0284,-43.1461] - @11790
  TP2  : 11:31:01 - [52.9680,-40.0616] - @19270
 Exit  : 13:13:46 - [51.1888,-41.0280] - @25421
Landing: 13:13:50 - [51.1889,-41.0283] - @25425
closed fai triangle 996.56 points, 622.85 km (×1.6) [ closing distance: 67 / 31146 m ]
```

Or as JSON, for a chosen ruleset:

```bash
> rustigc-xc-score --ruleset xcontest --format json < fai-01.igc
{
  "ruleset": "xcontest",
  "description": "closed fai triangle",
  "distance_m": 622852.252,
  "gap_m": 67.275,
  "threshold_m": 31145.976,
  "distance_km": 622.85,
  "score": 996.56,
  "penalty": 0.07,
  "multiplier": 1.6,
  "takeoff": 125,
  "entry": 125,
  "turnpoints": [
    1374,
    11790,
    19270
  ],
  "exit": 25421,
  "landing": 25425,
  "circuit": true
}
```

### Options

- `--ruleset <name>` — ruleset to score against, `xcontest` by default. `--help` lists them.
- `--ruleset-file <path>` — ruleset read from a JSON file in the presets' format, instead of
  `--ruleset`.
- `--format <human|json|geojson>` — output format, `human` by default. `geojson` draws the track,
  the flight and the task.
- `--window <start,stop>` — score this fix range instead of the detected flight; takeoff and
  landing are then its ends.

## `rustigc-parse`: parser test tool

### Usage

Reads IGC content on stdin and writes the parsed log as JSON to stdout. Meant for testing and
profiling the parser.

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
      "timestamp": 39695000,
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
