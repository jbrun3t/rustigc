// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Closing-gap cache: the closest pair of fixes between an entry range, up to the first turnpoint,
//! and an exit range, from the last one — the closing leg of a circuit.
//!
//! Two `rstar` trees: the cache indexes answered requests by fix index, the search indexes fixes by
//! projected position.

use rstar::primitives::GeomWithData;
use rstar::{
    AABB, PointDistance as _, RStarInsertionStrategy, RTree, RTreeObject, RTreeParams,
};

use crate::utils::geometry::{BBox, Fcc, PointDistance, SPoint};
use crate::utils::projector::CheapProjection;

#[derive(Debug)]
struct ClosingCacheEntry {
    entry: [usize; 2],
    exit: [usize; 2],
    dist: f64,
}

impl RTreeObject for ClosingCacheEntry {
    type Envelope = AABB<[i32; 2]>;

    /// A rectangle in `(entry_stop, exit_start)` space, from the pair found to the bounds searched.
    /// A query inside it searches subsets of those ranges that still hold the pair, so the pair is
    /// still the closest.
    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.entry[0] as i32, self.exit[0] as i32],
            [self.entry[1] as i32, self.exit[1] as i32],
        )
    }
}

#[derive(Debug)]
pub struct ClosingCache(rstar::RTree<ClosingCacheEntry>);

impl ClosingCache {
    pub fn new() -> Self {
        ClosingCache(rstar::RTree::new())
    }

    fn get(&self, entry_stop: usize, exit_start: usize) -> Option<&ClosingCacheEntry> {
        let point = AABB::from_point([entry_stop as i32, exit_start as i32]);
        self.0.locate_in_envelope_intersecting(point).next()
    }

    fn insert(&mut self, entry: [usize; 2], exit: [usize; 2], dist: f64) {
        self.0.insert(ClosingCacheEntry { entry, exit, dist });
    }

    /// Closest pair between `[0, entry_stop]` and `[exit_start, last]`, as
    /// `(entry, exit, distance)`.
    pub fn closing(
        &mut self,
        p: &[SPoint],
        entry_stop: usize,
        exit_start: usize,
    ) -> (usize, usize, f64) {
        debug_assert!(
            entry_stop < exit_start && exit_start < p.len(),
            "bad range entry_stop={entry_stop} exit_start={exit_start} for {} points",
            p.len()
        );

        if let Some(hit) = self.get(entry_stop, exit_start) {
            return (hit.entry[0], hit.exit[1], hit.dist);
        }

        let (ei, xi, dist) = closest_closing(p, entry_stop, exit_start);
        self.insert([ei, entry_stop], [exit_start, xi], dist);
        (ei, xi, dist)
    }
}

// Search-tree parameters. `bulk_load` reads only `MAX_SIZE`, the leaf size, which sets the tree's
// depth. Building dominates querying here, so a shallow tree is faster: 24 measured faster than
// rstar's default 6 and than 48. `MIN_SIZE` and `REINSERTION_COUNT` serve `insert`, unused here;
// rstar requires `MIN_SIZE <= MAX_SIZE / 2` and `REINSERTION_COUNT < MAX_SIZE - MIN_SIZE`.
struct ClosingParams;

impl RTreeParams for ClosingParams {
    const MIN_SIZE: usize = 8;
    const MAX_SIZE: usize = 24;
    const REINSERTION_COUNT: usize = 4;
    type DefaultInsertionStrategy = RStarInsertionStrategy;
}

/// Projected fixes of the tree side, carrying their offset into that side's range.
type Tree = RTree<GeomWithData<SPoint, u32>, ClosingParams>;

/// Nearest point of `tree` closer than `radius`, as `(offset, squared distance)`, or `None` when the
/// tree holds nothing that close.
fn nearest_under(tree: &Tree, point: &SPoint, radius: f64) -> Option<(usize, f64)> {
    if radius.is_infinite() {
        // With no bound yet, a radius query would walk the whole tree.
        return tree
            .nearest_neighbor(*point)
            .map(|o| (o.data as usize, o.distance_2(point)));
    }

    // Inclusive of `radius`, which the caller then rejects for not improving on it
    tree.locate_within_distance(*point, radius)
        .map(|o| (o.data as usize, o.distance_2(point)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
}

/// Closest pair between `[0, entry_stop]` and `[exit_start, last]`: one range goes in an R-tree in
/// a local projection, the other queries it. Only the winning pair gets its FCC distance.
///
/// Each query searches within the best distance so far. That still finds the optimum: when the loop
/// reaches the optimum's own fix, its partner is within the radius unless the best already equals
/// it.
fn closest_closing(
    p: &[SPoint],
    entry_stop: usize,
    exit_start: usize,
) -> (usize, usize, f64) {
    let exit_stop = p.len() - 1;
    let entry_len = entry_stop;
    let exit_len = exit_stop - exit_start;
    let entry_range = 0..=entry_stop;
    let exit_range = exit_start..=exit_stop;

    let mut bbox: BBox<SPoint> = BBox::from_items(&p[..=entry_stop]).unwrap();
    bbox.merge(&BBox::from_items(&p[exit_start..]).unwrap());
    let projection = CheapProjection::new(&bbox.center());

    // Tree the shorter range: bulk-loading sorts, so building costs more per point than querying.
    let (iter_range, tree_range) = if entry_len > exit_len {
        (entry_range, exit_range)
    } else {
        (exit_range, entry_range)
    };

    let mut iter_best = *iter_range.start();
    let tree_start = *tree_range.start();
    let mut tree_best_offset = 0usize;
    let mut best_dist = f64::INFINITY;

    let tree: Tree = RTree::bulk_load_with_params(
        p[tree_range]
            .iter()
            .enumerate()
            .map(|(offset, fix)| {
                GeomWithData::new(projection.project(fix), offset as u32)
            })
            .collect(),
    );

    for i in iter_range {
        let point = projection.project(&p[i]);
        let Some((offset, dist)) = nearest_under(&tree, &point, best_dist) else {
            continue;
        };
        if dist < best_dist {
            iter_best = i;
            tree_best_offset = offset;
            best_dist = dist;
        }
    }

    let tree_best = tree_start + tree_best_offset;
    let dist = Fcc::distance(&p[iter_best], &p[tree_best]);

    // Either range may have been iterated: return the pair in track order.
    if iter_best <= tree_best {
        (iter_best, tree_best, dist)
    } else {
        (tree_best, iter_best, dist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points() -> Vec<SPoint> {
        vec![
            [0.0, 45.0],
            [0.01, 45.0],
            [0.02, 45.0],
            [0.05, 45.0],
            [0.021, 45.0],
            [0.1, 45.0],
        ]
    }

    #[test]
    fn closing_finds_correct() {
        let p = points();
        let mut cache = ClosingCache::new();

        let (entry, exit, dist) = cache.closing(&p, 2, 4);

        assert_eq!((entry, exit), (2, 4));
        assert_eq!(dist, Fcc::distance(&p[2], &p[4]));
    }

    #[test]
    fn closing_cache_hit_repeat() {
        let p = points();
        let mut cache = ClosingCache::new();

        let first = cache.closing(&p, 2, 4);
        let second = cache.closing(&p, 2, 4);

        assert_eq!(first, second);
        assert_eq!(cache.0.size(), 1);
    }

    // Closest fix (index 2) is short of entry_stop (4), leaving slack on the entry axis: a narrower
    // entry_stop still lands inside the rectangle the first search stored.
    fn subset_points() -> Vec<SPoint> {
        vec![
            [0.0, 45.0],
            [0.01, 45.0],
            [0.02, 45.0],
            [-0.5, 45.0],
            [-0.6, 45.0],
            [0.05, 45.0],
            [0.021, 45.0],
            [0.5, 45.0],
        ]
    }

    #[test]
    fn closing_cache_hit_narrow() {
        let p = subset_points();
        let mut cache = ClosingCache::new();

        let first = cache.closing(&p, 4, 6);
        let second = cache.closing(&p, 3, 6);

        assert_eq!(first, (2, 6, Fcc::distance(&p[2], &p[6])));
        assert_eq!(second, first);
        assert_eq!(cache.0.size(), 1);
    }

    // Two candidate pairs on each side, so shifting which fixes are in range changes the winner.
    fn restart_points() -> Vec<SPoint> {
        vec![
            [0.0, 45.0],
            [1.0, 45.0],
            [0.06, 45.0],
            [0.001, 45.0],
            [1.01, 45.0],
            [2.0, 45.0],
        ]
    }

    #[test]
    fn closing_overlap_different_cache_miss() {
        let p = restart_points();
        let mut cache = ClosingCache::new();

        // entry_stop matches the cached rectangle exactly, but exit_start (3) falls short of the
        // cached exit bound (4): not a subset, so this must miss and search again.
        let first = cache.closing(&p, 1, 4);
        let second = cache.closing(&p, 1, 3);

        assert_eq!(first, (1, 4, Fcc::distance(&p[1], &p[4])));
        assert_eq!(second, (0, 3, Fcc::distance(&p[0], &p[3])));
        assert_ne!(first, second);
        assert_eq!(cache.0.size(), 2);
    }

    #[test]
    fn closing_overlap_cache_hit_extend() {
        let p = restart_points();
        let mut cache = ClosingCache::new();

        let narrow = cache.closing(&p, 1, 4);
        // entry_stop (2) exceeds the cached entry bound (1): outside the rectangle, so this must
        // search the wider range rather than reuse the narrower cached result.
        let wider = cache.closing(&p, 2, 4);

        assert_eq!(narrow, (1, 4, Fcc::distance(&p[1], &p[4])));
        assert_eq!(wider, (1, 4, Fcc::distance(&p[1], &p[4])));
        assert_eq!(cache.0.size(), 2);
    }

    // debug_assert! is compiled out in release, and here both indices stay in-bounds even reversed,
    // so nothing panics at all — no message to catch. Debug-only check.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "bad range")]
    fn closing_range_reverse() {
        let p = restart_points();
        let mut cache = ClosingCache::new();

        cache.closing(&p, 4, 1);
    }

    // Unlike the reverse case above, an out-of-bound index still panics in release (from the
    // downstream unwrap) — just not with this message.
    #[test]
    #[cfg_attr(debug_assertions, should_panic(expected = "bad range"))]
    #[cfg_attr(not(debug_assertions), should_panic)]
    fn closing_range_out_of_bound() {
        let p = restart_points();
        let mut cache = ClosingCache::new();

        cache.closing(&p, 0, p.len());
    }
}
