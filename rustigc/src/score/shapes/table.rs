// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Every geometry the crate scores with, found by [`ShapeSpec`]. A ruleset reaches a shape only
//! through this table, so it can name only the monomorphisations listed here.

use super::balanced::BalancedCircuit;
use super::closed::ClosedCircuit;
use super::polyline::OpenPolyline;
use super::{Shape, ShapeKind, ShapeSpec};

/// One geometry: how a ruleset names it, its search cardinality, whether it closes, and its
/// instance.
#[derive(Debug)]
pub struct ShapeDef {
    pub spec: ShapeSpec,
    pub cardinality: usize,
    /// Whether the geometry has a closing leg; a variant states a closing exactly when it does.
    pub closes: bool,
    pub shape: &'static dyn Shape,
}

/// Builds [`SHAPES`] from a list of types, each entry's fields from its one type. Evaluating
/// `CARDINALITY` here fires its assert at build time.
macro_rules! shapes {
    ($( $ty:ty ),* $(,)?) => {
        /// Every geometry a ruleset may name.
        pub static SHAPES: &[ShapeDef] = &[
            $(
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

    /// Every listed geometry is found by its own spec.
    #[test]
    fn shapes_reachable_by_spec() {
        for def in SHAPES {
            let found =
                by_spec(def.spec).expect("listed shape unreachable by its own spec");
            assert_eq!(found.spec, def.spec);
        }
    }
}
