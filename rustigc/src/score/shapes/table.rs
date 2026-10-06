// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! The geometries a rule may name, by [`ShapeSpec`].
//!
//! Every monomorphisation the crate scores with is listed here and nowhere else, and a rule reaches
//! one only through this table. The set is therefore fixed at compile time however the rules are
//! configured: nothing can ask for a `ClosedCircuit<9>` that was never instantiated.

use super::balanced::BalancedCircuit;
use super::closed::ClosedCircuit;
use super::polyline::OpenPolyline;
use super::{Shape, ShapeKind, ShapeSpec};

/// One geometry, as a rule names it.
///
/// `shape` borrows the geometry's own singleton, so naming a shape costs no allocation.
#[derive(Debug)]
pub struct ShapeDef {
    pub spec: ShapeSpec,
    pub cardinality: usize,
    /// Whether there is a closing leg for a rule to charge. A rule pricing a gap over a geometry
    /// that has none would charge nothing, whatever its closing said.
    pub closes: bool,
    pub shape: &'static dyn Shape,
}

/// Builds [`SHAPES`] from a list of types.
///
/// Every field of an entry comes from the one type, so they cannot drift apart, and evaluating
/// `CARDINALITY` here is what fires its post-monomorphisation assert at build time.
macro_rules! shapes {
    ($( $(#[$attr:meta])* $ty:ty ),* $(,)?) => {
        /// Every geometry a rule may name.
        pub static SHAPES: &[ShapeDef] = &[
            $(
                $(#[$attr])*
                ShapeDef {
                    spec: <$ty as ShapeKind>::SPEC,
                    cardinality: <$ty as ShapeKind>::CARDINALITY,
                    closes: <$ty as ShapeKind>::CLOSES,
                    shape: &<$ty as ShapeKind>::NEW,
                },
            )*
        ];
    };
}

shapes! {
    OpenPolyline<2>,
    OpenPolyline<3>,
    OpenPolyline<4>,
    OpenPolyline<5>,
    ClosedCircuit<2>,
    ClosedCircuit<3>,
    BalancedCircuit<3, 280>,
}

/// The geometry `spec` names, `None` for one this build does not carry.
pub fn by_spec(spec: ShapeSpec) -> Option<&'static ShapeDef> {
    SHAPES.iter().find(|def| def.spec == spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two entries answering to one spec would make `by_spec` pick by list order.
    #[test]
    fn shapes_specs_unique() {
        for (i, def) in SHAPES.iter().enumerate() {
            assert!(
                !SHAPES[..i].iter().any(|other| other.spec == def.spec),
                "{:?} is listed twice",
                def.spec
            );
        }
    }

    /// Every listed geometry answers to its own spec, which is what a ruleset file names it by.
    #[test]
    fn shapes_reachable_by_spec() {
        for def in SHAPES {
            let found =
                by_spec(def.spec).expect("listed shape unreachable by its own spec");
            assert_eq!(found.spec, def.spec);
        }
    }
}
