// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Closed-circuit shape.

use std::iter::once;

use crate::utils::geometry::{Fcc, PointDistance};
use crate::utils::iter::pairs;

use super::circuit::{Circuit, circuit_dynamic_search};
use super::common::{candidate_to_path, transition_matrices};
use super::{Candidate, Scorer, ShapeBound, ShapePeek};

/// Closed circuit: tp1 -> ... -> tpn -> tp1, the longest the turnpoint boxes allow, with the
/// closing gap left for the rule to charge.
#[derive(Debug, Default)]
pub struct ClosedCircuit<const TP: usize> {}

impl<const TP: usize> Circuit for ClosedCircuit<TP> {
    const TURNPOINTS: usize = TP;
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

        // Start on the last fix of TP1, take the first fix of each following turnpoint, close back
        // on the origin. Ranges start in order, so clamping to the origin is the whole of what
        // keeps the loop forward; a turnpoint that cannot advance yet collapses onto it.
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
