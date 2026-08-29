// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Scoring regression pins: `rustigc-test-data`'s `real/<fixture>.xcontest.json` holds the plain
//! serde dump of `log.score("xcontest", ..)`. The window scored is the reference's own `takeoff`/`landing`, so a
//! mismatch here means scoring math moved, not flight detection.
//! Re-bless with `rustigc-xc-score --format json` if the move was intended.

use std::fs;

use rustigc::{FlightDetection, FlightSelection, Log, Scorer, ScoringResult};
use rustigc_test_data::{for_each_fixture, real, stem_of};
use serde_json::Value;

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
    let igc_path = real().join(format!("{stem}.igc"));
    let content =
        fs::read(&igc_path).unwrap_or_else(|e| panic!("read {igc_path:?}: {e}"));
    let log = Log::new(&content).unwrap_or_else(|e| panic!("parse {igc_path:?}: {e}"));

    let ref_path = real().join(format!("{stem}.xcontest.json"));
    let expected_text = fs::read_to_string(&ref_path)
        .unwrap_or_else(|e| panic!("read {ref_path:?}: {e}"));
    let expected: Value = serde_json::from_str(&expected_text)
        .unwrap_or_else(|e| panic!("parse {ref_path:?}: {e}"));

    let result = window_of(&expected, &log)
        .map(|(start, stop)| log.score("xcontest", start, stop).expect("scorable window"))
        .flatten();
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

// A `Scorer` needs no `Log`. Check that each Scorer entry point yield the exact same
// result as a complete Log would.

/// triangle-01 as a `[latitude, longitude]` table, with what `Log::score` makes of it.
fn coord_table() -> (Vec<[f64; 2]>, ScoringResult) {
    let path = real().join("triangle-01.igc");
    let content = fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let log = Log::new(&content).unwrap_or_else(|e| panic!("parse {path:?}: {e}"));

    let last = log.track.len() - 1;
    let expected = log
        .score("xcontest", 0, last)
        .expect("scorable window")
        .expect("triangle-01 scores");
    let table = log.track.iter().map(|fix| [fix.lat, fix.lon]).collect();

    (table, expected)
}

#[test]
fn scorer_new_over_a_coord_table() {
    let (table, expected) = coord_table();
    let last = table.len() - 1;

    let result = Scorer::new(&table, 0, last)
        .unwrap()
        .solve("xcontest")
        .unwrap();

    assert_eq!(result.as_ref(), Some(&expected));
}

#[test]
fn scorer_from_slice_over_a_coord_table() {
    let (table, expected) = coord_table();

    let result = Scorer::from_slice(&table)
        .unwrap()
        .solve("xcontest")
        .unwrap();

    assert_eq!(result.as_ref(), Some(&expected));
}

#[test]
fn scorer_from_vec_over_a_coord_table() {
    let (table, expected) = coord_table();

    let result = Scorer::from_vec(table).unwrap().solve("xcontest").unwrap();

    assert_eq!(result.as_ref(), Some(&expected));
}
