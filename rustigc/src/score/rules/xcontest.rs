// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! XContest scoring rules (2026)
//! https://www.xcontest.org/world/en/rules/
//!
//! No minimum score. Gaps are charged in full, and a gap over 20 % of the distance does not close
//!
//! Both triangles come in two variants, on the gap as a share of the distance:
//!   - gap < 5 %        => closed variant, better multiplier
//!   - 5 % <= gap <= 20 % => open variant
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
    BalancedCircuit, ClosedCircuit, League, OpenPolyline, RuleDescription, RuleGeometry,
    Ruleset,
};

pub struct Xcontest;

impl Xcontest {
    /// Largest gap that still closes a circuit, as a share of the distance.
    const CLOSING_RATIO: f64 = 0.2;
    /// Below this share, the circuit counts as closed and pays the better rate.
    const CLOSED_RATIO: f64 = 0.05;

    /// Picks a rule's closed variant over its open one.
    fn closed_variant(
        distance: f64,
        gap: f64,
        open: (f64, &'static str),
        closed: (f64, &'static str),
    ) -> (f64, &'static str) {
        if gap < Self::CLOSED_RATIO * distance {
            closed
        } else {
            open
        }
    }
}

impl League for Xcontest {
    const NAME: &'static str = "xcontest";
    const RULES: Ruleset = &[&FreeFlight, &FreeTriangle, &FaiTriangle];

    fn penalty(distance: f64, gap: f64) -> f64 {
        if gap <= (Self::CLOSING_RATIO * distance) {
            gap
        } else {
            f64::INFINITY
        }
    }
}

#[derive(Debug)]
pub struct FreeFlight;

impl RuleGeometry for FreeFlight {
    type Shape = OpenPolyline<5>;
}

impl RuleDescription for FreeFlight {
    type League = Xcontest;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.0, "Free Flight")
    }
}

#[derive(Debug)]
pub struct FreeTriangle;

impl RuleGeometry for FreeTriangle {
    type Shape = ClosedCircuit<3>;
}

impl RuleDescription for FreeTriangle {
    type League = Xcontest;

    fn variant(&self, distance: f64, gap: f64) -> (f64, &'static str) {
        Xcontest::closed_variant(
            distance,
            gap,
            (1.2, "Free Triangle"),
            (1.4, "Closed Free Triangle"),
        )
    }
}

#[derive(Debug)]
pub struct FaiTriangle;

impl RuleGeometry for FaiTriangle {
    type Shape = BalancedCircuit<3, 280>;
}

impl RuleDescription for FaiTriangle {
    type League = Xcontest;

    fn variant(&self, distance: f64, gap: f64) -> (f64, &'static str) {
        Xcontest::closed_variant(
            distance,
            gap,
            (1.4, "FAI Triangle"),
            (1.6, "Closed FAI Triangle"),
        )
    }
}
