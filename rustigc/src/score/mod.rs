// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Tracklog scoring: what a flight is worth under the rules of a league.

mod cache;
mod engine;
mod rules;
mod shapes;

#[cfg(feature = "geojson")]
pub(crate) use rules::known_league;
pub use rules::league_names;

pub use engine::Scorer;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

fn round_mm(meters: f64) -> f64 {
    (meters * 1000.0).round() / 1000.0
}

/// What the winning rule scored, as that rule presents it.
///
/// Every fix here is an index into the track that was scored, not into the window. `distance_km`
/// and `score` are rounded the way the league publishes them; `distance_m` keeps the metres.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ScoringResult {
    /// Identity of the scoring league, e.g. "xcontest"
    pub league: String,
    /// Identity of the rule, e.g. `"triangle plat"`.
    pub description: String,
    /// Scored distance in meters, rounded to the nearest millimeter.
    pub distance_m: f64,
    /// Closing leg of a circuit, in meteres; 0 for an open polyline.
    pub gap_m: f64,
    /// Largest gap the reported rule and variant would still hold at, in meters.
    pub threshold_m: f64,
    /// Distance in kilometers, as the rule presents it.
    pub distance_km: f64,
    /// Final score, in league points.
    pub score: f64,
    /// What the rule charged for that gap, in point.
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
