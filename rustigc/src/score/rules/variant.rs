// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use serde::Deserialize;

use super::closing::Closing;
use super::constraint::Constraint;

/// One way a rule scores its shape.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    /// Reported as the result's description.
    pub name: String,
    pub multiplier: f64,
    /// `None` for an open shape.
    #[serde(default)]
    pub closing: Option<Closing>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
}

impl Variant {
    /// Scores a raw `distance` and its closing `gap` at this variant's rate, leaving its closing reach
    /// and constraints unchecked.
    pub(super) fn rate(&self, distance: f64, gap: f64) -> f64 {
        let charge = self.closing.map_or(0.0, |c| c.charge(gap));
        (distance - charge).max(0.0) * self.multiplier
    }

    /// Scores a raw `distance` and its closing `gap`. The score is 0 when a constraint fails or
    /// the gap does not close.
    pub(super) fn scored(&self, distance: f64, gap: f64) -> Scored<'_> {
        let penalty = self.closing.map_or(0.0, |c| c.penalty(distance, gap));
        let net = (distance - penalty).max(0.0);

        let mut scored = Scored {
            distance: net,
            penalty,
            score: net * self.multiplier,
            multiplier: self.multiplier,
            threshold: self.closing.map_or(0.0, |c| c.reach(distance)),
            description: &self.name,
        };

        if !self.constraints.iter().all(|c| c.met(&scored)) {
            scored.score = 0.0;
        }

        scored
    }
}

/// The first variant that earns anything, or the last one when none does.
pub(super) fn pick(variants: &[Variant], distance: f64, gap: f64) -> &Variant {
    variants
        .iter()
        .find(|v| v.scored(distance, gap).score > 0.0)
        .unwrap_or_else(|| {
            variants
                .last()
                .expect("a rule always has at least one variant")
        })
}

/// A distance and gap scored by one variant, in meters, unrounded.
#[derive(Debug)]
pub struct Scored<'a> {
    /// Distance net of the penalty.
    pub distance: f64,
    /// Distance the gap took off.
    pub penalty: f64,
    /// `distance` × `multiplier`, or 0 when a constraint fails.
    pub score: f64,
    pub multiplier: f64,
    /// Largest gap the variant closes for the raw distance; 0 without a closing.
    pub threshold: f64,
    /// The variant's name.
    pub description: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A gap charged in full out to 800 m, the way XC League states its circuits.
    const CLOSING: Closing = Closing::Distance {
        distance_m: 800.0,
        penalized: true,
    };

    /// Two rates banded by scoring distance, as XC League bands its circuits: ×2 from 35 km,
    /// ×1.2 from 5 km.
    fn banded() -> Vec<Variant> {
        vec![
            Variant {
                name: "high".into(),
                multiplier: 2.0,
                closing: Some(CLOSING),
                constraints: vec![Constraint::MinDistance { meters: 35_000.0 }],
            },
            Variant {
                name: "low".into(),
                multiplier: 1.2,
                closing: Some(CLOSING),
                constraints: vec![Constraint::MinDistance { meters: 5_000.0 }],
            },
        ]
    }

    /// A distance threshold holds at exactly its distance; the same band in points does not, since
    /// `25_000.0 * 2.3` falls short of `57_500.0`.
    #[test]
    fn variants_min_distance_exact_at_stated_distance() {
        const RATE: f64 = 2.3;
        const { assert!(25_000.0 * RATE < 57_500.0, "the product is exact after all") };

        let by_distance = [Variant {
            name: "fai triangle".into(),
            multiplier: RATE,
            closing: None,
            constraints: vec![Constraint::MinDistance { meters: 25_000.0 }],
        }];
        let by_score = [Variant {
            name: "fai triangle".into(),
            multiplier: RATE,
            closing: None,
            constraints: vec![Constraint::MinScore { points: 57_500.0 }],
        }];

        assert_eq!(
            pick(&by_distance, 25_000.0, 0.0)
                .scored(25_000.0, 0.0)
                .score,
            25_000.0 * RATE
        );
        assert_eq!(
            pick(&by_score, 25_000.0, 0.0).scored(25_000.0, 0.0).score,
            0.0
        );
    }

    #[test]
    fn variants_band_by_distance() {
        let banded = banded();

        assert_eq!(pick(&banded, 40_000.0, 0.0).name, "high");
        assert_eq!(pick(&banded, 10_000.0, 0.0).name, "low");
    }

    /// A threshold is tested on the net distance: 35.5 km less an 800 m gap misses the 35 km band.
    #[test]
    fn variants_band_on_scoring_distance() {
        let banded = banded();

        assert_eq!(pick(&banded, 35_500.0, 800.0).name, "low");
        assert_eq!(pick(&banded, 35_500.0, 0.0).name, "high");
    }

    /// Below every band, nothing is earned.
    #[test]
    fn variants_band_below_floor_scores_zero() {
        let banded = banded();

        assert_eq!(pick(&banded, 4_000.0, 0.0).scored(4_000.0, 0.0).score, 0.0);
    }

    #[test]
    fn variants_band_ignores_unclosable_gap() {
        let banded = banded();

        assert_eq!(
            pick(&banded, 40_000.0, 900.0).scored(40_000.0, 900.0).score,
            0.0
        );
    }

    /// The score never falls as distance grows; the search's bound relies on it.
    #[test]
    fn variants_band_monotone_in_distance() {
        let banded = banded();
        let mut previous = 0.0;

        for km in 1..60 {
            let distance = f64::from(km) * 1000.0;
            let score = pick(&banded, distance, 0.0).scored(distance, 0.0).score;

            assert!(
                score >= previous,
                "score fell at {km} km: {score} < {previous}"
            );
            previous = score;
        }
    }

    /// Without constraints, a variant earns from the first metre.
    #[test]
    fn variants_unthresholded_earn_from_zero() {
        let free = [Variant {
            name: "free distance".into(),
            multiplier: 1.0,
            closing: None,
            constraints: Vec::new(),
        }];

        assert_eq!(pick(&free, 1.0, 0.0).scored(1.0, 0.0).score, 1.0);
    }
}
