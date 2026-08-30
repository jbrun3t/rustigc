// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Furthest-point caches: how far the entry, or exit, leg of an open distance reaches, and where it
//! lands.
//!
//! Three levels, all keyed on the question their caller asks rather than on the one they ask below:
//! [`Furthest::terminals`] on the turnpoint range the bound is working on, [`Furthest::end`] on the
//! turnpoint a leg hangs off, and [`FurthestCache`] underneath on the anchor and the bound scanned.
//! The upper two collapse many distinct scans into one lookup; keying them on the scan parameters
//! instead reads as a healthy cache while doing hundreds of times the work.

use std::cell::RefCell;

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::score::Scorer;
use crate::utils::geometry::{
    BBox, Fcc, Flat, PointCoords, PointDistance, SPoint, Vertices,
};
use crate::utils::projector::CheapProjection;

/// Reach of an entry or exit leg, one value per vertex of the adjacent turnpoint box.
pub type Terminals = SmallVec<[f64; 4]>;

/// Which end of the turnpoint chain a leg hangs off, and so which way its scan runs. The
/// discriminant indexes the per-direction caches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leg {
    Entry = 0,
    Exit = 1,
}

impl Leg {
    fn index(self) -> usize {
        self as usize
    }

    /// Fix a scan for this leg stops at, out of the turnpoint range the leg hangs off: the edge
    /// facing it. Both legs therefore scan away from the range and skip its interior, which
    /// [`Furthest::terminals`] covers with the box diagonal instead.
    fn tp(self, range: &[usize; 2]) -> usize {
        range[self.index()]
    }
}

/// A search, in cache coordinates
#[derive(Debug)]
struct FurthestCacheEntry {
    /// Furthest fix it found
    start: usize,
    /// Bound covered
    stop: usize,
    /// Cached distance
    distance: f64,
}

impl FurthestCacheEntry {
    fn contains(&self, index: usize) -> bool {
        (self.start <= index) && (index <= self.stop)
    }
}

#[derive(Debug)]
struct FurthestCache {
    cache: FxHashMap<u128, Vec<FurthestCacheEntry>>,
    leg: Leg,
    len: usize,
}

impl FurthestCache {
    fn key(point: &SPoint) -> u128 {
        point.x().to_bits() as u128 | (point.y().to_bits() as u128) << 64
    }

    fn new(len: usize, leg: Leg) -> Self {
        FurthestCache {
            cache: FxHashMap::default(),
            leg,
            len,
        }
    }

    /// Mirror exit-leg coordinates so both directions grow from 0
    fn translate(&self, value: usize) -> usize {
        match self.leg {
            Leg::Entry => value,
            Leg::Exit => self.len - 1 - value,
        }
    }

    /// Returns full or partial match on the search range, if any
    fn get(&self, point: &SPoint, tp: usize) -> Option<(usize, f64, usize, bool)> {
        let key = Self::key(point);
        let stop = self.translate(tp);
        let list = self.cache.get(&key)?;

        // Entry whose fix sits closest to `tp`: the likeliest to cover it, and otherwise the one
        // leaving the least to rescan
        let upper = list.partition_point(|e| e.start <= stop);
        list[..upper].last().map(|e| {
            (
                self.translate(e.start),
                e.distance,
                self.translate(e.stop),
                e.contains(stop),
            )
        })
    }

    /// Records the result of a scan, possibly extending an existing record.
    /// Returns the furthest fix over the whole range.
    fn insert(&mut self, point: &SPoint, tp: usize, r: &(usize, f64)) -> (usize, f64) {
        let key = Self::key(point);
        let (mut start, mut distance) = (self.translate(r.0), r.1);
        let stop = self.translate(tp);

        let list = self.cache.entry(key).or_default();

        // Adopt the entry the scan resumed from when it still holds the furthest fix, leaving the
        // extension below to stretch it over the range just scanned
        let upper = list.partition_point(|e| e.start <= stop);
        if let Some(prefix) = list[..upper].last()
            && prefix.distance > distance
        {
            start = prefix.start;
            distance = prefix.distance;
        }

        // Same fix found means the same distance, so only the bound reached can have grown
        match list.iter_mut().find(|e| e.start == start) {
            Some(entry) => {
                debug_assert_eq!(
                    entry.distance, distance,
                    "Distance conflict for {} to {:?}: {} != {}",
                    start, point, entry.distance, distance
                );
                entry.stop = stop.max(entry.stop);
            }
            None => {
                // Kept sorted by start, `get` binary searches it
                let pos = list.partition_point(|e| e.start < start);
                list.insert(
                    pos,
                    FurthestCacheEntry {
                        start,
                        stop,
                        distance,
                    },
                );
            }
        }

        (self.translate(start), distance)
    }

    #[allow(unused)]
    fn len(&self) -> usize {
        self.cache.len()
    }

    /// Get-or-compute the furthest fix from `anchor` between `tp` and the window end this cache
    /// searches from, as `(index, distance)`.
    fn furthest(&mut self, p: &[SPoint], anchor: &SPoint, tp: usize) -> (usize, f64) {
        debug_assert!(tp < p.len(), "bad range tp={tp} for {} points", p.len());

        let mut start = self.translate(0);

        if let Some((fix, distance, bound, covered)) = self.get(anchor, tp) {
            if covered {
                return (fix, distance);
            }

            // Resume from the bound reached, `insert` puts back what this scan leaves out
            start = bound;
        }

        let r = furthest_flat(p, anchor, start, tp);
        self.insert(anchor, tp, &r)
    }
}

/// Linear scan, ranking the fixes on a plane projected around `anchor` and computing the real
/// distance for the winner only.
fn furthest_flat(
    p: &[SPoint],
    anchor: &SPoint,
    ground: usize,
    tp: usize,
) -> (usize, f64) {
    let (left, right) = if ground < tp {
        (ground, tp)
    } else {
        (tp, ground)
    };

    let mut max_dist = 0.;
    let mut max_idx = left;

    let projection = CheapProjection::new(anchor);

    // Ranked on the square: same winner, without a `sqrt` per fix
    for (offset, fix) in p[left..=right].iter().enumerate() {
        let point = projection.project(fix);
        let dist = Flat::distance_squared(&[0., 0.], &point);
        if max_dist < dist {
            max_idx = left + offset;
            max_dist = dist;
        }
    }

    max_dist = Fcc::distance(anchor, &p[max_idx]);

    (max_idx, max_dist)
}

/// The whole furthest stack, one set of caches per leg.
#[derive(Debug)]
pub struct Furthest {
    scan: [RefCell<FurthestCache>; 2],
    terminals: [RefCell<FxHashMap<[usize; 2], Terminals>>; 2],
    ends: [RefCell<Vec<Option<usize>>>; 2],
}

impl Furthest {
    pub fn new(len: usize) -> Self {
        Furthest {
            scan: [
                RefCell::new(FurthestCache::new(len, Leg::Entry)),
                RefCell::new(FurthestCache::new(len, Leg::Exit)),
            ],
            terminals: Default::default(),
            ends: [RefCell::new(vec![None; len]), RefCell::new(vec![None; len])],
        }
    }

    /// Furthest fix from `anchor` between `tp` and the end of the window `leg` reaches for, as
    /// `(index, distance)`.
    fn furthest(
        &self,
        scorer: &Scorer,
        leg: Leg,
        anchor: &SPoint,
        tp: usize,
    ) -> (usize, f64) {
        self.scan[leg.index()]
            .borrow_mut()
            .furthest(&scorer.track, anchor, tp)
    }

    /// Reach of `leg` from each vertex of the turnpoint box covering `range`, floored at the box
    /// diagonal: the scan stops at the edge of the range, so the floor is what covers a furthest
    /// point sitting inside the box.
    pub(super) fn terminals(
        &self,
        scorer: &Scorer,
        leg: Leg,
        range: &[usize; 2],
        bbox: &BBox<SPoint>,
        vertices: &Vertices<SPoint>,
    ) -> Terminals {
        debug_assert_eq!(*bbox, scorer.bbox(range), "box does not cover {range:?}");

        let cache = &self.terminals[leg.index()];

        // Not held across the scans below, which reach back into the furthest caches
        let hit = cache.borrow().get(range).cloned();
        if let Some(hit) = hit {
            return hit;
        }

        let diagonal = bbox.diagonal(Fcc);
        let tp = leg.tp(range);

        let value: Terminals = vertices
            .iter()
            .map(|v| self.furthest(scorer, leg, v, tp).1.max(diagonal))
            .collect();

        cache.borrow_mut().insert(*range, value.clone());
        value
    }

    /// Fix `leg` lands on when it hangs off the turnpoint at index `tp`.
    pub(super) fn end(&self, scorer: &Scorer, leg: Leg, tp: usize) -> usize {
        let slots = &self.ends[leg.index()];

        // Not held across the search below, which reaches back into the furthest caches
        let hit = slots.borrow()[tp];
        if let Some(hit) = hit {
            return hit;
        }

        let anchor = scorer.track[tp];
        let fix = self.furthest(scorer, leg, &anchor, tp).0;

        slots.borrow_mut()[tp] = Some(fix);
        fix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Strictly increasing distance from anchor [0, 45]: the furthest fix is always the last one in
    // range.
    fn points() -> Vec<SPoint> {
        vec![
            [0.001, 45.0],
            [0.01, 45.0],
            [0.02, 45.0],
            [0.03, 45.0],
            [0.05, 45.0],
            [0.2, 45.0],
        ]
    }

    #[test]
    fn furthest_correct() {
        let anchor = [0.0, 45.0];
        let p = points();
        let mut cache = FurthestCache::new(p.len(), Leg::Entry);

        let (idx, dist) = cache.furthest(&p, &anchor, 4);

        assert_eq!(idx, 4);
        assert_eq!(dist, Fcc::distance(&anchor, &p[4]));
    }

    #[test]
    fn furthest_reverse_correct() {
        let anchor = [0.0, 45.0];
        let p: Vec<SPoint> = points().into_iter().rev().collect();
        let mut cache = FurthestCache::new(p.len(), Leg::Exit);

        // Exit-leg search from the last fix down to tp=1: the furthest fix in [1, 5] is index 1
        // (0.05), the reversed fixture's largest value in range.
        let (idx, dist) = cache.furthest(&p, &anchor, 1);

        assert_eq!(idx, 1);
        assert_eq!(dist, Fcc::distance(&anchor, &p[1]));
    }

    #[test]
    fn furthest_cache_hit() {
        let anchor = [0.0, 45.0];
        let p = points();
        let mut cache = FurthestCache::new(p.len(), Leg::Entry);

        let first = cache.furthest(&p, &anchor, 4);
        let second = cache.furthest(&p, &anchor, 4);

        assert_eq!(first, second);
        assert_eq!(cache.len(), 1);
        assert_eq!(
            cache.cache.get(&FurthestCache::key(&anchor)).unwrap().len(),
            1
        );
    }

    #[test]
    fn furthest_cache_extend() {
        let anchor = [0.0, 45.0];
        // Winner (index 1) sits short of the first query's stop (index 3), so the second query's
        // resumed segment [3, 4] doesn't include it: the old winner must be adopted across entries,
        // not just re-found locally.
        let p = vec![
            [0.001, 45.0],
            [0.05, 45.0],
            [0.01, 45.0],
            [0.02, 45.0],
            [0.03, 45.0],
        ];
        let mut cache = FurthestCache::new(p.len(), Leg::Entry);

        let first = cache.furthest(&p, &anchor, 3);
        let second = cache.furthest(&p, &anchor, 4);

        assert_eq!(first, (1, Fcc::distance(&anchor, &p[1])));
        assert_eq!(second, first);
        assert_eq!(
            cache.cache.get(&FurthestCache::key(&anchor)).unwrap().len(),
            1
        );
    }

    #[test]
    fn furthest_extend_new_max() {
        let anchor = [0.0, 45.0];
        let p = points();
        let mut cache = FurthestCache::new(p.len(), Leg::Entry);

        let first = cache.furthest(&p, &anchor, 4);
        let second = cache.furthest(&p, &anchor, 5);

        assert_eq!(first.0, 4);
        assert_eq!(second.0, 5);
        assert_ne!(first, second);

        // Both fixes are kept: the old entry still answers any tp within its own range, the new one
        // takes over only once tp reaches index 5.
        assert_eq!(
            cache.cache.get(&FurthestCache::key(&anchor)).unwrap().len(),
            2
        );
    }

    #[test]
    fn furthest_cache_miss_key() {
        let p = points();
        let mut cache = FurthestCache::new(p.len(), Leg::Entry);

        cache.furthest(&p, &[0.0, 45.0], 4);
        cache.furthest(&p, &[1.0, 45.0], 4);

        assert_eq!(cache.len(), 2);
    }

    // debug_assert! is compiled out in release, so a bad range panics later from the out-of-bounds
    // slice instead — still a panic, just not this message.
    #[test]
    #[cfg_attr(debug_assertions, should_panic(expected = "bad range"))]
    #[cfg_attr(not(debug_assertions), should_panic)]
    fn furthest_range_out_of_bound() {
        let p = points();
        let mut cache = FurthestCache::new(p.len(), Leg::Entry);

        cache.furthest(&p, &[0.0, 45.0], p.len());
    }
}
