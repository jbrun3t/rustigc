// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Tracklog scoring: what a flight is worth under the rules of a league.

mod cache;
mod engine;
mod rules;
mod shapes;

pub use rules::league_names;

pub use engine::Scorer;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Meters to kilometers, at the hundredth — the unit of every distance below but `raw_distance`.
fn round_km(meters: f64) -> f64 {
    (meters / 10.0).round() / 100.0
}

fn round_mm(meters: f64) -> f64 {
    (meters * 1000.0).round() / 1000.0
}

/// What the best rule scored, as that rule presents it.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct ScoringResult {
    /// Identity of the rule, e.g. `"Triangle plat"`.
    pub description: &'static str,
    /// Scored distance. Presentation defined by the rule.
    pub distance: f64,
    /// The same distance in meters, rounded to the nearest millimeter.
    pub raw_distance: f64,
    /// Closing leg of a circuit, 0 for an open polyline.
    pub gap: f64,
    /// What the rule charged for that gap.
    pub penalty: f64,
    /// Final score, in league points.
    pub score: f64,
    /// Multiplier the rule scored at.
    pub multiplier: f64,
    /// Start of the scoring window.
    pub takeoff: usize,
    /// Start fix of the task scored.
    pub entry: usize,
    /// Turnpoints of the task.
    pub turnpoints: Vec<usize>,
    /// Stop fix of the task scored.
    pub exit: usize,
    /// End of the scoring window.
    pub landing: usize,
    /// Whether the result closes on itself. Mostly to simplify presentation
    pub circuit: bool,
}
