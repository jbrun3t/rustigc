// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use serde::Deserialize;

/// How far a circuit may stay open. `penalized` takes the gap off the distance; otherwise the gap
/// is free.
///
/// A rulebook charging one gap two ways states two variants, each with its own closing.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Closing {
    /// Closes out to a gap of `distance_m` meters.
    Distance { distance_m: f64, penalized: bool },
    /// Closes out to a gap of `ratio` times the distance.
    Ratio { ratio: f64, penalized: bool },
}

impl Closing {
    /// Largest gap this closes, in meters, for the raw `distance`.
    pub(super) fn reach(&self, distance: f64) -> f64 {
        match self {
            Closing::Distance { distance_m, .. } => *distance_m,
            Closing::Ratio { ratio, .. } => ratio * distance,
        }
    }

    /// Distance the gap takes off within reach: the gap itself when penalized, else 0.
    pub(super) fn charge(&self, gap: f64) -> f64 {
        let (Closing::Distance { penalized, .. } | Closing::Ratio { penalized, .. }) =
            self;

        if *penalized { gap } else { 0.0 }
    }

    /// [`Closing::charge`], INFINITY past [`Closing::reach`].
    pub(super) fn penalty(&self, distance: f64, gap: f64) -> f64 {
        if gap > self.reach(distance) {
            f64::INFINITY
        } else {
            self.charge(gap)
        }
    }
}
