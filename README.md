# Rustigc - Workspace

Fast IGC (International Gliding Commission) flight recorder file parser written in Rust with Python bindings.

## Features

- ✅ Parse IGC files to a collection of records (RawLog)
- ✅ Create a tracklog for app usage (Log)
  - Extract position fixes (lat/lon/altitude/time)
  - Log header and task data 
- ⏳ Detect takeoff and landing
  - Very basic as it stands (average speed ~15km/h)
- ⏳ CLI tool
  - Currently just dump the tracklog as a json
- ⏳ Python bindings
  - As it stands the bare minumum to be usable by a logbook app
    Poor code and too many copies for now

## Build prerequisites

**Debian/Ubuntu:**
```bash
sudo apt install -y rust-all cargo rustc rust-clippy rustfmt build-essential 
```

## Usage example

Rustigc comes with an example which parse the igc content and dump
the tracklog as a JSON

```bash
cat track.igc | rustigc
```

## License

MIT
