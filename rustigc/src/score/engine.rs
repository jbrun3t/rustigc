// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Branch-and-bound search scoring a flight against a ruleset.
//!
//! A run evaluates every rule of the ruleset at once: candidates of all rules share one heap, the
//! caches and the pruning floor.
//!
//! **Branch.** A [`Candidate`] holds one `[start, end]` fix-index range per turnpoint. A split cuts
//! one of them, giving two children.
//!
//! **Bound.** `Shape::bound` maximises the distance over the vertices of the candidate bounding
//! boxes, under whatever its own geometry also constrains.
//!
//! **Peek.** `Shape::peek` measures the distance through one real fix of each range: a score some
//! flight reaches.
//!
//! **Floor.** The highest score found so far, all rules included.

use crate::TrackError;
use crate::utils::geometry::{
    AntimeridianCheck, AntimeridianUnwrap, BBox, PointCoords, SPoint, Vertices,
};
use log::{debug, info};
use std::cmp::{Ordering, max, min};
use std::collections::BinaryHeap;

use crate::utils::round_km;

use super::cache::{Caches, Leg, Terminals};
use super::rules::{Rule, Ruleset};
use super::shapes::margin::Band;
use super::{ScoringResult, round_mm};

// How many times larger a range's box area must be than the widest range's to take the split.
// Tuned on runtime only.
const AREA_OVERRIDE: f64 = 2.5;

/// A node of the search: one fix-index range per turnpoint.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub ranges: Vec<[usize; 2]>,
    leaf: bool,
}

impl Candidate {
    fn rlen(&self, i: usize) -> usize {
        self.ranges[i][1].saturating_sub(self.ranges[i][0])
    }

    /// Whether every range is down to one fix.
    fn is_leaf(&self) -> bool {
        self.leaf
    }

    pub fn first_last_overlap(&self) -> bool {
        let first = self.ranges.first().unwrap();
        let last = self.ranges.last().unwrap();
        first[1] >= last[0]
    }

    /// First fix of each range: the turnpoints, on a leaf.
    pub(crate) fn positions(&self) -> Vec<usize> {
        self.ranges.iter().map(|r| r[0]).collect()
    }

    pub(crate) fn new(mut ranges: Vec<[usize; 2]>) -> Self {
        let len = ranges.len() - 1;

        // Turnpoints are flown in order: TP(n+1) cannot start before TP(n), and TP(n) cannot end
        // after TP(n+1). Trimming both keeps any fix sequence picked from the ranges chronological.
        for i in 0..len {
            ranges[i + 1][0] = max(ranges[i][0], ranges[i + 1][0]);
        }
        for i in (0..len).rev() {
            ranges[i][1] = min(ranges[i][1], ranges[i + 1][1]);
        }

        let leaf = ranges.iter().all(|r| r[1] <= r[0]);

        Candidate { ranges, leaf }
    }

    /// Root of the search: every turnpoint over the whole window.
    fn initial(size: usize, c: usize) -> Self {
        debug_assert!(c >= 1);

        Self::new(vec![[0, size - 1]; c])
    }

    /// Splits one range at its midpoint: the widest, unless another range's box is more than
    /// [`AREA_OVERRIDE`] times larger by area, in degrees squared.
    ///
    /// The choice only steers the search, never its result. A gap in the track inflates a box well
    /// beyond its fix count; splitting it first keeps the search from grinding down to the leaves.
    fn split(&self, scorer: &Scorer) -> [Self; 2] {
        debug_assert!(!self.leaf);

        let mut best = 0;
        for i in 1..self.ranges.len() {
            if self.rlen(i) > self.rlen(best) {
                best = i;
            }
        }

        let area = |range: &[usize; 2]| {
            // A cache hit: `eval` built these boxes.
            let b = scorer.bbox(range);
            (b.max.x() - b.min.x()) * (b.max.y() - b.min.y())
        };

        let mut largest = area(&self.ranges[best]);
        for i in 0..self.ranges.len() {
            if self.rlen(i) == 0 {
                continue;
            }

            let a = area(&self.ranges[i]);
            if a > largest * AREA_OVERRIDE {
                best = i;
                largest = a;
            }
        }

        let mid = (self.ranges[best][0] + self.ranges[best][1]) / 2;

        let mut left = self.ranges.clone();
        left[best][1] = mid;
        let mut right = self.ranges.clone();
        right[best][0] = mid + 1;

        [Self::new(left), Self::new(right)]
    }
}

/// A candidate on the heap, ordered by `bound`: its rule's score at the candidate's upper bound.
#[derive(Debug)]
struct Solution<'r> {
    bound: f64,
    candidate: Candidate,
    rule: &'r Rule,
}

impl Solution<'_> {
    fn is_leaf(&self) -> bool {
        self.candidate.is_leaf()
    }
}

impl Ord for Solution<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bound.total_cmp(&other.bound)
    }
}

impl PartialOrd for Solution<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Solution<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.bound == other.bound
    }
}

impl Eq for Solution<'_> {}

/// A scoring window, prepared once and searched by [`Scorer::solve`].
///
/// [`Log::score`] does both steps in one call. Build a `Scorer` to score one window under several
/// rulesets: the fix layout, the caches and the latitude band are then set up once.
///
/// Any [`PointCoords`] slice will do, no [`Log`] needed. `x` is longitude, `y` latitude, decimal
/// degrees, in flight order.
///
/// ```no_run
/// use rustigc::{Log, Ruleset, Scorer};
///
/// let log = Log::new(&std::fs::read("flight.igc")?)?;
/// let scorer = Scorer::new(&log.track, 125, 25425)?;
///
/// for id in rustigc::preset_names() {
///     let ruleset = Ruleset::preset(id)?;
///     println!("{id}: {:?}", scorer.solve(&ruleset).map(|r| r.score));
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [`Log::score`]: crate::Log::score
/// [`Log`]: crate::Log
/// [`PointCoords`]: crate::PointCoords
#[derive(Debug)]
pub struct Scorer {
    pub(super) track: Vec<SPoint>,
    pub(super) caches: Caches,
    band: Band,
    offset: usize,
}

/// Rejects fewer than two points, or a point that is not a finite latitude within ±90 and longitude
/// within ±180. A NaN would break the heap's ordering, a longitude past 180 the antimeridian check.
fn check_points<P: PointCoords<f64>>(track: &[P]) -> Result<(), TrackError> {
    if track.len() < 2 {
        return Err(TrackError::TooFewPoints {
            points: track.len(),
        });
    }

    match track.iter().position(|p| {
        !(p.y().is_finite()
            && p.y().abs() <= 90.0
            && p.x().is_finite()
            && p.x().abs() <= 180.0)
    }) {
        Some(index) => Err(TrackError::BadCoordinate { index }),
        None => Ok(()),
    }
}

impl Scorer {
    /// Latitude band of a window, from the furthest its fixes reach from the equator.
    fn band(track: &[SPoint]) -> Band {
        let lat = track
            .iter()
            .fold(0f64, |worst, fix| worst.max(fix.y().abs()));
        Band::of(lat)
    }

    /// Relative FCC-vs-geodesic slack to allow on `distance`, for this window's latitudes.
    pub(super) fn margin(&self, distance: f64) -> f64 {
        self.band.margin(distance)
    }

    /// `BBox` covering `range`.
    pub(super) fn bbox(&self, range: &[usize; 2]) -> BBox<SPoint> {
        self.caches.bbox(self, range)
    }

    /// Closest pair of fixes between `[0, entry_stop]` and `[exit_start, last]`, as
    /// `(entry, exit, distance)`.
    pub(super) fn closing(
        &self,
        entry_stop: usize,
        exit_start: usize,
    ) -> (usize, usize, f64) {
        self.caches.closing(self, entry_stop, exit_start)
    }

    /// Reach of `leg` from each vertex of `bbox`, the turnpoint box covering `range`.
    pub(super) fn terminals(
        &self,
        leg: Leg,
        range: &[usize; 2],
        bbox: &BBox<SPoint>,
        vertices: &Vertices<SPoint>,
    ) -> Terminals {
        self.caches.terminals(self, leg, range, bbox, vertices)
    }

    /// Fix `leg` lands on when it hangs off the turnpoint at index `tp`.
    pub(super) fn end(&self, leg: Leg, tp: usize) -> usize {
        self.caches.end(self, leg, tp)
    }

    /// Evaluates `candidate` under `rule`: the solution to push, and the score the floor may rise
    /// to. `None` for a non-leaf that cannot reach `floor` even with no gap.
    fn eval<'r>(
        &self,
        rule: &'r Rule,
        candidate: Candidate,
        floor: f64,
    ) -> Option<(Solution<'r>, f64)> {
        let shape = rule.shape();

        let leaf = candidate.is_leaf();
        let bound = shape.bound(self, &candidate);

        // A gap of 0 is the most a rule can be given, so this prunes on the real gap too.
        if !leaf && rule.score(bound, 0.0) < floor {
            return None;
        }

        let gap = shape.gap(self, &candidate);
        let upper = rule.score(bound, gap);
        let solution = Solution {
            candidate,
            bound: upper,
            rule,
        };

        // Cannot raise the floor: skip `peek`.
        if solution.bound < floor {
            return Some((solution, upper));
        }

        // `peek` on a leaf too: only `peek` checks a balanced circuit's ratio with the margin.
        // TODO: Return None early if bound is re-usable
        let dist = shape.peek(self, &solution.candidate);

        // Shrink the distance and grow the gap by the margin, so the floor never rests on a score
        // the geodesic re-measure would refuse.
        let lower = rule.score(
            dist * (1.0 - self.margin(dist)),
            gap * (1.0 + self.margin(gap)),
        );

        // A floor above the bound would prune the optimum.
        debug_assert!(upper >= lower);

        Some((solution, lower))
    }

    /// Whether a solution is worth keeping: its bound reaches the floor and is above 0. The second
    /// test matters while the floor is 0, when candidates that can never score would fill the heap.
    fn retain(bound: f64, floor: f64) -> bool {
        !(bound < floor || bound <= 0.0)
    }

    /// One root solution per rule, and the floor they reach.
    fn seed<'r>(&self, rules: &'r [Rule]) -> (Vec<Solution<'r>>, f64) {
        let mut floor = 0.0;
        let mut solutions = Vec::new();

        for rule in rules {
            let candidate = Candidate::initial(self.track.len(), rule.cardinality());
            if let Some(result) = self.eval(rule, candidate, floor) {
                info!(
                    "Seed for {:?}: Bound = {:.03}, Score = {:.03}",
                    rule, result.0.bound, result.1
                );

                solutions.push(result.0);

                floor = floor.max(result.1);
            }
        }

        (solutions, floor)
    }

    /// Splits `solution` and evaluates both children; returns them with the highest floor they
    /// reach.
    fn split<'r>(
        &self,
        solution: Solution<'r>,
        floor: f64,
    ) -> ([Option<Solution<'r>>; 2], f64) {
        let rule = solution.rule;
        let mut highest = floor;

        let children = solution.candidate.split(self).map(|candidate| {
            if let Some((child, lower)) = self.eval(rule, candidate, floor) {
                debug!(
                    "Split for {:?}: Bound = {:.03}, Score = {:.03} // {:?}",
                    rule, child.bound, floor, child.candidate
                );
                highest = highest.max(lower);
                Some(child)
            } else {
                None
            }
        });

        (children, highest)
    }

    /// Re-measures the winning leaf on the geodesic, scores it under its rule, and builds the
    /// result in the caller's track indices. `None` when the shape rejects the re-measure, or the
    /// rule scores it 0.
    fn report(&self, ruleset: &Ruleset, solution: &Solution) -> Option<ScoringResult> {
        let rule = solution.rule;

        let shape = rule.shape();
        let task = shape.report(self, &solution.candidate)?;
        let scored = rule.scored(task.distance, task.gap);

        if scored.score <= 0.0 {
            return None;
        }

        Some(ScoringResult {
            ruleset: ruleset.name().to_string(),
            description: scored.description.to_string(),
            distance_km: round_km(scored.distance),
            distance_m: round_mm(scored.distance),
            gap_m: round_mm(task.gap),
            threshold_m: round_mm(scored.threshold),
            penalty: round_km(scored.penalty),
            score: round_km(scored.score),
            multiplier: scored.multiplier,
            takeoff: self.offset,
            entry: task.entry + self.offset,
            turnpoints: task.turnpoints.iter().map(|tp| tp + self.offset).collect(),
            exit: task.exit + self.offset,
            landing: self.offset + self.track.len() - 1,
            circuit: task.circuit,
        })
    }

    /// Prepares the fixes in `[start, stop]` for searching.
    ///
    /// Checks the window, not the coordinates, which must be finite degrees within range; use
    /// [`Scorer::from_slice`] or [`Scorer::from_vec`] for points not checked yet.
    ///
    /// # Errors
    ///
    /// [`TrackError::InvalidWindow`] when `start` is at or past `stop`,
    /// [`TrackError::FixOutOfRange`] when `stop` reaches past `track`.
    pub fn new<P: PointCoords<f64>>(
        track: &[P],
        start: usize,
        stop: usize,
    ) -> Result<Self, TrackError> {
        if start >= stop {
            return Err(TrackError::InvalidWindow { start, stop });
        }
        if stop >= track.len() {
            return Err(TrackError::FixOutOfRange {
                index: stop,
                len: track.len(),
            });
        }

        let fixes = &track[start..=stop];

        Ok(Self::build(
            fixes.iter().map(|fix| [fix.y(), fix.x()]).collect(),
            start,
        ))
    }

    /// Prepares the whole of `track`, [`Scorer::new`] without a window.
    ///
    /// # Errors
    ///
    /// [`TrackError::TooFewPoints`] for fewer than two points,
    /// [`TrackError::BadCoordinate`] for one that is not a finite degree pair.
    pub fn from_slice<P: PointCoords<f64>>(track: &[P]) -> Result<Self, TrackError> {
        check_points(track)?;

        Self::new(track, 0, track.len() - 1)
    }

    /// Prepares an owned table of `[latitude, longitude]` points, in decimal degrees and in flight
    /// order.
    ///
    /// Takes the buffer as is, without copying it.
    ///
    /// # Errors
    ///
    /// [`TrackError::TooFewPoints`] for fewer than two points,
    /// [`TrackError::BadCoordinate`] for one that is not a finite degree pair.
    pub fn from_vec(track: Vec<[f64; 2]>) -> Result<Self, TrackError> {
        check_points(&track)?;

        Ok(Self::build(track, 0))
    }

    /// A scorer over `[lat, lon]` points starting `offset` fixes into the caller's track.
    /// Longitudes are unwrapped when the track crosses the antimeridian.
    fn build(mut track: Vec<SPoint>, offset: usize) -> Self {
        if track.crosses_antimeridian() {
            track = track.iter().copied().unwrapped().collect();
        }

        let caches = Caches::new(track.len());
        let band = Self::band(&track);

        Self {
            track,
            offset,
            caches,
            band,
        }
    }

    /// Scores the window against every rule of `ruleset` and reports the best.
    ///
    /// `None` when no rule scores the window.
    #[must_use]
    pub fn solve(&self, ruleset: &Ruleset) -> Option<ScoringResult> {
        let (solutions, mut floor) = self.seed(ruleset.rules());
        let mut heap: BinaryHeap<Solution> = solutions
            .into_iter()
            .filter(|s| Self::retain(s.bound, floor))
            .collect();

        // Hot path.
        loop {
            let Some(solution) = heap.pop() else {
                break None;
            };

            if solution.is_leaf() {
                // The geodesic re-measure may reject this leaf. The next best is still on the
                // heap: the floor never rose above a score the re-measure could withdraw.
                match self.report(ruleset, &solution) {
                    Some(result) => {
                        info!("Best {:?}", solution);
                        break Some(result);
                    }
                    None => {
                        info!("Rejected on the geodesic re-measure: {:?}", solution);
                        continue;
                    }
                }
            }

            let (children, score) = self.split(solution, floor);
            if score > floor {
                floor = score;
                heap.retain(|s| s.bound >= floor);

                info!("Floor rising: {:.03} // Heap size: {}", floor, heap.len());
            }

            heap.extend(
                children
                    .into_iter()
                    .flatten()
                    .filter(|s| Self::retain(s.bound, floor)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_vec_err(track: Vec<[f64; 2]>) -> TrackError {
        Scorer::from_vec(track).unwrap_err()
    }

    #[test]
    fn from_vec_rejects_short() {
        assert_eq!(
            from_vec_err(vec![[45.0, 6.0]]),
            TrackError::TooFewPoints { points: 1 }
        );
    }

    #[test]
    fn from_vec_rejects_nan() {
        assert_eq!(
            from_vec_err(vec![[45.0, 6.0], [f64::NAN, 6.1]]),
            TrackError::BadCoordinate { index: 1 }
        );
    }

    #[test]
    fn from_vec_rejects_infinite() {
        assert_eq!(
            from_vec_err(vec![[45.0, 6.0], [45.1, f64::INFINITY]]),
            TrackError::BadCoordinate { index: 1 }
        );
    }

    #[test]
    fn from_vec_rejects_out_of_band() {
        assert_eq!(
            from_vec_err(vec![[45.0, 6.0], [90.5, 6.1]]),
            TrackError::BadCoordinate { index: 1 }
        );
    }

    #[test]
    fn from_slice_rejects_empty() {
        assert_eq!(
            Scorer::from_slice::<SPoint>(&[]).unwrap_err(),
            TrackError::TooFewPoints { points: 0 }
        );
    }

    #[test]
    fn from_slice_rejects_short() {
        assert_eq!(
            Scorer::from_slice(&[[45.0, 6.0]]).unwrap_err(),
            TrackError::TooFewPoints { points: 1 }
        );
    }

    #[test]
    fn from_slice_rejects_bad_coordinate() {
        assert_eq!(
            Scorer::from_slice(&[[45.0, 6.0], [f64::NAN, 6.1]]).unwrap_err(),
            TrackError::BadCoordinate { index: 1 }
        );
    }

    #[test]
    fn new_rejects_inverted_window() {
        let track = [[45.0, 6.0], [45.1, 6.1], [45.2, 6.2]];

        assert_eq!(
            Scorer::new(&track, 2, 1).unwrap_err(),
            TrackError::InvalidWindow { start: 2, stop: 1 }
        );
        assert_eq!(
            Scorer::new(&track, 0, 3).unwrap_err(),
            TrackError::FixOutOfRange { index: 3, len: 3 }
        );
    }
}
