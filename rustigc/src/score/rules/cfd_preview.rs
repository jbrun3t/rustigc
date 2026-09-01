// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! CFD scoring rules Preview
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
//!   - Quadrilatère: ×1.1
//!     - Closed circuit with 4 turnpoints

use super::{
    BalancedCircuit, ClosedCircuit, Closing, League, Limit, OpenPolyline,
    RuleDescription, RuleGeometry, Ruleset, Variant, VariantKind,
};

pub struct CfdPreview;

impl CfdPreview {
    /// Least score that counts: 15 points
    const MIN_POINTS: f64 = 15000.0;
    /// 3 km free, then charged in full out to 5 % of the distance.
    const CLOSING: Closing = Closing::new(Limit::Fixed(3000.0), Limit::Ratio(0.05));
}

impl League for CfdPreview {
    const NAME: &'static str = "cfd preview";
    const RULES: Ruleset =
        &[&Distance3Points, &TrianglePlat, &TriangleFai, &Quadrilatere];

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
    type League = CfdPreview;

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "distance 3 points",
            multiplier: 1.0,
            kind: VariantKind::Open,
        }]
    }
}

#[derive(Debug)]
pub struct TrianglePlat;

impl RuleGeometry for TrianglePlat {
    type Shape = ClosedCircuit<3>;
}

impl RuleDescription for TrianglePlat {
    type League = CfdPreview;

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "triangle plat",
            multiplier: 1.2,
            kind: VariantKind::Closing(CfdPreview::CLOSING),
        }]
    }
}

#[derive(Debug)]
pub struct TriangleFai;

impl RuleGeometry for TriangleFai {
    type Shape = BalancedCircuit<3, 280>;
}

impl RuleDescription for TriangleFai {
    type League = CfdPreview;

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "triangle fai",
            multiplier: 1.4,
            kind: VariantKind::Closing(CfdPreview::CLOSING),
        }]
    }
}

#[derive(Debug)]
pub struct Quadrilatere;

impl RuleGeometry for Quadrilatere {
    type Shape = ClosedCircuit<4>;
}

impl RuleDescription for Quadrilatere {
    type League = CfdPreview;

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "quadrilatère",
            multiplier: 1.1,
            kind: VariantKind::Closing(CfdPreview::CLOSING),
        }]
    }
}
