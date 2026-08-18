// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! CFD scoring rules (FFVL distance competitions)
//! https://parapente.ffvl.fr/sites/parapente.ffvl.fr/files/Reglement_competitions_CFD-2024_25vdef.pdf
//!
//! Minimum score: 15 points.
//!
//! Closing conditions, on the gap:
//!   - gap <= 3 km                        => no penalty
//!   - 3 km < gap <= 5 % of the distance  => penalty = gap
//!   - beyond that                        => does not close
//!
//! Three rules:
//!   - Free Flight (aka Distance 3 Points): ×1.0
//!     - Polyline with 3 turnpoints
//!   - Free Triangle (Triangle Plat): ×1.2
//!     - Closed circuit with 3 turnpoints
//!   - FAI Triangle: ×1.4
//!     - Closed circuit with 3 turnpoints
//!     - Shortest side at least 28 % of total distance

use super::{
    BalancedCircuit, ClosedCircuit, League, OpenPolyline, RuleDescription, RuleGeometry,
    Ruleset,
};

pub struct Cfd;

impl Cfd {
    /// Least score that counts: 15 points
    const MIN_POINTS: f64 = 15000.0;
    /// Gap allowed free of charge, in meters.
    const CLOSING_FIXED: f64 = 3000.0;
    /// Largest gap that still closes a circuit, as a share of the distance. Charged in full.
    const CLOSING_RATIO: f64 = 0.05;
}

impl League for Cfd {
    const NAME: &'static str = "cfd";
    const RULES: Ruleset = &[&Distance3Points, &TrianglePlat, &TriangleFai];

    fn penalty(distance: f64, gap: f64) -> f64 {
        if gap <= Self::CLOSING_FIXED {
            0.0
        } else if gap <= (Self::CLOSING_RATIO * distance) {
            gap
        } else {
            f64::INFINITY
        }
    }

    fn minimum() -> f64 {
        Self::MIN_POINTS
    }
}

#[derive(Debug)]
pub struct Distance3Points;

impl RuleGeometry for Distance3Points {
    type Shape = OpenPolyline<5>;
}

impl RuleDescription for Distance3Points {
    type League = Cfd;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.0, "Distance 3 points")
    }
}

#[derive(Debug)]
pub struct TrianglePlat;

impl RuleGeometry for TrianglePlat {
    type Shape = ClosedCircuit<3>;
}

impl RuleDescription for TrianglePlat {
    type League = Cfd;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.2, "Triangle plat")
    }
}

#[derive(Debug)]
pub struct TriangleFai;

impl RuleGeometry for TriangleFai {
    type Shape = BalancedCircuit<3, 280>;
}

impl RuleDescription for TriangleFai {
    type League = Cfd;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.4, "Triangle FAI")
    }
}
