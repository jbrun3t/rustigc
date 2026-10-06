// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! What every circuit shares: the closing search, the circuit DP and the report.

use crate::utils::geometry::{Geodesic, PointDistance};
use crate::utils::iter::pairs;

use super::{Candidate, Scorer, ShapeCommon, Task, TransitionMatrix};

use super::common::dynamic_search;

use std::fmt::Debug;

/// Best circuit through one vertex of each turnpoint box, `pick` choosing longest or shortest.
pub fn circuit_dynamic_search<F>(transitions: &[TransitionMatrix], pick: &F) -> f64
where
    F: Fn(f64, f64) -> f64,
{
    // The closing leg depends on where the circuit started, so this runs one DP per vertex of
    // TP1: the legs TP(1 -> 2) and TP(n -> 1) are that run's entry and exit costs.
    match transitions {
        [entry, tr @ .., exit] => {
            debug_assert_eq!(entry.rows().len(), exit.rows().len());
            entry
                .rows()
                .zip(exit.rows())
                .map(|(e, x)| dynamic_search(e, x, tr, pick))
                .fold(f64::NAN, pick)
        }
        _ => {
            // `CARDINALITY` asserts at least 2 turnpoints, which wrap to two legs.
            unreachable!("a circuit needs at least 2 turnpoints");
        }
    }
}

/// A loop through its turnpoints, closed back to the first.
pub trait Circuit {
    /// Turnpoints the loop is defined on.
    const TURNPOINTS: usize;

    /// [`ShapeKind::CARDINALITY`](super::ShapeKind::CARDINALITY) for a circuit, asserting at least
    /// 2 turnpoints. Every circuit shape forwards it.
    const CARDINALITY: usize = {
        assert!(Self::TURNPOINTS >= 2);
        Self::TURNPOINTS
    };

    /// Whether the final geodesic legs still satisfy the circuit's own constraints.
    fn admissible(&self, _legs: &[f64]) -> bool {
        true
    }

    /// Closing pair `(entry, exit, gap)`: the closest fixes up to the end of the first range and
    /// from the start of the last. `(0, last, 0)` while those ranges overlap.
    fn closing(&self, scorer: &Scorer, c: &Candidate) -> (usize, usize, f64) {
        if c.first_last_overlap() {
            (0, scorer.track.len() - 1, 0.0)
        } else {
            // The entry and exit may still fall inside the first and last turnpoint ranges.
            scorer.closing(c.ranges[0][1], c.ranges[c.ranges.len() - 1][0])
        }
    }
}

impl<T: Circuit + Debug> ShapeCommon for T {
    fn gap(&self, scorer: &Scorer, c: &Candidate) -> f64 {
        let (_, _, gap) = self.closing(scorer, c);
        gap
    }

    fn endpoints(&self, scorer: &Scorer, c: &Candidate) -> (usize, usize) {
        if c.first_last_overlap() {
            (0, scorer.track.len() - 1)
        } else {
            let (entry, exit, _) = self.closing(scorer, c);
            (entry, exit)
        }
    }

    fn report(&self, scorer: &Scorer, c: &Candidate) -> Option<Task> {
        let track = &scorer.track;
        let turnpoints = c.positions();

        let legs: Vec<f64> = pairs(&turnpoints, true)
            .map(|(a, b)| Geodesic::distance(&track[*a], &track[*b]))
            .collect();

        if !self.admissible(&legs) {
            return None;
        }

        let (entry, exit) = self.endpoints(scorer, c);

        Some(Task {
            distance: legs.iter().sum(),
            gap: Geodesic::distance(&track[entry], &track[exit]),
            entry,
            turnpoints,
            exit,
            circuit: true,
        })
    }
}
