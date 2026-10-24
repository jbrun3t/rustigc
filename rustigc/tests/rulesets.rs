// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! The samples under `rulesets/samples/` are not built in, so nothing else parses them.

use rustigc::Ruleset;

/// A sample parses as a caller's ruleset would.
fn check_sample(file: &str) {
    let path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/rulesets/samples/").to_owned() + file;
    let text = std::fs::read_to_string(&path).unwrap();
    Ruleset::from_json(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
}

macro_rules! sample_test {
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            check_sample($file);
        }
    };
}

sample_test!(sample_1tp, "1tp.json");
sample_test!(sample_2tp, "2tp.json");
sample_test!(sample_line, "line.json");
sample_test!(sample_out_and_return, "out-and-return.json");
