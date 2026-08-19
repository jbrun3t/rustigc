# Rustigc - CLI Tools

Command-line toolbox for IGC files

## Installation

```bash
cargo install --path .
```

## 🏗️ `rustigc-xc-score`: scoring cli

The scoring tool reads the IGC content from stdin and outputs the result on stdout

```bash
> rustigc-xc-score < flight.igc
Entry @36
Exit  @12702
  - TP0 @372
  - TP1 @7852
  - TP2 @10462
Closed FAI Triangle 104.31 points, 65.2 km (×1.6) [ closing distance: 0.93 km ]
```

Or with a json output, specifying the league:
```bash
> rustigc-xc-score --league xcontest --format json < flight.igc
{
  "description": "Closed FAI Triangle",
  "distance": 65.2,
  "raw_distance": 65196.626,
  "gap": 0.93,
  "penalty": 0.93,
  "score": 104.31,
  "multiplier": 1.6,
  "takeoff": 0,
  "entry": 36,
  "turnpoints": [
    372,
    7852,
    10462
  ],
  "exit": 12702,
  "landing": 12911,
  "circuit": true
}
```

### Options

- `--league <name>` — ruleset to score against, `xcontest` by default.
- `--format <human|json>` — output format, `human` by default.
- `--window <start,stop>` — score this fix range instead of the auto-detected flight. The detected
  flight is still what the report calls takeoff/landing.

## `rustigc-parse`: Parser test tool

### Usage

The parsing testtool reads the IGC content from stdin and outputs JSON to stdout.
It is not terribly useful and mostly there to test and profile the parser

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

### Output Format

JSON structure with:
- `headers`: Flight metadata (pilot, glider, date, etc.)
- `recorder`: Flight recorder information
- `track`: Array of GPS fixes with coordinates and altitudes

Example output:
```json
{
  "headers": {
    "PLT": { "text": "John Smith", "origin": "flightrecorder" },
    "GTY": { "text": "Ozone Delta 4", "origin": "flightrecorder" },
    "DTE": { "text": "201024", "origin": "flightrecorder" }
  },
  "recorder": {
    "manufacturer": "XTR",
    "uid": "12345",
    "data": null
  },
  "track": [
    {
      "timestamp": 39695,
      "lat": 52.105,
      "lon": -0.103,
      "baro_alt": 587,
      "gnss_alt": 558
    }
  ]
}
```

## License

`GPL-2.0-or-later`
