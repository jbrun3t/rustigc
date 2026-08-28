// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Integration tests with real IGC files

mod common;

use rustigc::{FlightDetection, FlightSelection, Log};

use common::{for_each_fixture, stem_of};
use std::fs;
use std::path::PathBuf;

/// Get path to test fixture
fn fixture_test_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test_data")
}

fn fixture_parse_file(path: &PathBuf) -> Log {
    let content =
        fs::read(path).unwrap_or_else(|e| panic!("Failed to read {:?}: {}", *path, e));

    Log::new(&content).unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", *path, e))
}

/// Test parsing a test file
#[test]
fn test_parse_plouf() {
    let path = fixture_test_dir().join("real/free-06.igc");
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
}

/// `Log::track` promises strictly increasing timestamps
fn assert_increasing_timestamps(log: &Log, path: &PathBuf) {
    for i in 1..log.track.len() {
        assert!(
            log.track[i].timestamp > log.track[i - 1].timestamp,
            "{path:?}: timestamps not increasing at index {i}"
        );
    }
}

/// `problem-time-gaps.igc` has 3 big gaps, check we identify them correctly
#[test]
fn flight_window_stops_at_time_gap() {
    let path = fixture_test_dir().join("real/problem-time-gaps.igc");
    let log = fixture_parse_file(&path);

    let flights = log.track.flights();
    let flight = flights.longest().expect("a flight is detected");

    assert!(
        flight.stop <= 3103,
        "window [{}, {}] reaches past the gap",
        flight.start,
        flight.stop
    );
}

/// fai-01 is the fixture the bindings and their READMEs quote, so pin what they quote.
#[test]
fn flight_window_of_fai_01() {
    let path = fixture_test_dir().join("real/fai-01.igc");
    let log = fixture_parse_file(&path);

    assert_eq!(log.track.len(), 25459);
    assert_eq!(log.headers["PLT"].text, "Mike Young");
    assert_eq!(log.headers["GTY"].text, "Ventus 3T");

    let flights = log.track.flights();
    let flight = flights.longest().expect("a flight is detected");

    assert_eq!((flight.start, flight.stop), (125, 25425));
}

fn check_fixture(name: &str) {
    let stem = stem_of(name);
    let path = fixture_test_dir().join("real").join(format!("{stem}.igc"));
    let log = fixture_parse_file(&path);
    assert!(!log.track.is_empty());
    assert_increasing_timestamps(&log, &path);
}

macro_rules! parse_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            check_fixture(stringify!($name));
        }
    };
}

for_each_fixture!(parse_test);
