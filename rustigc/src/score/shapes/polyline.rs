// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Open-distance shape.

use smallvec::{SmallVec, smallvec};

use crate::score::cache::Leg;
use crate::utils::geometry::{BBox, Fcc, Geodesic, PointDistance, SPoint};
use crate::utils::iter::pairs;

use super::common::{dynamic_search, transition_matrices};
use super::{
    Candidate, Path, Scorer, ShapeBound, ShapeCommon, ShapeKind, ShapePeek, Task,
};

/// Open free-distance polyline of `POINTS` points, entry and exit included
#[derive(Debug, Default)]
pub struct OpenPolyline<const POINTS: usize> {}

impl<const POINTS: usize> OpenPolyline<POINTS> {
    /// Whether the ends are looked up as furthest points rather than searched as two more ranges.
    const TERMINALS: bool = POINTS > 3;
}

impl<const POINTS: usize> ShapeKind for OpenPolyline<POINTS> {
    // Two points is the shortest thing with a length. Above the threshold two of them are spent
    // on the terminal legs instead of being searched.
    const CARDINALITY: usize = {
        assert!(POINTS >= 2);
        if Self::TERMINALS { POINTS - 2 } else { POINTS }
    };
}

impl<const POINTS: usize> ShapeBound for OpenPolyline<POINTS> {
    fn bound(&self, scorer: &Scorer, c: &Candidate) -> f64 {
        let boxes: SmallVec<[BBox<SPoint>; 4]> =
            c.ranges.iter().map(|r| scorer.bbox(r)).collect();
        let path: Path = boxes.iter().map(|b| b.vertices()).collect();
        let last = c.ranges.len() - 1;

        let (entry, exit) = if Self::TERMINALS {
            (
                scorer.terminals(Leg::Entry, &c.ranges[0], &boxes[0], &path[0]),
                scorer.terminals(Leg::Exit, &c.ranges[last], &boxes[last], &path[last]),
            )
        } else {
            (
                smallvec![0.; path[0].len()],
                smallvec![0.; path[last].len()],
            )
        };

        let transitions = transition_matrices(pairs(&path, false), false);
        dynamic_search(&entry, &exit, &transitions, &f64::max)
    }
}

impl<const POINTS: usize> ShapePeek for OpenPolyline<POINTS> {
    fn peek(&self, scorer: &Scorer, c: &Candidate) -> f64 {
        let track = &scorer.track;
        let (entry, exit) = self.endpoints(scorer, c);

        let (total, last) = c.ranges.iter().fold((0., entry), |(total, prev), r| {
            (total + Fcc::distance(&track[prev], &track[r[0]]), r[0])
        });

        total + Fcc::distance(&track[last], &track[exit])
    }
}

impl<const POINTS: usize> ShapeCommon for OpenPolyline<POINTS> {
    fn gap(&self, _scorer: &Scorer, _c: &Candidate) -> f64 {
        0.
    }

    fn endpoints(&self, scorer: &Scorer, c: &Candidate) -> (usize, usize) {
        let last = c.ranges.len() - 1;

        if !Self::TERMINALS {
            return (c.ranges[0][0], c.ranges[last][0]);
        }

        // NOTE: this is only valid as long as peek() pick these points
        (
            scorer.end(Leg::Entry, c.ranges[0][0]),
            scorer.end(Leg::Exit, c.ranges[last][0]),
        )
    }

    fn report(&self, scorer: &Scorer, c: &Candidate) -> Option<Task> {
        let track = &scorer.track;
        let mut turnpoints = c.positions();
        let (entry, exit) = self.endpoints(scorer, c);

        // The ends are reported as such, not as turnpoints. The fold still walks the whole chain:
        // it starts at `entry` and closes on `exit`, which are the two being dropped here.
        if !Self::TERMINALS {
            turnpoints.remove(0);
            turnpoints.pop();
        }

        let (total, last) = turnpoints.iter().fold((0., entry), |(total, prev), &i| {
            (total + Geodesic::distance(&track[prev], &track[i]), i)
        });

        // Nothing but length to constrain, so the re-measure can never reject.
        Some(Task {
            distance: total + Geodesic::distance(&track[last], &track[exit]),
            gap: 0.0,
            entry,
            turnpoints,
            exit,
            circuit: false,
        })
    }
}
