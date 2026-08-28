// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Single-rule leagues, each exercising one shape on its own. Their numbers are the [`League`]
//! defaults — nothing charged, no minimum — except for `Oar`.

use super::{
    ClosedCircuit, Closing, League, Limit, OpenPolyline, RuleDescription, RuleGeometry,
    Ruleset, Variant, VariantKind,
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

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "2 turnpoints free distance",
            multiplier: 1.0,
            kind: VariantKind::Open,
        }]
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

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "1 turnpoint free distance",
            multiplier: 1.0,
            kind: VariantKind::Open,
        }]
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

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "straight distance",
            multiplier: 1.0,
            kind: VariantKind::Open,
        }]
    }
}

pub struct Oar;

impl Oar {
    /// Charged in full out to 10 % of the distance.
    const CLOSING: Closing = Closing::new(Limit::None, Limit::Ratio(0.10));
}

impl League for Oar {
    const NAME: &'static str = "oar";
    const RULES: Ruleset = &[&OutAndReturn];
}

#[derive(Debug)]
pub struct OutAndReturn;

impl RuleGeometry for OutAndReturn {
    type Shape = ClosedCircuit<2>;
}

impl RuleDescription for OutAndReturn {
    type League = Oar;

    fn variants(&self) -> &'static [Variant] {
        &[Variant {
            name: "out and return",
            multiplier: 1.0,
            kind: VariantKind::Closing(Oar::CLOSING),
        }]
    }
}
