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
    RuleDescription, RuleGeometry, Ruleset,
};

pub struct Xcontest;

impl Xcontest {
    /// A circuit closing this tightly counts as closed and pays the better rate.
    const CLOSED: Closing = Closing::new(Limit::None, Limit::Ratio(0.05));
    /// Above `CLOSED` a circuit still counts, at the open rate, out to this share.
    const OPEN: Closing = Closing::new(Limit::None, Limit::Ratio(0.2));

    /// Picks a rule's closed variant over its open one. Both halves of a variant come out of this
    /// one test, so the rate a rule reports and the threshold it reports cannot disagree.
    fn closed_variant<T>(distance: f64, gap: f64, closed: T, open: T) -> T {
        if gap <= Self::CLOSED.limit(distance) {
            closed
        } else {
            open
        }
    }
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

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str, Closing) {
        (1.0, "free flight", Closing::NONE)
    }
}

#[derive(Debug)]
pub struct FreeTriangle;

impl RuleGeometry for FreeTriangle {
    type Shape = ClosedCircuit<3>;
}

impl RuleDescription for FreeTriangle {
    type League = Xcontest;

    fn variant(&self, distance: f64, gap: f64) -> (f64, &'static str, Closing) {
        Xcontest::closed_variant(
            distance,
            gap,
            (1.4, "closed free triangle", Xcontest::CLOSED),
            (1.2, "free triangle", Xcontest::OPEN),
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

    fn variant(&self, distance: f64, gap: f64) -> (f64, &'static str, Closing) {
        Xcontest::closed_variant(
            distance,
            gap,
            (1.6, "closed fai triangle", Xcontest::CLOSED),
            (1.4, "fai triangle", Xcontest::OPEN),
        )
    }
}
