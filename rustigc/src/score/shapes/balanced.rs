// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Balanced closed-circuit shape.

use std::iter::once;

use crate::utils::geometry::{Fcc, PointDistance};
use crate::utils::iter::pairs;

use super::circuit::{circuit_dynamic_search, Circuit};
use super::common::{candidate_to_path, transition_matrices};
use super::{Candidate, Scorer, ShapeBound, ShapePeek};

/// Balanced circuit: a closed circuit whose shortest leg must reach a given share of the total,
/// the FAI triangle's 28 % rule.
///
/// `MINSIDE` is that share in per mille, so 280 is the 28 % rule. Carrying it on the type rather
/// than in a field keeps the struct empty, and a boxed ZST costs no allocation.
#[derive(Debug, Default)]
pub struct BalancedCircuit<const TP: usize, const MINSIDE: u32> {}

impl<const TP: usize, const MINSIDE: u32> BalancedCircuit<TP, MINSIDE> {
    fn ratio(&self) -> f64 {
        MINSIDE as f64 / 1000.0
    }
}

impl<const TP: usize, const MINSIDE: u32> Circuit for BalancedCircuit<TP, MINSIDE> {
    const TURNPOINTS: usize = TP;

    fn admissible(&self, legs: &[f64]) -> bool {
        let total: f64 = legs.iter().sum();
        legs.iter().all(|leg| *leg >= self.ratio() * total)
    }
}

impl<const TP: usize, const MINSIDE: u32> ShapeBound for BalancedCircuit<TP, MINSIDE> {
    fn bound(&self, scorer: &Scorer, c: &Candidate) -> f64 {
        let path = candidate_to_path(scorer, c);
        let transitions = transition_matrices(pairs(&path, true), true);

        // shortest leg >= ratio * total, so total <= shortest / ratio. No leg reaches beyond the
        // longest transition its own two boxes allow, so the smallest of those caps the shortest.
        let balanced_max = transitions
            .iter()
            .map(|tr| tr.rows().flatten().fold(f64::NAN, |r, v| r.max(*v)))
            .fold(f64::NAN, |r, v| r.min(v))
            / self.ratio();

        // Nothing in these boxes is short enough to balance.
        let shape_min = circuit_dynamic_search(transitions.as_slice(), &f64::min);
        if balanced_max < shape_min {
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

        // The ratio has to hold once `report` re-measures geodesically, and the two metrics may
        // disagree either way: test it with the total long and the shortest leg short.
        if total * (1.0 + scorer.margin(total)) * self.ratio()
            > short * (1.0 - scorer.margin(short))
        {
            return 0.;
        }

        total
    }
}
