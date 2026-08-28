// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Branch-and-bound search scoring a flight against a league.
//!
//! A run evaluates every rule of the league at once: candidates of all rules share the
//! heap, the caches and the pruning floor, so the weaker rules get discarded early instead of each
//! paying for a full search of its own.
//!
//! **Branch.** A [`Candidate`] holds one `[start, end]` fix-index range per turnpoint. A split cuts
//! one of them, giving two children.
//!
//! **Bound.** `Shape::bound` maximises the distance over the vertices of the candidate bounding
//! boxes, under whatever its own geometry also constrains.
//!
//! **Score.** `Shape::peek` measures the distance through one real fix of each range.
//!
//! **Floor.** The highest score found so far, all rules included.

use crate::utils::geometry::{
    AntimeridianCheck, AntimeridianUnwrap, BBox, PointCoords, SPoint, Vertices,
};
use log::{debug, info};
use std::cmp::{max, min, Ordering};
use std::collections::BinaryHeap;

use crate::utils::round_km;

use super::cache::{Caches, Leg, Terminals};
use super::rules::{league_rules, Rule};
use super::shapes::margin::Band;
use super::{round_mm, ScoringResult};

// How much larger a range's box must be to take the split from the widest range.
// There is not much to back this number over another, other than a slightly better
// runtime.
const AREA_OVERRIDE: f64 = 2.5;

/// A node of the search: one fix-index range per turnpoint
#[derive(Debug, Clone)]
pub struct Candidate {
    pub ranges: Vec<[usize; 2]>,
    leaf: bool,
}

impl Candidate {
    fn rlen(&self, i: usize) -> usize {
        self.ranges[i][1].saturating_sub(self.ranges[i][0])
    }

    // Not needed but kept for symmetry with `Solution`
    fn is_leaf(&self) -> bool {
        self.leaf
    }

    pub fn first_last_overlap(&self) -> bool {
        let first = self.ranges.first().unwrap();
        let last = self.ranges.last().unwrap();
        first[1] >= last[0]
    }

    // Mostly useful on leafs
    pub(crate) fn positions(&self) -> Vec<usize> {
        self.ranges.iter().map(|r| r[0]).collect()
    }

    pub(crate) fn new(mut ranges: Vec<[usize; 2]>) -> Self {
        let len = ranges.len() - 1;

        // Turnpoints are flown in order, so trim the impossible parts of each range: TP(n+1) cannot
        // start before TP(n) does, TP(n) cannot end after TP(n+1) does. This keeps any fix sequence
        // picked from the ranges chronological.
        for i in 0..len {
            ranges[i + 1][0] = max(ranges[i][0], ranges[i + 1][0]);
        }
        for i in (0..len).rev() {
            ranges[i][1] = min(ranges[i][1], ranges[i + 1][1]);
        }

        let leaf = ranges.iter().all(|r| r[1] <= r[0]);

        Candidate { ranges, leaf }
    }

    /// Root of the search: every turnpoint over the whole window
    fn initial(size: usize, c: usize) -> Self {
        debug_assert!(c >= 1);

        Self::new(vec![[0, size - 1]; c])
    }

    /// Splits a range at its midpoint: the widest one, unless another covers a much larger area.
    ///
    /// The areas are degrees squared on purpose, for faster operation. This is just heuristic and does
    /// not affect correctness. It helps the B&B unglue itself when there is big gap/jump in the track.
    /// In such case the diagonal of a box is bigger than it should, forcing the B&B to grind to the leaf.
    /// The area trick solve that problem by forcing a split on the gap.
    fn split(&self, scorer: &Scorer) -> [Self; 2] {
        debug_assert!(!self.leaf);

        // Find largest range in the candidate
        let mut best = 0;
        for i in 1..self.ranges.len() {
            if self.rlen(i) > self.rlen(best) {
                best = i;
            }
        }

        // ... then hand the split over to a range whose box dwarfs it, if there is one
        let area = |range: &[usize; 2]| {
            // `eval` already built these, so this is a cache hit
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

#[derive(Debug, Clone)]
struct Solution {
    bound: f64,
    candidate: Candidate,
    rule: &'static dyn Rule,
}

impl Solution {
    fn is_leaf(&self) -> bool {
        self.candidate.is_leaf()
    }
}

impl Ord for Solution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bound.total_cmp(&other.bound)
    }
}

impl PartialOrd for Solution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Solution {
    fn eq(&self, other: &Self) -> bool {
        self.bound == other.bound
    }
}

impl Eq for Solution {}

/// A scoring window, prepared once and searched by [`Scorer::solve`].
///
/// [`Log::score`] does both steps in one call. Build a `Scorer` to score one window under several
/// leagues: the fix layout, the caches and the latitude band are then set up once.
///
/// Any [`PointCoords`] slice will do, no [`Log`] needed. `x` is longitude, `y` latitude, decimal
/// degrees, in flight order.
///
/// ```no_run
/// use rustigc::{Log, Scorer};
///
/// let log = Log::new(&std::fs::read("flight.igc")?)?;
/// let scorer = Scorer::new(&log.track, 125, 25425).expect("unusable window");
///
/// for league in rustigc::league_names() {
///     println!("{league}: {:?}", scorer.solve(league).map(|r| r.score));
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

impl Scorer {
    /// Latitude band of a window, from the furthest its fixes reach from the equator.
    fn band(track: &[SPoint]) -> Band {
        let lat = track
            .iter()
            .fold(0f64, |worst, fix| worst.max(fix.y().abs()));
        Band::of(lat)
    }

    /// FCC-vs-geodesic slack to allow on `distance`
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

    /// Evaluates a candidate, returning the `Solution` to push and its score.
    fn eval(
        &self,
        rule: &'static dyn Rule,
        candidate: Candidate,
        floor: f64,
    ) -> Option<(Solution, f64)> {
        let shape = (rule.shape())();

        let leaf = candidate.is_leaf();
        let bound = shape.bound(self, &candidate);

        // A gap of 0 is the most favourable a rule can be given, so a candidate
        // rejected here cannot score above the floor with its real gap either.
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

        // Cannot raise the floor, so skip the work of measuring it.
        if solution.bound < floor {
            return Some((solution, upper));
        }

        // `peek` on a leaf too, though `bound` agrees with it on the distance there.
        // Only `peek` tests a shape's constraints with the Fcc-vs-geodesic margin for balanced
        // TODO: Return None early if bound is re-usable
        let dist = shape.peek(self, &solution.candidate);

        // Add slack on peeked distance to account for Fcc-vs-Geodesic distance difference.
        // Ensure a solution valid with Fcc but invalid with Geodesic does not poison the heap.
        let lower = rule.score(
            dist * (1.0 - self.margin(dist)),
            gap * (1.0 + self.margin(gap)),
        );

        // A score higher than the bound would poison the B&B. If it happens, something is broken
        debug_assert!(upper >= lower);

        Some((solution, lower))
    }

    /// Whether a solution is worth keeping.
    fn retain(bound: f64, floor: f64) -> bool {
        // NOTE: Rejecting a 0 bound matters while the floor is still 0:
        // Candidates which can never score (cannot close, ...) would
        // otherwise pollute the heap until the floor rises.
        !(bound < floor || bound <= 0.0)
    }

    /// Seeds one initial solution per rule, and the best floor they provide.
    fn seed(&self, rules: &[&'static dyn Rule]) -> (Vec<Solution>, f64) {
        let mut floor = 0.0;
        let mut solutions = Vec::new();

        for &rule in rules {
            let candidate = Candidate::initial(self.track.len(), rule.cardinality());
            if let Some(result) = self.eval(rule, candidate, floor) {
                info!(
                    "Seed for {:?}: Bound = {:.03}, Score = {:.03}",
                    rule, result.0.bound, result.1
                );

                solutions.push(result.0);

                // NOTE: the first and last bounding boxes necessarily overlap at this stage,
                // so every floor should be 0 here. Keep it in case the overlap handling
                // changes.
                floor = floor.max(result.1);
            }
        }

        (solutions, floor)
    }

    /// Branching part of the B&B: evaluates both children and returns them with the floor they
    /// raise, if any.
    fn split(&self, solution: Solution, floor: f64) -> ([Option<Solution>; 2], f64) {
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

    /// Rebuilds the winning geometry, lets the rule price it, and assembles the report,
    /// shift the index space back out of the window to the track reference.
    ///
    /// `None` when the geodesic re-measure rejects what the FCC search accepted.
    fn report(&self, solution: &Solution) -> Option<ScoringResult> {
        let rule = solution.rule;

        let shape = (rule.shape())();
        let task = shape.report(self, &solution.candidate)?;
        let scored = rule.scored(task.distance, task.gap);

        let distance = round_km(scored.distance);
        if distance == 0.0 {
            return None;
        }

        Some(ScoringResult {
            league: scored.league.to_string(),
            description: scored.description.to_string(),
            distance_km: distance,
            distance_m: round_mm(scored.distance),
            gap_km: round_km(task.gap),
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
    /// `None` when the window is empty, inverted, or reaches past `track`.
    ///
    /// Collects the fixes into the cache-friendly layout the search reads, with longitude
    /// "unwrapped" so the antimeridian is no longer an issue.
    pub fn new<P: PointCoords<f64>>(
        track: &[P],
        start: usize,
        stop: usize,
    ) -> Option<Self> {
        if start >= stop || stop >= track.len() {
            return None;
        }

        let fixes = &track[start..=stop];
        let points = fixes.iter().map(|fix| [fix.y(), fix.x()]);
        let track: Vec<SPoint> = if fixes.crosses_antimeridian() {
            points.unwrapped().collect()
        } else {
            points.collect()
        };
        let caches = Caches::new(track.len());
        let band = Self::band(&track);

        Some(Self {
            track,
            offset: start,
            caches,
            band,
        })
    }

    /// Scores the window against every rule of `league` and reports the best.
    ///
    /// `None` when `league` is not one of [`league_names`], or when no rule could score the
    /// window.
    ///
    /// Main B&B loop. Evaluating the rules together is important to quickly discard the least
    /// performing ones.
    ///
    /// [`league_names`]: crate::league_names
    pub fn solve(&self, league: &str) -> Option<ScoringResult> {
        let rules = league_rules(league)?;

        let (solutions, mut floor) = self.seed(rules);
        let mut heap: BinaryHeap<Solution> = solutions
            .into_iter()
            .filter(|s| Self::retain(s.bound, floor))
            .collect();

        // Critical performance hot-path
        let result = loop {
            let Some(solution) = heap.pop() else {
                break None;
            };

            if solution.is_leaf() {
                // The geodesic re-measure can reject what the FCC search accepted. Take the next
                // best leaf instead of giving up: this is not an error, and the floor was never
                // raised above what a re-measure can withdraw, so the heap still holds the rest.
                match self.report(&solution) {
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
        };

        result
    }
}
