//! DP shared by every shape.

use crate::geometry::{Fcc, PointDistance, SPoint, Vertices};

use super::{Candidate, Scorer};
use super::{Path, Transitions};

/// Leg distances between two turnpoint boxes, destination-major: a row holds every leg reaching one
/// vertex of the destination box, which is the layer [`dynamic_search`] carries forward.
#[derive(Debug, Clone, Copy)]
pub struct TransitionMatrix {
    cells: [f64; 16],
    rows: usize,
    cols: usize,
}

impl TransitionMatrix {
    /// Every leg from a vertex of `src` to a vertex of `dst`.
    pub fn new(src: &[SPoint], dst: &[SPoint]) -> Self {
        let (rows, cols) = (dst.len(), src.len());
        let mut cells = [0f64; 16];

        // Row `i` starts at `i * cols`, which is what `rows` relies on to cut them back apart.
        for (row, to) in cells.chunks_exact_mut(cols).zip(dst) {
            for (cell, from) in row.iter_mut().zip(src) {
                *cell = Fcc::distance(from, to);
            }
        }

        TransitionMatrix { cells, rows, cols }
    }

    /// One row per destination vertex, each holding one leg per source vertex.
    pub fn rows(&self) -> impl ExactSizeIterator<Item = &[f64]> {
        self.cells[..self.rows * self.cols].chunks_exact(self.cols)
    }
}

pub fn candidate_to_path(scorer: &Scorer, c: &Candidate) -> Path {
    c.ranges.iter().map(|r| scorer.bbox(r).vertices()).collect()
}

/// Consecutive pairs of `s`; `wraps` adds the last-to-first one, closing a circuit.
pub fn pairs<T>(s: &[T], wraps: bool) -> impl Iterator<Item = (&T, &T)> + '_ {
    s.iter()
        .zip(s.iter().cycle().skip(1))
        .take(if wraps { s.len() } else { s.len() - 1 })
}

/// Dynamic Programming (DP) path search  — through one vertex of each turnpoint box, in order.
pub fn dynamic_search<F>(
    entry: &[f64],
    exit: &[f64],
    transitions: &[TransitionMatrix],
    pick: &F,
) -> f64
where
    F: Fn(f64, f64) -> f64,
{
    // A layer carries one distance per vertex of the box it stands on, so at most 4.
    // `entry` and `exit` hold the terminal cost of every vertex of the first and last box
    let mut distances = [f64::NAN; 4];
    let mut next = [f64::NAN; 4];
    let mut len = entry.len();

    distances[..len].copy_from_slice(entry);

    for tb in transitions {
        // Get the transition cost from each vertex of TPb to the all of the previous
        // TPa vertices
        for (i, tbv) in tb.rows().enumerate() {
            debug_assert_eq!(len, tbv.len());

            next[i] = tbv
                .iter()
                .enumerate()
                .fold(f64::NAN, |acc, (j, tba)| pick(acc, *tba + distances[j]));
        }

        len = tb.rows().len();
        std::mem::swap(&mut distances, &mut next);
    }

    debug_assert_eq!(len, exit.len());

    distances[..len]
        .iter()
        .zip(exit.iter())
        .map(|(d, e)| *d + *e)
        .fold(f64::NAN, pick)
}

/// One [`TransitionMatrix`] per leg of `iter`, in order.
pub fn transition_matrices<'a, I>(iter: I, circuit: bool) -> Transitions
where
    I: Iterator<Item = (&'a Vertices<SPoint>, &'a Vertices<SPoint>)>,
{
    // `circuit` transposes the first one so that its rows are indexed by a vertex of TP1
    // rather than of TP2: `circuit_dynamic_search` enumerates the origins, since a
    // circuit's closing leg depends on where it started.
    let mut flip = circuit;
    let mut transition = Transitions::new();

    for pair in iter {
        // Flipping the pair transposes the matrix, cheaper than doing it afterwards.
        let (a, b) = if flip {
            (pair.1, pair.0)
        } else {
            (pair.0, pair.1)
        };

        transition.push(TransitionMatrix::new(a, b));
        flip = false;
    }

    transition
}
