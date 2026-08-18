// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Corpus fixture list shared by `tests/scoring.rs` and `tests/parsing.rs`, so the two test files
//! can't drift apart on which fixtures under `test_data/real/` they check.

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

        $test!(problem_detect_cadence);
        $test!(problem_duplicate_fixes);
        $test!(problem_fai_limit);
        $test!(problem_small_gap);
        $test!(problem_time_gaps);
    };
}
pub(crate) use for_each_fixture;

pub(crate) fn stem_of(name: &str) -> String {
    name.replace('_', "-")
}
