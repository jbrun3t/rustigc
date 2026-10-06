// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use super::variant::{Scored, Variant, pick};
use crate::score::shapes::table::{self, ShapeDef};
use crate::score::shapes::{Shape, ShapeSpec};

/// One scoring rule of a ruleset: a geometry priced through one or more [`Variant`]s, listed best
/// rate first. [`pick`] takes the first that earns anything, so a rate that improves down the list
/// would never be reached.
#[derive(Debug)]
pub struct Rule {
    shape: &'static ShapeDef,
    variants: Vec<Variant>,
}

impl Rule {
    /// A rule over the geometry `spec` names.
    ///
    /// Refused when this build does not carry that geometry, or for what [`Rule::check`] refuses.
    pub(super) fn new(spec: ShapeSpec, variants: Vec<Variant>) -> Result<Self, String> {
        let shape =
            table::by_spec(spec).ok_or_else(|| format!("no such shape: {spec:?}"))?;
        let rule = Self { shape, variants };

        rule.check()?;
        Ok(rule)
    }

    /// What a rule's types cannot state: that it prices something, that its variants agree with
    /// its geometry on closing, and that they are listed best rate first.
    fn check(&self) -> Result<(), String> {
        let shape = self.shape.spec;

        if self.variants.is_empty() {
            return Err(format!("{shape:?} prices nothing"));
        }

        let mut previous = f64::INFINITY;
        for variant in &self.variants {
            // A gap priced over a geometry that has none would be charged nothing, whatever the
            // closing said, and a circuit left open would go free.
            if variant.closing.is_some() != self.shape.closes {
                return Err(format!(
                    "{:?} disagrees with {shape:?} on closing",
                    variant.name
                ));
            }

            // `pick` takes the first variant that earns anything, so a better rate further down
            // would never be reached and `score` would fall as distance grows.
            if variant.multiplier > previous {
                return Err(format!(
                    "{:?} pays more than the variant before it",
                    variant.name
                ));
            }
            previous = variant.multiplier;
        }

        Ok(())
    }

    /// Fix-index ranges the search branches on.
    pub(crate) fn cardinality(&self) -> usize {
        self.shape.cardinality
    }

    /// The geometry this rule scores over.
    pub(crate) fn shape(&self) -> &'static dyn Shape {
        self.shape.shape
    }

    /// Score in meters, unrounded — the search prunes on this.
    ///
    /// Non-decreasing in `distance` and non-increasing in `gap`, which is what makes the largest
    /// distance against the smallest gap an upper bound of the subtree. [`Rule::check`] holds the
    /// variants to the order that keeps it so.
    pub(crate) fn score(&self, distance: f64, gap: f64) -> f64 {
        self.scored(distance, gap).score
    }

    /// How the rule presents that score, once on the winning leaf.
    pub(crate) fn scored(&self, distance: f64, gap: f64) -> Scored<'_> {
        pick(&self.variants, distance, gap).scored(distance, gap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::rules::closing::Closing;

    /// A closing priced over an open shape would go uncharged.
    #[test]
    fn new_refuses_closing_on_open_shape() {
        let variants = vec![Variant {
            name: "free".into(),
            multiplier: 1.0,
            closing: Some(Closing::Ratio {
                ratio: 0.2,
                penalized: true,
            }),
            constraints: Vec::new(),
        }];

        let err = Rule::new(ShapeSpec::OpenPolyline { points: 3 }, variants).unwrap_err();
        assert!(err.contains("on closing"), "{err}");
    }
}
