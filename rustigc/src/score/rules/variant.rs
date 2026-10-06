// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use super::closing::Closing;
use super::constraint::Constraint;

/// One way a rule prices a geometry: what it requires, and what the price is worth.
#[derive(Debug)]
pub struct Variant {
    pub name: String,
    pub multiplier: f64,
    /// `None` over an open geometry, which has no closing leg and reports no gap.
    pub closing: Option<Closing>,
    pub constraints: Vec<Constraint>,
}

impl Variant {
    /// What this rate makes of a geometry; its score is 0 unless every constraint holds.
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

/// The variant a rule prices a geometry at: the first, listed best rate first, that earns anything
/// — the loosest one if none do, which then scores 0.
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

/// What a rule made of a geometry, in meters, unrounded.
///
/// Borrows its description from the rule that produced it, which owns the name its ruleset states.
#[derive(Debug)]
pub struct Scored<'a> {
    /// Distance net of the penalty.
    pub distance: f64,
    pub penalty: f64,
    /// 0 when a constraint is not met: there is a path, it is worth nothing.
    pub score: f64,
    pub multiplier: f64,
    /// Largest gap this result's variant would still hold at, on the raw distance; 0 when there is
    /// no closing leg.
    pub threshold: f64,
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

    /// One closing paying two rates by distance, best first, as XC League bands its circuits:
    /// ×2 from 35 km of scoring distance, ×1.2 from 5 km.
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

    /// A distance threshold holds at exactly the distance its rulebook states. The same threshold
    /// in points cannot: it is that distance times the rate, and the product falls short of the
    /// round number, so a ×2.3 band written in points refuses a 25.000 km triangle.
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

    /// A threshold is tested on the distance the variant scores, net of what it charges for the
    /// gap: 35.5 km raw, 800 m of it charged, misses a band stated at 35 km.
    #[test]
    fn variants_band_on_scoring_distance() {
        let banded = banded();

        assert_eq!(pick(&banded, 35_500.0, 800.0).name, "low");
        assert_eq!(pick(&banded, 35_500.0, 0.0).name, "high");
    }

    /// Under every band a rule earns nothing, so the search prunes it rather than reporting a path.
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

    /// Thresholds step the score up, never down. A band paying a worse rate as distance grows would
    /// break the bound the search prunes on, with nothing to report it.
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

    /// A variant stating no threshold earns from the first metre.
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
