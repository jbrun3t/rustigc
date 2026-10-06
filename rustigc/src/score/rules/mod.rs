// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Rules: what a ruleset scores, and over which shape.
//!
//! One file per embedded ruleset, under `rulesets/`. A ruleset's numbers belong in that file and
//! nowhere else.

mod closing;
mod constraint;
mod rule;
mod ruleset;
mod rulesets;
mod variant;

pub use rule::Rule;
pub use ruleset::{Ruleset, ruleset_names};
