// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! XContest scoring rules
//! <https://www.xcontest.org/world/en/rules/>

use super::super::closing::Closing;
use super::super::rule::Rule;
use super::super::ruleset::Ruleset;
use super::super::variant::Variant;
use crate::score::shapes::ShapeSpec;

/// A triangle closing within 5 % of its distance earns the closed rate, within 20 % the open one.
fn triangle(closed: (&str, f64), open: (&str, f64)) -> Vec<Variant> {
    [(closed, 0.05), (open, 0.2)]
        .map(|((name, multiplier), ratio)| Variant {
            name: name.into(),
            multiplier,
            closing: Some(Closing::Ratio {
                ratio,
                penalized: true,
            }),
            constraints: Vec::new(),
        })
        .into()
}

pub(super) fn xcontest() -> Result<Ruleset, String> {
    Ruleset::new(
        "xcontest",
        vec![
            Rule::new(
                ShapeSpec::OpenPolyline { points: 5 },
                vec![Variant {
                    name: "free flight".into(),
                    multiplier: 1.0,
                    closing: None,
                    constraints: Vec::new(),
                }],
            )?,
            Rule::new(
                ShapeSpec::ClosedCircuit { turnpoints: 3 },
                triangle(("closed free triangle", 1.4), ("free triangle", 1.2)),
            )?,
            Rule::new(
                ShapeSpec::BalancedCircuit {
                    turnpoints: 3,
                    min_side: 280,
                },
                triangle(("closed fai triangle", 1.6), ("fai triangle", 1.4)),
            )?,
        ],
    )
}
