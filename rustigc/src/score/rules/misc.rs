// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Single-rule leagues, each exercising one shape on its own. Their numbers are the [`League`]
//! defaults — nothing charged, no minimum — except for `Oar`.

use super::{
    ClosedCircuit, League, OpenPolyline, RuleDescription, RuleGeometry, Ruleset,
};

pub struct TwoTurnpoints;

impl League for TwoTurnpoints {
    const NAME: &'static str = "2tp";
    const RULES: Ruleset = &[&FreeDistance2];
}

#[derive(Debug)]
pub struct FreeDistance2;

impl RuleGeometry for FreeDistance2 {
    type Shape = OpenPolyline<4>;
}

impl RuleDescription for FreeDistance2 {
    type League = TwoTurnpoints;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.0, "2 Turnpoints Free Distance")
    }
}

pub struct OneTurnpoint;

impl League for OneTurnpoint {
    const NAME: &'static str = "1tp";
    const RULES: Ruleset = &[&FreeDistance1];
}

#[derive(Debug)]
pub struct FreeDistance1;

impl RuleGeometry for FreeDistance1 {
    type Shape = OpenPolyline<3>;
}

impl RuleDescription for FreeDistance1 {
    type League = OneTurnpoint;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.0, "1 Turnpoint Free Distance")
    }
}

pub struct Line;

impl League for Line {
    const NAME: &'static str = "line";
    const RULES: Ruleset = &[&StraightDistance];
}

/// The two furthest-apart fixes of the flight.
#[derive(Debug)]
pub struct StraightDistance;

impl RuleGeometry for StraightDistance {
    type Shape = OpenPolyline<2>;
}

impl RuleDescription for StraightDistance {
    type League = Line;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.0, "Straight Distance")
    }
}

pub struct Oar;

impl Oar {
    /// Largest gap that still closes the circuit, as a share of the distance.
    const CLOSING_RATIO: f64 = 0.10;
}

impl League for Oar {
    const NAME: &'static str = "oar";
    const RULES: Ruleset = &[&OutAndReturn];

    fn penalty(distance: f64, gap: f64) -> f64 {
        if gap <= (Self::CLOSING_RATIO * distance) {
            gap
        } else {
            f64::INFINITY
        }
    }
}

#[derive(Debug)]
pub struct OutAndReturn;

impl RuleGeometry for OutAndReturn {
    type Shape = ClosedCircuit<2>;
}

impl RuleDescription for OutAndReturn {
    type League = Oar;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.0, "Out And Return")
    }
}
