// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Scoring rules, read from JSON.
//!
//! A [`Ruleset`] lists [`Rule`]s; a rule scores one shape with variants, each a multiplier with
//! an optional closing and a list of constraints. The embedded rulesets are the files under
//! `rulesets/`.

mod closing;
mod constraint;
mod rule;
mod ruleset;
mod variant;

pub use rule::Rule;
pub use ruleset::{Ruleset, ruleset_names};
