// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Scoring regression pins: `test_data/real/<fixture>.xcontest.json` holds the plain serde dump of
//! `log.score("xcontest", ..)`. The window scored is the reference's own `takeoff`/`landing`, so a
//! mismatch here means scoring math moved, not flight detection.
//! Re-bless with `rustigc-xc-score --format json` if the move was intended.

mod common;

use std::fs;
use std::path::PathBuf;

use common::{for_each_fixture, stem_of};
use rustigc::{FlightDetection, FlightSelection, Log};
use serde_json::Value;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test_data/real")
}

/// `(takeoff, landing)` from the reference's own fields, when it has any — a blessed "no XC" carries
/// no window, so falling back to our own detection is the best this can do for that case.
fn window_of(expected: &Value, log: &Log) -> Option<(usize, usize)> {
    match (expected.get("takeoff"), expected.get("landing")) {
        (Some(t), Some(l)) => {
            Some((t.as_u64().unwrap() as usize, l.as_u64().unwrap() as usize))
        }
        _ => log.track.flights().longest().map(|f| (f.start, f.stop)),
    }
}

fn check_fixture(name: &str) {
    let stem = stem_of(name);
    let igc_path = corpus_dir().join(format!("{stem}.igc"));
    let content =
        fs::read(&igc_path).unwrap_or_else(|e| panic!("read {igc_path:?}: {e}"));
    let log = Log::new(&content).unwrap_or_else(|e| panic!("parse {igc_path:?}: {e}"));

    let ref_path = corpus_dir().join(format!("{stem}.xcontest.json"));
    let expected_text = fs::read_to_string(&ref_path)
        .unwrap_or_else(|e| panic!("read {ref_path:?}: {e}"));
    let expected: Value = serde_json::from_str(&expected_text)
        .unwrap_or_else(|e| panic!("parse {ref_path:?}: {e}"));

    let result = window_of(&expected, &log)
        .and_then(|(start, stop)| log.score("xcontest", start, stop));
    let actual = serde_json::to_value(&result).unwrap();

    pretty_assertions::assert_eq!(actual, expected, "{stem}: xcontest result moved");
}

// free-04 and free-05 are near-straight tracks, worst case for the search — way too slow in debug
// mode. Everything else runs there too.
macro_rules! xcontest_test {
    (free_04) => {
        xcontest_test!(@slow free_04);
    };
    (free_05) => {
        xcontest_test!(@slow free_05);
    };
    (@slow $name:ident) => {
        #[test]
        #[cfg_attr(debug_assertions, ignore = "too slow in debug mode, run --release")]
        fn $name() {
            check_fixture(stringify!($name));
        }
    };
    ($name:ident) => {
        #[test]
        fn $name() {
            check_fixture(stringify!($name));
        }
    };
}

for_each_fixture!(xcontest_test);
