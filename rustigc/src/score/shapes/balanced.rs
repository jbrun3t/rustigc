// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Balanced closed-circuit shape.

use std::iter::once;

use crate::utils::geometry::{Fcc, PointDistance};
use crate::utils::iter::pairs;

use super::circuit::{Circuit, circuit_dynamic_search};
use super::common::{candidate_to_path, transition_matrices};
use super::{Candidate, Scorer, ShapeBound, ShapeKind, ShapePeek, ShapeSpec};

/// A closed circuit whose shortest leg is at least `MINSIDE` per mille of the total: 280 is the FAI
/// triangle's 28 %.
#[derive(Debug)]
pub struct BalancedCircuit<const TP: usize, const MINSIDE: u32> {}

impl<const TP: usize, const MINSIDE: u32> BalancedCircuit<TP, MINSIDE> {
    fn ratio(&self) -> f64 {
        MINSIDE as f64 / 1000.0
    }
}

impl<const TP: usize, const MINSIDE: u32> ShapeKind for BalancedCircuit<TP, MINSIDE> {
    const CARDINALITY: usize = <Self as Circuit>::CARDINALITY;
    const CLOSES: bool = true;
    const SPEC: ShapeSpec = ShapeSpec::BalancedCircuit {
        turnpoints: TP,
        min_side: MINSIDE,
    };
    const NEW: Self = Self {};
}

impl<const TP: usize, const MINSIDE: u32> Circuit for BalancedCircuit<TP, MINSIDE> {
    const TURNPOINTS: usize = TP;

    fn valid(&self, legs: &[f64]) -> bool {
        let total: f64 = legs.iter().sum();
        legs.iter().all(|leg| *leg >= self.ratio() * total)
    }
}

impl<const TP: usize, const MINSIDE: u32> ShapeBound for BalancedCircuit<TP, MINSIDE> {
    fn bound(&self, scorer: &Scorer, c: &Candidate) -> f64 {
        let path = candidate_to_path(scorer, c);
        let transitions = transition_matrices(pairs(&path, true), true);

        // shortest leg >= ratio * total, so total <= shortest / ratio. Each leg is at most the
        // longest transition between its two boxes, so the smallest of those caps the shortest.
        let short_max = transitions
            .iter()
            .map(|tr| tr.rows().flatten().fold(f64::NAN, |r, v| r.max(*v)))
            .fold(f64::NAN, |r, v| r.min(v));
        let shape_min = circuit_dynamic_search(transitions.as_slice(), &f64::min);

        // Nothing in these boxes is short enough to balance. The ratio must not reject what the
        // geodesic would accept: `peek`'s test reversed, the shortest leg stretched and the total
        // shrunk. The cap stays on that stretched scale, as on the leaves it bounds.
        let balanced_max = scorer.stretch(short_max) / self.ratio();
        if balanced_max < scorer.shrink(shape_min) {
            return 0f64;
        }

        let shape_max = circuit_dynamic_search(transitions.as_slice(), &f64::max);
        debug_assert!(shape_min <= shape_max);
        balanced_max.min(shape_max)
    }
}

impl<const TP: usize, const MINSIDE: u32> ShapePeek for BalancedCircuit<TP, MINSIDE> {
    fn peek(&self, scorer: &Scorer, c: &Candidate) -> f64 {
        if c.first_last_overlap() {
            return 0.;
        }

        let track = &scorer.track;
        let origin = c.ranges[0][1];
        let fixes = c.ranges[1..].iter().map(|r| r[0].max(origin));

        let mut from = origin;
        let mut total = 0.;
        let mut short = f64::INFINITY;
        for to in fixes.chain(once(origin)) {
            let leg = Fcc::distance(&track[from], &track[to]);
            total += leg;
            short = short.min(leg);
            from = to;
        }

        // The ratio must still hold on the geodesic: the total stretched, the shortest leg shrunk.
        if scorer.stretch(total) * self.ratio() > scorer.shrink(short) {
            return 0.;
        }

        total
    }
}
