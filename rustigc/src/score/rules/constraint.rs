// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use serde::Deserialize;

use super::variant::Scored;

/// One threshold a variant requires.
///
/// Listed rather than combined into the kinds of variant that exist, so a further condition is one
/// more arm and never a product of the others. A variant requires all of them at once.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Constraint {
    /// Earns nothing below this score, in points on the metre scale a multiplier applies to,
    /// so CFD's 15 points is `15000.0`.
    MinScore { points: f64 },
    /// Earns nothing below this scoring distance, in meters net of the closing penalty.
    ///
    /// A threshold is stated in the unit its rulebook uses and never converted: the same one in
    /// points is the distance times the rate, and `25_000.0 * 2.3` is `57499.99999999999`, so a
    /// ×2.3 band written in points would refuse a triangle at exactly 25.000 km.
    // No ruleset in the tree states one yet; XC League is what it is here for.
    MinDistance { meters: f64 },
}

impl Constraint {
    /// Whether what a variant made of a geometry clears this constraint.
    pub(super) fn met(&self, scored: &Scored) -> bool {
        match self {
            Constraint::MinScore { points } => scored.score >= *points,
            Constraint::MinDistance { meters } => scored.distance >= *meters,
        }
    }
}
