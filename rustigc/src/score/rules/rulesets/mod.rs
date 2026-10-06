// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! The embedded rulesets, one file per rulebook.
//!
//! A ruleset's numbers belong in its file and nowhere else.

mod cfd;
mod misc;
mod xcontest;

use super::ruleset::Ruleset;

/// Builds one ruleset, or says what is wrong with it.
type Build = fn() -> Result<Ruleset, String>;

/// Builds each embedded ruleset, paired with its file for the panics `load` raises, in the order the
/// CLI shows them.
pub(super) static RULESET_FILES: &[(&str, Build)] = &[
    ("cfd.rs", cfd::cfd),
    ("xcontest.rs", xcontest::xcontest),
    ("misc.rs", misc::one_turnpoint),
    ("misc.rs", misc::two_turnpoints),
    ("misc.rs", misc::line),
    ("misc.rs", misc::out_and_return),
];
