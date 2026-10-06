// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use serde::Deserialize;

/// How far a circuit may stay open, and whether the gap is charged.
///
/// One closing answers both what a gap costs and how large it may get, so a charged penalty and a
/// reported threshold cannot drift apart. A rulebook pricing one gap two ways states two variants.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Closing {
    /// Closes out to a gap of `distance_m` meters.
    Distance { distance_m: f64, penalized: bool },
    /// Closes out to a gap of `ratio` times the distance.
    Ratio { ratio: f64, penalized: bool },
}

impl Closing {
    /// Largest gap this closes, in meters, against the raw distance.
    pub(super) fn reach(&self, distance: f64) -> f64 {
        match self {
            Closing::Distance { distance_m, .. } => *distance_m,
            Closing::Ratio { ratio, .. } => ratio * distance,
        }
    }

    /// Charge for the gap: its own length when penalized, else nothing. INFINITY for a gap this does
    /// not reach.
    pub(super) fn penalty(&self, distance: f64, gap: f64) -> f64 {
        let (Closing::Distance { penalized, .. } | Closing::Ratio { penalized, .. }) =
            self;

        if gap > self.reach(distance) {
            f64::INFINITY
        } else if *penalized {
            gap
        } else {
            0.0
        }
    }
}
