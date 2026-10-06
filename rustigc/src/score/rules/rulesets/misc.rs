// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Single-rule rulesets, each exercising one shape on its own.

use super::super::closing::Closing;
use super::super::rule::Rule;
use super::super::ruleset::Ruleset;
use super::super::variant::Variant;
use crate::score::shapes::ShapeSpec;

/// A ruleset of one rule, priced at ×1 through one variant.
fn single(
    ruleset: &str,
    shape: ShapeSpec,
    name: &str,
    closing: Option<Closing>,
) -> Result<Ruleset, String> {
    let variant = Variant {
        name: name.into(),
        multiplier: 1.0,
        closing,
        constraints: Vec::new(),
    };

    Ruleset::new(ruleset, vec![Rule::new(shape, vec![variant])?])
}

pub(super) fn one_turnpoint() -> Result<Ruleset, String> {
    single(
        "1tp",
        ShapeSpec::OpenPolyline { points: 3 },
        "1 turnpoint free distance",
        None,
    )
}

pub(super) fn two_turnpoints() -> Result<Ruleset, String> {
    single(
        "2tp",
        ShapeSpec::OpenPolyline { points: 4 },
        "2 turnpoints free distance",
        None,
    )
}

pub(super) fn line() -> Result<Ruleset, String> {
    single(
        "line",
        ShapeSpec::OpenPolyline { points: 2 },
        "straight distance",
        None,
    )
}

pub(super) fn out_and_return() -> Result<Ruleset, String> {
    single(
        "out-and-return",
        ShapeSpec::ClosedCircuit { turnpoints: 2 },
        "out and return",
        Some(Closing::Ratio {
            ratio: 0.1,
            penalized: true,
        }),
    )
}
