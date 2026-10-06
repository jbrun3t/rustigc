// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use serde::Deserialize;

use super::variant::{Scored, Variant, pick};
use crate::score::shapes::Shape;
use crate::score::shapes::table::{self, ShapeDef};

/// A shape and the [`Variant`]s it is scored with, listed best rate first.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Looked up in the shape table as the file is read; an unknown shape is a parse error.
    #[serde(deserialize_with = "shape_def")]
    shape: &'static ShapeDef,
    variants: Vec<Variant>,
}

/// Deserializes a [`ShapeSpec`] into the shape table entry it names.
///
/// [`ShapeSpec`]: crate::score::shapes::ShapeSpec
fn shape_def<'de, D>(deserializer: D) -> Result<&'static ShapeDef, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let spec = Deserialize::deserialize(deserializer)?;
    table::by_spec(spec)
        .ok_or_else(|| D::Error::custom(format!("no such shape: {spec:?}")))
}

impl Rule {
    /// Refuses a rule with no variant, a variant whose closing disagrees with the shape, or a
    /// multiplier rising down the list.
    pub(super) fn check(&self) -> Result<(), String> {
        let shape = self.shape.spec;

        if self.variants.is_empty() {
            return Err(format!("{shape:?} has no variant"));
        }

        let mut previous = f64::INFINITY;
        for variant in &self.variants {
            // An open shape reports no gap to charge, and a circuit without a closing would close
            // for free.
            if variant.closing.is_some() != self.shape.closes {
                return Err(format!(
                    "{:?} disagrees with {shape:?} on closing",
                    variant.name
                ));
            }

            // The first variant that earns wins, so a better rate further down would never be
            // reached, and `score` could fall as distance grows.
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

    /// Number of fix ranges the search branches on.
    pub(crate) fn cardinality(&self) -> usize {
        self.shape.cardinality
    }

    /// The shape this rule scores.
    pub(crate) fn shape(&self) -> &'static dyn Shape {
        self.shape.shape
    }

    /// Score of the first variant that earns anything, else 0; in meters, unrounded.
    ///
    /// The search prunes on it, so it must be non-decreasing in `distance` and non-increasing in
    /// `gap`. The multiplier order [`Rule::check`] enforces keeps it so.
    pub(crate) fn score(&self, distance: f64, gap: f64) -> f64 {
        self.variants
            .iter()
            .map(|v| v.scored(distance, gap).score)
            .find(|&score| score > 0.0)
            .unwrap_or(0.0)
    }

    /// The full result of the winning variant, for the report.
    pub(crate) fn scored(&self, distance: f64, gap: f64) -> Scored<'_> {
        pick(&self.variants, distance, gap).scored(distance, gap)
    }
}
