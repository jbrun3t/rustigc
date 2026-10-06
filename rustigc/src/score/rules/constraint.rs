// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use serde::Deserialize;

use super::variant::Scored;

/// A threshold a variant must meet to earn anything. A variant requires all of its constraints.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Constraint {
    /// Minimum score, on the metre scale a multiplier applies to: 15 points is `15000.0`.
    MinScore { points: f64 },
    /// Minimum scoring distance in meters, net of the closing penalty.
    ///
    /// For a rulebook stating a distance: the same band in points is distance × rate, and
    /// `25_000.0 * 2.3` is `57499.99999999999`, which refuses a 25.000 km triangle.
    MinDistance { meters: f64 },
}

impl Constraint {
    /// Whether `scored` meets this threshold.
    pub(super) fn met(&self, scored: &Scored) -> bool {
        match self {
            Constraint::MinScore { points } => scored.score >= *points,
            Constraint::MinDistance { meters } => scored.distance >= *meters,
        }
    }
}
