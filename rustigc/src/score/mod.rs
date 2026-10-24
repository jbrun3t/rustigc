// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Tracklog scoring: what a flight is worth under a ruleset.

mod cache;
mod engine;
mod rules;
mod shapes;

pub use rules::{Ruleset, preset_names};

pub use engine::Scorer;

use serde::{Deserialize, Serialize};

pub(crate) fn round_mm(meters: f64) -> f64 {
    (meters * 1000.0).round() / 1000.0
}

/// The winning rule's result.
///
/// Every fix is an index into the track that was scored, not into the window. Kilometre values
/// and `score` are rounded to two decimals, metre values to the millimetre.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ScoringResult {
    /// Name of the scoring ruleset, e.g. `"xcontest"`.
    pub ruleset: String,
    /// Name of the winning variant, e.g. `"triangle plat"`.
    pub description: String,
    /// Scored distance in meters, net of the penalty.
    pub distance_m: f64,
    /// Closing leg of a circuit, in metres; 0 for an open polyline.
    pub gap_m: f64,
    /// Largest gap the winning variant closes, in meters; 0 for an open task.
    pub threshold_m: f64,
    /// Scored distance in kilometers.
    pub distance_km: f64,
    /// Final score, in the ruleset's points.
    pub score: f64,
    /// What the rule charged for that gap, in kilometers off the distance.
    pub penalty: f64,
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
    /// Whether the task closes on itself.
    pub circuit: bool,
}
