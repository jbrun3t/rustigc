// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Closed-circuit shape.

use std::iter::once;

use crate::utils::geometry::{Fcc, PointDistance};
use crate::utils::iter::pairs;

use super::circuit::{Circuit, circuit_dynamic_search};
use super::common::{candidate_to_path, transition_matrices};
use super::{Candidate, Scorer, ShapeBound, ShapeKind, ShapePeek, ShapeSpec};

/// A loop through `TP` turnpoints, tp1 -> ... -> tpn -> tp1. The gap between the flight's entry and
/// exit is left for the rule to charge.
#[derive(Debug)]
pub struct ClosedCircuit<const TP: usize> {}

impl<const TP: usize> Circuit for ClosedCircuit<TP> {
    const TURNPOINTS: usize = TP;
}

impl<const TP: usize> ShapeKind for ClosedCircuit<TP> {
    const CARDINALITY: usize = <Self as Circuit>::CARDINALITY;
    const CLOSES: bool = true;
    const SPEC: ShapeSpec = ShapeSpec::ClosedCircuit { turnpoints: TP };
    const NEW: Self = Self {};
}

impl<const TP: usize> ShapeBound for ClosedCircuit<TP> {
    fn bound(&self, scorer: &Scorer, c: &Candidate) -> f64 {
        let path = candidate_to_path(scorer, c);
        let transitions = transition_matrices(pairs(&path, true), true);

        circuit_dynamic_search(transitions.as_slice(), &f64::max)
    }
}

impl<const TP: usize> ShapePeek for ClosedCircuit<TP> {
    fn peek(&self, scorer: &Scorer, c: &Candidate) -> f64 {
        if c.first_last_overlap() {
            return 0.;
        }

        let track = &scorer.track;

        // From the last fix of TP1 through the first fix of each following range, back to TP1.
        // Ranges start in order, so clamping to the origin keeps the loop forward; a turnpoint
        // that cannot advance yet collapses onto it.
        let origin = c.ranges[0][1];
        let fixes = c.ranges[1..].iter().map(|r| r[0].max(origin));

        let mut from = origin;
        let mut total = 0.;
        for to in fixes.chain(once(origin)) {
            total += Fcc::distance(&track[from], &track[to]);
            from = to;
        }

        total
    }
}
