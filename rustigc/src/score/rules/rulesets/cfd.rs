// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! CFD scoring rules (FFVL distance competitions)
//! <https://parapente.ffvl.fr/sites/parapente.ffvl.fr/files/Reglement_competitions_CFD-2024_25vdef.pdf>

use super::super::closing::Closing;
use super::super::constraint::Constraint;
use super::super::rule::Rule;
use super::super::ruleset::Ruleset;
use super::super::variant::Variant;
use crate::score::shapes::ShapeSpec;

/// 15 points.
const MIN_SCORE: Constraint = Constraint::MinScore { points: 15_000.0 };

/// A circuit closes free of charge out to 3 km, then charged in full out to 5 % of the distance:
/// one gap priced two ways, so two variants.
fn closed(name: &str, multiplier: f64) -> Vec<Variant> {
    [
        Closing::Distance {
            distance_m: 3000.0,
            penalized: false,
        },
        Closing::Ratio {
            ratio: 0.05,
            penalized: true,
        },
    ]
    .map(|closing| Variant {
        name: name.into(),
        multiplier,
        closing: Some(closing),
        constraints: vec![MIN_SCORE],
    })
    .into()
}

pub(super) fn cfd() -> Result<Ruleset, String> {
    Ruleset::new(
        "cfd",
        vec![
            Rule::new(
                ShapeSpec::OpenPolyline { points: 5 },
                vec![Variant {
                    name: "distance 3 points".into(),
                    multiplier: 1.0,
                    closing: None,
                    constraints: vec![MIN_SCORE],
                }],
            )?,
            Rule::new(
                ShapeSpec::ClosedCircuit { turnpoints: 3 },
                closed("triangle plat", 1.2),
            )?,
            Rule::new(
                ShapeSpec::BalancedCircuit {
                    turnpoints: 3,
                    min_side: 280,
                },
                closed("triangle fai", 1.4),
            )?,
        ],
    )
}
