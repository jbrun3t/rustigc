// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! XContest scoring rules (2026)
//! <https://www.xcontest.org/world/en/rules/>
//!
//! No minimum score. Gaps are charged in full, and a gap over 20 % of the distance does not close
//!
//! Both triangles come in two variants, on the gap as a share of the distance:
//!   - gap <= 5 %       => closed variant, better multiplier
//!   - 5 % < gap <= 20 % => open variant
//!
//! Three rules:
//!   - Free Flight: ×1.0
//!     - Polyline with 3 turnpoints
//!   - Free Triangle:
//!     - Closed circuit with 3 turnpoints
//!     - Open variant:   ×1.2
//!     - Closed variant: ×1.4
//!   - FAI Triangle:
//!     - Closed circuit with 3 turnpoints
//!     - Shortest side at least 28 % of total distance
//!     - Open variant:   ×1.4
//!     - Closed variant: ×1.6

use super::{
    BalancedCircuit, ClosedCircuit, Closing, League, Limit, OpenPolyline,
    RuleDescription, RuleGeometry, Ruleset, Variant, VariantKind,
};

pub struct Xcontest;

impl Xcontest {
    /// A circuit closing this tightly counts as closed and pays the better rate.
    const CLOSED: Closing = Closing::new(Limit::None, Limit::Ratio(0.05));
    /// Above `CLOSED` a circuit still counts, at the open rate, out to this share.
    const OPEN: Closing = Closing::new(Limit::None, Limit::Ratio(0.2));
}

impl League for Xcontest {
    const NAME: &'static str = "xcontest";
    const RULES: Ruleset = &[&FreeFlight, &FreeTriangle, &FaiTriangle];
}

#[derive(Debug)]
pub struct FreeFlight;

impl RuleGeometry for FreeFlight {
    type Shape = OpenPolyline<5>;
}

impl RuleDescription for FreeFlight {
    type League = Xcontest;

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "free flight",
            multiplier: 1.0,
            kind: VariantKind::Open,
        }]
    }
}

#[derive(Debug)]
pub struct FreeTriangle;

impl RuleGeometry for FreeTriangle {
    type Shape = ClosedCircuit<3>;
}

impl RuleDescription for FreeTriangle {
    type League = Xcontest;

    fn variants(&self) -> &'static [Variant] {
        &[
            Variant {
                name: "closed free triangle",
                multiplier: 1.4,
                kind: VariantKind::Closing(Xcontest::CLOSED),
            },
            Variant {
                name: "free triangle",
                multiplier: 1.2,
                kind: VariantKind::Closing(Xcontest::OPEN),
            },
        ]
    }
}

#[derive(Debug)]
pub struct FaiTriangle;

impl RuleGeometry for FaiTriangle {
    type Shape = BalancedCircuit<3, 280>;
}

impl RuleDescription for FaiTriangle {
    type League = Xcontest;

    fn variants(&self) -> &'static [Variant] {
        &[
            Variant {
                name: "closed fai triangle",
                multiplier: 1.6,
                kind: VariantKind::Closing(Xcontest::CLOSED),
            },
            Variant {
                name: "fai triangle",
                multiplier: 1.4,
                kind: VariantKind::Closing(Xcontest::OPEN),
            },
        ]
    }
}
