// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Crazy testing rules
//! These rules' sole purpose is to test the engine with different cardinalities.
//! Expect those rules to be computationally heavy.

use super::{
    ClosedCircuit, Closing, League, Limit, OpenPolyline, RuleDescription, RuleGeometry,
    Ruleset,
};

pub struct Crazy;

impl Crazy {
    const CLOSING: Closing = Closing::new(Limit::None, Limit::Ratio(0.10));
}

impl League for Crazy {
    const NAME: &'static str = "misc4tp";
    const RULES: Ruleset = &[&FreeDistance4TP, &Quad];
}

#[derive(Debug)]
pub struct FreeDistance4TP;

impl RuleGeometry for FreeDistance4TP {
    type Shape = OpenPolyline<6>;
}

impl RuleDescription for FreeDistance4TP {
    type League = Crazy;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str, Closing) {
        (1.0, "free distance 4 turnpoints", Closing::NONE)
    }
}

#[derive(Debug)]
pub struct Quad;

impl RuleGeometry for Quad {
    type Shape = ClosedCircuit<4>;
}

impl RuleDescription for Quad {
    type League = Crazy;

    fn variant(&self, _distance: f64, _gap: f64) -> (f64, &'static str, Closing) {
        (1.2, "quadrilateral circuit", Crazy::CLOSING)
    }
}
