// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Scoring pins: `real/<fixture>.xcontest.json` is the serde dump of `Log::score` under xcontest,
//! over the pin's own `takeoff`/`landing`, so a mismatch is a scoring change, not a detection one.
//! Re-bless with `rustigc-xc-score --format json` when the change is intended.

use std::fs;

use rustigc::{FlightDetection, FlightSelection, Log, Ruleset, Scorer, ScoringResult};
use rustigc_test_data::{for_each_fixture, real, stem_of};
use serde_json::Value;

/// `(takeoff, landing)` from the pin, or the detected flight when the pin is `null`.
fn window_of(expected: &Value, log: &Log) -> Option<(usize, usize)> {
    match (expected.get("takeoff"), expected.get("landing")) {
        (Some(t), Some(l)) => {
            Some((t.as_u64().unwrap() as usize, l.as_u64().unwrap() as usize))
        }
        _ => log.track.flights().longest().map(|f| (f.start, f.stop)),
    }
}

fn xcontest() -> &'static Ruleset {
    Ruleset::named("xcontest").unwrap()
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

    let result = window_of(&expected, &log).and_then(|(start, stop)| {
        log.score(xcontest(), start, stop).expect("scorable window")
    });
    let actual = serde_json::to_value(&result).unwrap();

    pretty_assertions::assert_eq!(actual, expected, "{stem}: xcontest result moved");
}

// free-04 and free-05 are near-straight tracks, the search's worst case: release builds only.
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

// Each `Scorer` entry point gives what `Log::score` gives.

/// triangle-01 as a `[latitude, longitude]` table, with what `Log::score` makes of it.
fn coord_table() -> (Vec<[f64; 2]>, ScoringResult) {
    let path = real().join("triangle-01.igc");
    let content = fs::read(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    let log = Log::new(&content).unwrap_or_else(|e| panic!("parse {path:?}: {e}"));

    let last = log.track.len() - 1;
    let expected = log
        .score(xcontest(), 0, last)
        .expect("scorable window")
        .expect("triangle-01 scores");
    let table = log.track.iter().map(|fix| [fix.lat, fix.lon]).collect();

    (table, expected)
}

#[test]
fn scorer_new_over_a_coord_table() {
    let (table, expected) = coord_table();
    let last = table.len() - 1;

    let result = Scorer::new(&table, 0, last).unwrap().solve(xcontest());

    assert_eq!(result.as_ref(), Some(&expected));
}

#[test]
fn scorer_from_slice_over_a_coord_table() {
    let (table, expected) = coord_table();

    let result = Scorer::from_slice(&table).unwrap().solve(xcontest());

    assert_eq!(result.as_ref(), Some(&expected));
}

#[test]
fn scorer_from_vec_over_a_coord_table() {
    let (table, expected) = coord_table();

    let result = Scorer::from_vec(table).unwrap().solve(xcontest());

    assert_eq!(result.as_ref(), Some(&expected));
}
