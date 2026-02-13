# Rustigc - CLI Tool

Command-line tool for parsing IGC files and outputting structured JSON.

## Installation

```bash
cargo install --path .
```

## Usage

The tool reads IGC content from stdin and outputs JSON to stdout:

```bash
# Parse a file
rustigc < flight.igc

# Or with pipe
cat flight.igc | rustigc

# Save to file
rustigc < flight.igc > flight.json

# Quiet mode (no output, useful for profiling)
rustigc --quiet < flight.igc
```

## Output Format

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

MIT
