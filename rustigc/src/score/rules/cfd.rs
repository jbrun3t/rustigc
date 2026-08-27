// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! CFD scoring rules (FFVL distance competitions)
//! <https://parapente.ffvl.fr/sites/parapente.ffvl.fr/files/Reglement_competitions_CFD-2024_25vdef.pdf>
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
    BalancedCircuit, ClosedCircuit, Closing, League, Limit, OpenPolyline,
    RuleDescription, RuleGeometry, Ruleset,
};

pub struct Cfd;

impl Cfd {
    /// Least score that counts: 15 points
    const MIN_POINTS: f64 = 15000.0;
    /// 3 km free, then charged in full out to 5 % of the distance.
    const CLOSING: Closing = Closing::new(Limit::Fixed(3000.0), Limit::Ratio(0.05));
}

impl League for Cfd {
    const NAME: &'static str = "cfd";
    const RULES: Ruleset = &[&Distance3Points, &TrianglePlat, &TriangleFai];

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

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str, Closing) {
        (1.0, "distance 3 points", Closing::NONE)
    }
}

#[derive(Debug)]
pub struct TrianglePlat;

impl RuleGeometry for TrianglePlat {
    type Shape = ClosedCircuit<3>;
}

impl RuleDescription for TrianglePlat {
    type League = Cfd;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str, Closing) {
        (1.2, "triangle plat", Cfd::CLOSING)
    }
}

#[derive(Debug)]
pub struct TriangleFai;

impl RuleGeometry for TriangleFai {
    type Shape = BalancedCircuit<3, 280>;
}

impl RuleDescription for TriangleFai {
    type League = Cfd;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str, Closing) {
        (1.4, "triangle fai", Cfd::CLOSING)
    }
}
