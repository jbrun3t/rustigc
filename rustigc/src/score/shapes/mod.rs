// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! The geometries a flight can be scored over. A rule names one and prices what it measures.

mod circuit;
mod common;

pub mod balanced;
pub mod closed;
pub mod margin;
pub mod polyline;

use smallvec::SmallVec;
use std::fmt::Debug;

use crate::score::Scorer;
use crate::score::engine::Candidate;
use crate::utils::geometry::{SPoint, Vertices};
use common::TransitionMatrix;

pub trait ShapeBound: Debug {
    /// Upper bound of the distance reachable within this geometry. Hot-path.
    fn bound(&self, scorer: &Scorer, c: &Candidate) -> f64;
}

pub trait ShapePeek: Debug {
    /// FCC distance the flight actually reaches, through one real fix of each range, or 0 when no
    /// such path meets the shape's own constraints. Floors the shared search. Hot-path.
    fn peek(&self, scorer: &Scorer, c: &Candidate) -> f64;
}

/// The task a candidate flew, on the geodesic. Which of its fixes are turnpoints and which are the
/// ends is the shape's call, not the rule's.
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
    /// Entry and exit fixes of this candidate: the closing points of a circuit, the furthest points
    /// around an open polyline. Hot-path.
    fn endpoints(&self, scorer: &Scorer, c: &Candidate) -> (usize, usize);
    /// Closing leg of a circuit — a raw distance, not a penalty. 0 when there is no closing leg, or
    /// when the closing search was skipped. Hot-path.
    fn gap(&self, scorer: &Scorer, c: &Candidate) -> f64;
    /// Cold-path, once on the winner. `None` when the geodesic re-measure fails the shape's own
    /// constraints.
    fn report(&self, scorer: &Scorer, c: &Candidate) -> Option<Task>;
}

pub trait Shape: ShapeCommon + ShapePeek + ShapeBound {}
impl<T: ShapeCommon + ShapePeek + ShapeBound> Shape for T {}

/// A geometry a rule can name, e.g. `ClosedCircuit<3>`.
///
/// A cardinality floor is asserted inside `CARDINALITY`. Being post-monomorphization, it fails
/// `cargo build` but not `cargo check`.
pub trait ShapeKind: Shape + Sized + Default + 'static {
    /// Fix-index ranges the search branches on. Needed to size the root candidate, before any
    /// shape exists.
    const CARDINALITY: usize;

    /// Shapes are stateless ZSTs, so the box never reaches the allocator.
    fn create() -> Box<dyn Shape> {
        Box::new(Self::default())
    }
}

/// Each geometry's `create`, e.g. `ClosedCircuit::<3>::create`.
pub type ShapeBuilder = fn() -> Box<dyn Shape>;

/// The legs of a shape, one matrix each. At most 4.
pub type Transitions = SmallVec<[TransitionMatrix; 4]>;

/// One turnpoint box's vertices per turnpoint, in order. At most 4 turnpoints.
pub type Path = SmallVec<[Vertices<SPoint>; 4]>;
