// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Search-time caches, owned by the [`Scorer`]: one set per scoring pass, shared by every rule of
//! that pass.
//!
//! Rules keep no state of their own, which is what makes the sharing possible: two rules closing the
//! same way reuse a single `ClosingCache` instead of each filling its own.

mod bbox;
mod closing;
mod furthest;

use std::cell::RefCell;

pub use bbox::BoxCache;
pub use closing::ClosingCache;
pub use furthest::{Furthest, Leg, Terminals};

use crate::utils::geometry::{BBox, SPoint, Vertices};

use super::Scorer;

#[derive(Debug)]
pub struct Caches {
    bbox: RefCell<BoxCache>,
    closing: RefCell<ClosingCache>,
    furthest: Furthest,
}

impl Caches {
    pub fn new(len: usize) -> Self {
        Caches {
            bbox: RefCell::new(BoxCache::new()),
            closing: RefCell::new(ClosingCache::new()),
            furthest: Furthest::new(len),
        }
    }

    /// `BBox` covering `range`.
    pub(super) fn bbox(&self, scorer: &Scorer, range: &[usize; 2]) -> BBox<SPoint> {
        self.bbox.borrow_mut().bbox(&scorer.track, range)
    }

    /// Closest pair of fixes between `[0, entry_stop]` and `[exit_start, last]`, as
    /// `(entry, exit, distance)`.
    pub(super) fn closing(
        &self,
        scorer: &Scorer,
        entry_stop: usize,
        exit_start: usize,
    ) -> (usize, usize, f64) {
        self.closing
            .borrow_mut()
            .closing(&scorer.track, entry_stop, exit_start)
    }

    /// Reach of `leg` from each vertex of `bbox`, the turnpoint box covering `range`.
    pub(super) fn terminals(
        &self,
        scorer: &Scorer,
        leg: Leg,
        range: &[usize; 2],
        bbox: &BBox<SPoint>,
        vertices: &Vertices<SPoint>,
    ) -> Terminals {
        self.furthest.terminals(scorer, leg, range, bbox, vertices)
    }

    /// Fix `leg` lands on when it hangs off the turnpoint at index `tp`.
    pub(super) fn end(&self, scorer: &Scorer, leg: Leg, tp: usize) -> usize {
        self.furthest.end(scorer, leg, tp)
    }
}
