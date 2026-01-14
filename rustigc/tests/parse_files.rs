//! Integration tests with real IGC files

use rustigc::FRawData;
use rustigc::Log;

use std::fs;
use std::path::PathBuf;

/// Get path to test fixture
fn fixture_test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test_data")
}

fn fixture_parse_file(path: &PathBuf) -> Log {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", *path, e));

    let log = Log::new(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", *path, e));

    return log;
}

/// Test parsing a test file
#[test]
fn test_parse_plouf() {
    let path = fixture_test_dir().join("real/plouf-01.igc");
    let log = fixture_parse_file(&path);

    assert_eq!(log.track.len(), 1095);
    assert_eq!(log.recorder.manufacturer, "XTR");
    assert_eq!(log.headers["PLT"].text, "Jerome Brunet");
    assert_eq!(log.headers["GTY"].text, "Ozone Delta 4 MS");
    assert_eq!(log.headers["DTE"].text, "201024");

    // Verify fix coordinates are reasonable
    for (i, fix) in log.track.iter().enumerate() {
        assert!(fix.lat.abs() <= 90.0, "Invalid latitude at fix {}", i);
        assert!(fix.lon.abs() <= 180.0, "Invalid longitude at fix {}", i);
    }

    // Check that timestamps are monotonically increasing or equal
    for i in 1..log.track.len() {
        assert!(
            log.track[i].timestamp >= log.track[i - 1].timestamp,
            "Timestamps not in order at index {}",
            i
        );
    }
}

/// Test all files can be parsed without errors
#[test]
fn test_parse_all_files() {
    let dir = fixture_test_dir().join("real/");

    println!("Checking directory {:?}", dir);

    for entry in
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("Cannot read {:?}: {}", dir, e))
    {
        let entry = entry
            .as_ref()
            .unwrap_or_else(|e| panic!("Cannot read {:?}: {}", entry, e));
        let path = entry.path();

        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase() != "igc")
            .unwrap_or(false)
        {
            continue;
        }

        // Just check we can parse all of them
        println!("Parsing {:?} ...", path);
        let log = fixture_parse_file(&path);
        let data = FRawData::new(&log);
        if let Some((start, stop)) = data.phases() {
            eprintln!("Flight {} -> {}", start, stop);
        } else {
            eprintln!("No takeoff found");
        }
        assert!(log.track.len() > 0)
    }
}
