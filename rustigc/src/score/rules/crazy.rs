// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Crazy testing rules
//! These rules' sole purpose is to test the engine with different cardinalities.
//! Expect those rules to be computationally heavy.

use super::{
    ClosedCircuit, League, OpenPolyline, RuleDescription, RuleGeometry, Ruleset,
};

pub struct Crazy;

impl Crazy {
    const CLOSING_RATIO: f64 = 0.10;
}

impl League for Crazy {
    const NAME: &'static str = "misc4tp";
    const RULES: Ruleset = &[&FreeDistance4TP, &Quad];

    fn penalty(distance: f64, gap: f64) -> f64 {
        if gap <= (Self::CLOSING_RATIO * distance) {
            gap
        } else {
            f64::INFINITY
        }
    }
}

#[derive(Debug)]
pub struct FreeDistance4TP;

impl RuleGeometry for FreeDistance4TP {
    type Shape = OpenPolyline<6>;
}

impl RuleDescription for FreeDistance4TP {
    type League = Crazy;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.0, "Free Distance 4 Turnpoints")
    }
}

#[derive(Debug)]
pub struct Quad;

impl RuleGeometry for Quad {
    type Shape = ClosedCircuit<4>;
}

impl RuleDescription for Quad {
    type League = Crazy;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str) {
        (1.2, "Quadrilateral Circuit")
    }
}
