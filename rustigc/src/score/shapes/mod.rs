// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! The geometries a flight is scored over. A rule names one and scores the distance and gap it
//! measures.

mod circuit;
mod common;

pub mod balanced;
pub mod closed;
pub mod margin;
pub mod polyline;
pub mod table;

use serde::Deserialize;
use smallvec::SmallVec;
use std::fmt::Debug;

use crate::score::Scorer;
use crate::score::engine::Candidate;
use crate::utils::geometry::{SPoint, Vertices};
use common::TransitionMatrix;

pub trait ShapeBound: Debug {
    /// Upper bound of the distance any flight through `c`'s ranges reaches. On the hot path.
    fn bound(&self, scorer: &Scorer, c: &Candidate) -> f64;
}

pub trait ShapePeek: Debug {
    /// FCC distance through one real fix of each range, or 0 when that path breaks the shape's
    /// constraints. A score some flight reaches, so it raises the search floor. On the hot path.
    fn peek(&self, scorer: &Scorer, c: &Candidate) -> f64;
}

/// The task a leaf flies, measured on the geodesic, with its turnpoints and ends as track indices.
#[derive(Debug)]
pub struct Task {
    pub distance: f64,
    pub gap: f64,
    pub entry: usize,
    pub turnpoints: Vec<usize>,
    pub exit: usize,
    pub circuit: bool,
}

pub trait ShapeCommon: Debug {
    /// Entry and exit fixes: a circuit's closing pair, an open polyline's furthest points. On the
    /// hot path.
    fn endpoints(&self, scorer: &Scorer, c: &Candidate) -> (usize, usize);
    /// Closing leg of a circuit, as a distance. 0 for an open shape, and while a circuit's
    /// first and last ranges overlap. On the hot path.
    fn gap(&self, scorer: &Scorer, c: &Candidate) -> f64;
    /// The task `c` flies, re-measured on the geodesic; `None` when it then breaks the shape's
    /// constraints. Called once, on the winning leaf.
    fn report(&self, scorer: &Scorer, c: &Candidate) -> Option<Task>;
}

/// A geometry the search can bound, peek and report. `Sync` for the static [`table::SHAPES`].
pub trait Shape: ShapeCommon + ShapePeek + ShapeBound + Sync {}
impl<T: ShapeCommon + ShapePeek + ShapeBound + Sync> Shape for T {}

/// A geometry as a ruleset names it: the parameters that pick one monomorphisation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShapeSpec {
    /// Points of the polyline, the two ends included.
    OpenPolyline {
        points: usize,
    },
    ClosedCircuit {
        turnpoints: usize,
    },
    /// `min_side` is the shortest leg's share of the total in per mille, so 280 is the FAI 28 %.
    BalancedCircuit {
        turnpoints: usize,
        min_side: u32,
    },
}

/// A concrete geometry, e.g. `ClosedCircuit<3>`, listed in [`table::SHAPES`].
///
/// `CARDINALITY` asserts the geometry's minimum size; that assert fails `cargo build` but not
/// `cargo check`.
pub trait ShapeKind: Shape + Sized + 'static {
    /// Number of fix ranges the search branches on.
    const CARDINALITY: usize;

    /// Whether the geometry has a closing leg.
    const CLOSES: bool;

    /// How a ruleset names this geometry.
    const SPEC: ShapeSpec;

    /// The geometry's single, stateless instance; a `const` so a `static` can hold it.
    const NEW: Self;
}

/// The legs of a shape, one matrix each. At most 4.
pub type Transitions = SmallVec<[TransitionMatrix; 4]>;

/// One turnpoint box's vertices per turnpoint, in order. At most 4 turnpoints.
pub type Path = SmallVec<[Vertices<SPoint>; 4]>;
