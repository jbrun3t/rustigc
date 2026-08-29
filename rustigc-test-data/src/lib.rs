// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! IGC fixtures for the rustigc test suites, and the list of which of them the corpus tests sweep.

use std::path::PathBuf;

/// Corpus root.
pub fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Directory of the real-flight fixtures and their reference pins.
pub fn real() -> PathBuf {
    dir().join("real")
}

/// The file stem a fixture's identifier names: `problem_small_gap` is `problem-small-gap.igc`.
pub fn stem_of(name: &str) -> String {
    name.replace('_', "-")
}

/// The fixtures every corpus sweep covers, applied to a caller-supplied macro taking one
/// identifier.
#[macro_export]
macro_rules! for_each_fixture {
    ($test:ident) => {
        $test!(fai_01);
        $test!(fai_02);
        $test!(fai_03);
        $test!(fai_04);
        $test!(fai_05);

        $test!(free_01);
        $test!(free_02);
        $test!(free_03);
        $test!(free_04);
        $test!(free_05);
        $test!(free_06);
        $test!(free_07);
        $test!(free_08);
        $test!(free_09);

        $test!(triangle_01);
        $test!(triangle_02);
        $test!(triangle_03);
        $test!(triangle_04);
        $test!(triangle_05);
        $test!(triangle_06);

        $test!(problem_antimeridian);
        $test!(problem_detect_cadence);
        $test!(problem_duplicate_fixes);
        $test!(problem_fai_limit);
        $test!(problem_small_gap);
        $test!(problem_time_gaps);
    };
}
