// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Bounding-box cache, keyed on the inclusive `(start, end)` fix-index range.
//!
//! Building a box is linear in the range length and the search keeps revisiting the same ranges as
//! it splits neighbouring candidates.

use rustc_hash::FxHashMap;

use crate::utils::geometry::{BBox, SPoint};

#[derive(Debug)]
pub struct BoxCache(FxHashMap<(usize, usize), BBox<SPoint>>);

impl BoxCache {
    pub fn new() -> Self {
        BoxCache(FxHashMap::default())
    }

    /// Get-or-compute the bounding box covering the range.
    pub fn bbox(&mut self, p: &[SPoint], range: &[usize; 2]) -> BBox<SPoint> {
        debug_assert!(
            range[0] <= range[1] && range[1] < p.len(),
            "bad range {range:?} for {} points",
            p.len()
        );

        if let Some(&b) = self.0.get(&(range[0], range[1])) {
            return b;
        }

        let b = BBox::from_items(&p[range[0]..=range[1]]).unwrap();
        self.0.insert((range[0], range[1]), b);
        b
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points() -> Vec<SPoint> {
        vec![[0.0, 0.0], [1.0, 3.0], [4.0, 1.0], [2.0, 2.0], [5.0, 5.0]]
    }

    #[test]
    fn bbox_correct_coverage() {
        let p = points();
        let mut cache = BoxCache::new();

        let b = cache.bbox(&p, &[1, 3]);

        assert_eq!(b.min, [1.0, 1.0]);
        assert_eq!(b.max, [4.0, 3.0]);
    }

    #[test]
    fn bbox_cache_hit() {
        let p = points();
        let mut cache = BoxCache::new();

        let first = cache.bbox(&p, &[0, 2]);
        let second = cache.bbox(&p, &[0, 2]);

        assert_eq!(first, second);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn bbox_cache_miss() {
        let p = points();
        let mut cache = BoxCache::new();

        cache.bbox(&p, &[0, 2]);
        cache.bbox(&p, &[2, 4]);

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn bbox_get_point() {
        let p = points();
        let mut cache = BoxCache::new();

        let b = cache.bbox(&p, &[2, 2]);

        assert_eq!(b.min, p[2]);
        assert_eq!(b.max, p[2]);
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic(expected = "bad range"))]
    #[cfg_attr(not(debug_assertions), should_panic)]
    fn bbox_cache_reverse_range() {
        let p = points();
        let mut cache = BoxCache::new();

        cache.bbox(&p, &[3, 1]);
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic(expected = "bad range"))]
    #[cfg_attr(not(debug_assertions), should_panic)]
    fn bbox_cache_out_of_bound() {
        let p = points();
        let mut cache = BoxCache::new();

        cache.bbox(&p, &[0, p.len()]);
    }
}
