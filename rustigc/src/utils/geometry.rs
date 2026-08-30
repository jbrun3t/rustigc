// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Geometry primitives and operations
//!
//! A point is a plain `[T; 2]` array — zero-cost, and what `rstar` already indexes. What the two
//! coordinates mean is the metric's business, not the point's: [`Flat`] reads them as a Euclidean
//! plane, [`Fcc`] and [`Geodesic`] as (lon, lat) on the Earth.

use crate::utils::projector::lon_round;
use geographiclib_rs::InverseGeodesic;
use num_traits::Float;
use smallvec::{SmallVec, smallvec};

// ============= Traits =============

/// Read access to 2D coordinates
pub trait PointCoords<T: Float> {
    /// X coordinate (or longitude for spherical)
    fn x(&self) -> T;

    /// Y coordinate (or latitude for spherical)
    fn y(&self) -> T;
}

/// Write access to 2D coordinates
pub trait PointSetCoords<T: Float> {
    /// X coordinate (or longitude for spherical)
    fn set_x(&mut self, x: T);

    /// Y coordinate (or latitude for spherical)
    fn set_y(&mut self, y: T);
}

pub trait PointDistance<T: Float> {
    fn distance<U: PointCoords<T>>(a: &U, b: &U) -> T;
}

#[allow(dead_code)]
pub trait PointBearing<T: Float> {
    fn bearing<U: PointCoords<T>>(a: &U, b: &U) -> T;
}

/// Constructible 2D point from coordinates
pub trait PointNew<T: Float>: PointCoords<T> + Copy {
    /// Create a point from coordinates (x, y)
    fn new(x: T, y: T) -> Self;
}

// ==== Euclidean Math ====

pub struct Flat;

impl Flat {
    /// Squared distance, for ranking only: monotone in [`Flat::distance`], without its `sqrt`.
    pub fn distance_squared<T: Float, U: PointCoords<T>>(a: &U, b: &U) -> T {
        let dx = b.x() - a.x();
        let dy = b.y() - a.y();
        dx * dx + dy * dy
    }
}

impl<T: Float> PointDistance<T> for Flat {
    fn distance<U: PointCoords<T>>(a: &U, b: &U) -> T {
        Flat::distance_squared(a, b).sqrt()
    }
}

impl<T: Float> PointBearing<T> for Flat {
    fn bearing<U: PointCoords<T>>(a: &U, b: &U) -> T {
        let dx = a.x() - b.x();
        let dy = a.y() - b.y();
        let mut bearing = (-dx).atan2(-dy).to_degrees();
        // Normalize to [0, 360)
        if bearing < T::zero() {
            bearing = bearing + T::from(360).unwrap();
        }
        bearing
    }
}

// ==== Earth Distance using the FCC Simplification ====

pub struct Fcc;

/// `cos` of a mean latitude in radians — **only valid on [-π/2, π/2]**, where it is a 6-term minimax
/// polynomial in `fm²` (Remez, degree 10). Latitudes parse within ±90° (`decode/utils.rs`), so their
/// mean cannot leave that range; outside it this diverges fast.
///
/// Worth 4.9× libm's `cos`, which spends most of its work on the argument reduction a general `cos`
/// owes any input and this one never needs. The 2.2e-10 it costs on `cos` lands ~0.8 mm per 100 km on
/// the distance, against the FCC formula's own ~7.9 m per 100 km versus the WGS84 geodesic — the two
/// errors add, so this is a 0.0001 % widening of a budget that was already there.
///
/// Hand-rolled because nothing on crates.io covers it: the f64 `cos`es on offer are general-purpose
/// and pay for that reduction, and `sleef-trig` — the only credible one — measured 0.97× libm here.
fn cos_lat<T: Float>(fm: T) -> T {
    let a0 = T::from(0.9999999997806517).unwrap();
    let a1 = T::from(-0.49999999358471703).unwrap();
    let a2 = T::from(0.04166663625806798).unwrap();
    let a3 = T::from(-0.0013888361400249912).unwrap();
    let a4 = T::from(2.4760161351450572e-05).unwrap();
    let a5 = T::from(-2.605149519760686e-07).unwrap();

    let u = fm * fm;
    a0 + u * (a1 + u * (a2 + u * (a3 + u * (a4 + u * a5))))
}

impl<T: Float> PointDistance<T> for Fcc {
    fn distance<U: PointCoords<T>>(a: &U, b: &U) -> T {
        let one = T::from(1).unwrap();
        let two = T::from(2).unwrap();

        let df = b.y() - a.y();
        let dg = lon_round(b.x() - a.x());
        let fm = ((a.y() + b.y()) / two).to_radians();

        let cos_fm = cos_lat(fm);
        let cos2fm = two * cos_fm * cos_fm - one;
        let cos3fm = cos_fm * (two * cos2fm - one);
        let cos4fm = two * cos2fm * cos2fm - one;
        let cos5fm = two * cos2fm * cos3fm - cos_fm;

        // the FCC formula as per 47 CFR 73.208
        let k1c1 = T::from(111.13209).unwrap();
        let k1c2 = T::from(0.56605).unwrap();
        let k1c3 = T::from(0.00120).unwrap();
        let k1 = k1c1 - k1c2 * cos2fm + k1c3 * cos4fm;

        let k2c1 = T::from(111.41513).unwrap();
        let k2c2 = T::from(0.09455).unwrap();
        let k2c3 = T::from(0.00012).unwrap();
        let k2 = k2c1 * cos_fm - k2c2 * cos3fm + k2c3 * cos5fm;

        let thousand = T::from(1000).unwrap();
        ((k1 * k1 * df * df) + (k2 * k2 * dg * dg)).sqrt() * thousand
    }
}

// ==== WGS84 Geodesic Math ====

pub struct Geodesic;

impl PointDistance<f64> for Geodesic {
    fn distance<U: PointCoords<f64>>(a: &U, b: &U) -> f64 {
        let g = geographiclib_rs::Geodesic::wgs84();
        let (dist, _, _, _) = g.inverse(a.y(), a.x(), b.y(), b.x());
        dist
    }
}

// ============= plain [T; 2] Point =============

// Stored `[y, x]`, so a geographic point reads `[lat, lon]`

impl<T: Float> PointCoords<T> for [T; 2] {
    #[inline]
    fn x(&self) -> T {
        self[1]
    }

    #[inline]
    fn y(&self) -> T {
        self[0]
    }
}

impl<T: Float> PointSetCoords<T> for [T; 2] {
    #[inline]
    fn set_x(&mut self, x: T) {
        self[1] = x;
    }

    #[inline]
    fn set_y(&mut self, y: T) {
        self[0] = y;
    }
}

impl<T: Float> PointNew<T> for [T; 2] {
    #[inline]
    fn new(x: T, y: T) -> Self {
        [y, x]
    }
}

pub type TPoint<T> = [T; 2];

/// The `f64` point the scoring engine (`score/`) and flight detection (`analysis/`) both work in.
pub type SPoint = TPoint<f64>;

/// A `BBox`'s vertices: at most 4, held inline so a box costs no allocation.
pub type Vertices<P> = SmallVec<[P; 4]>;

/// Longitudes made continuous across ±180: every point keeps the shortest step from the one before,
/// so a track straddling the antimeridian reads as one interval instead of two.
pub trait AntimeridianCheck<T: Float> {
    fn crosses_antimeridian(&self) -> bool;
}

impl<T: Float, P: PointCoords<T>> AntimeridianCheck<T> for [P] {
    fn crosses_antimeridian(&self) -> bool {
        let half = T::from(180).unwrap();

        self.windows(2).any(|w| (w[1].x() - w[0].x()).abs() > half)
    }
}

pub trait AntimeridianUnwrap<T: Float>: Iterator + Sized {
    fn unwrapped(self) -> impl Iterator<Item = Self::Item>;
}

impl<T: Float, I> AntimeridianUnwrap<T> for I
where
    I: Iterator,
    I::Item: PointCoords<T> + PointSetCoords<T>,
{
    // NOTE: This does not unroll well because of the loop dependency, so it is worth
    // checking if unwrapping is necessary before doing it
    fn unwrapped(self) -> impl Iterator<Item = Self::Item> {
        self.scan(None, |carry: &mut Option<(T, T)>, mut point| {
            let (lon, raw) = match *carry {
                Some((lon, raw)) => (lon + lon_round(point.x() - raw), point.x()),
                None => (point.x(), point.x()),
            };
            *carry = Some((lon, raw));
            point.set_x(lon);

            Some(point)
        })
    }
}

// ============= Bounding Box =============

/// Axis-aligned bounding box
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BBox<P> {
    /// Bottom-left corner (or southwest for spherical)
    pub min: P,
    /// Top-right corner (or northeast for spherical)
    pub max: P,
}

impl<P> BBox<P> {
    /// Create bounding box from a slice of items with coordinates
    pub fn from_items<T, I>(items: &[I]) -> Option<Self>
    where
        T: Float,
        P: PointNew<T>,
        I: PointCoords<T>,
    {
        let iter = items.iter();
        let first = items.first()?;

        // TODO: consider itertools minmax_by
        let (l, b, r, t) = iter.fold(
            (first.x(), first.y(), first.x(), first.y()),
            |(mut l, mut b, mut r, mut t), item| {
                let (x, y) = (item.x(), item.y());
                l = T::min(l, x);
                b = T::min(b, y);
                r = T::max(r, x);
                t = T::max(t, y);
                (l, b, r, t)
            },
        );

        Some(BBox {
            min: P::new(l, b),
            max: P::new(r, t),
        })
    }

    pub fn merge<T>(&mut self, other: &Self)
    where
        T: Float,
        P: PointSetCoords<T> + PointCoords<T>,
    {
        self.min.set_x(T::min(self.min.x(), other.min.x()));
        self.min.set_y(T::min(self.min.y(), other.min.y()));
        self.max.set_x(T::max(self.max.x(), other.max.x()));
        self.max.set_y(T::max(self.max.y(), other.max.y()));
    }

    /// Get the center point of the bounding box
    pub fn center<T>(&self) -> P
    where
        T: Float,
        P: PointNew<T>,
    {
        let two = T::from(2).unwrap();
        P::new(
            (self.min.x() + self.max.x()) / two,
            (self.min.y() + self.max.y()) / two,
        )
    }

    /// Diagonal length of the bounding box
    pub fn diagonal<T, M>(&self, _metric: M) -> T
    where
        T: Float,
        P: PointCoords<T>,
        M: PointDistance<T>,
    {
        M::distance(&self.min, &self.max)
    }

    pub fn vertices<T>(&self) -> Vertices<P>
    where
        T: Float,
        P: PointNew<T>,
    {
        if (self.min.x() == self.max.x()) && (self.min.y() == self.max.y()) {
            smallvec![self.min]
        } else if (self.min.x() == self.max.x()) || (self.min.y() == self.max.y()) {
            smallvec![self.min, self.max]
        } else {
            smallvec![
                self.min,
                self.max,
                P::new(self.min.x(), self.max.y()), // Top Left
                P::new(self.max.x(), self.min.y()), // Bottom Right
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_box() -> BBox<TPoint<f64>> {
        let v = [[1.0, 1.0], [1.0, 3.0], [1.4, 2.7], [5.0, 1.0], [5.0, 3.0]];

        BBox::from_items(v.as_slice()).unwrap()
    }

    // The polynomial is only sound on [-pi/2, pi/2] and only useful if it stays far under the FCC
    // formula's own error, so pin both ends of that: swept against libm across the whole domain.
    #[test]
    fn cos_lat_holds_its_bound_over_every_latitude() {
        const BOUND: f64 = 2.2e-10;

        let mut worst = 0f64;
        for i in 0..=2_000_000 {
            let fm = (-90.0 + 180.0 * (i as f64) / 2_000_000.0).to_radians();
            worst = worst.max((cos_lat(fm) - fm.cos()).abs());
        }

        assert!(worst <= BOUND, "max error {worst:e} exceeds {BOUND:e}");
        // A polynomial quietly replaced by something weaker should fail here, not silently degrade
        assert!(
            worst > BOUND / 100.0,
            "max error {worst:e} unexpectedly small — stale bound?"
        );
    }

    #[test]
    fn euclidean_distance() {
        let p1: TPoint<f64> = [0.0, 0.0];
        let p2: TPoint<f64> = [3.0, 4.0];
        assert_eq!(Flat::distance(&p1, &p2), 5.0);
    }

    #[test]
    fn euclidean_bearing() {
        let p1: TPoint<f64> = [0.0, 0.0];
        let p2: TPoint<f64> = [1.0, 0.0]; // Due north
        assert_eq!(Flat::bearing(&p1, &p2), 0.0);

        let p2: TPoint<f64> = [0.0, 1.0]; // Due east
        assert_eq!(Flat::bearing(&p1, &p2), 90.0);

        let p2: TPoint<f64> = [-1.0, 0.0]; // Due south
        assert_eq!(Flat::bearing(&p1, &p2), 180.0);

        let p2: TPoint<f64> = [0.0, -1.0]; // Due west
        assert_eq!(Flat::bearing(&p1, &p2), 270.0);
    }

    #[test]
    fn unwrapped_without_crossing() {
        let t = vec![[45.0, 6.9], [45.1, 7.0], [45.2, 7.1]];

        assert_eq!(t.iter().copied().unwrapped().collect::<Vec<_>>(), t);
    }

    #[test]
    fn unwrapped_with_crossing() {
        let t = [[45.0, 179.9], [45.0, -179.9], [45.0, 179.9]];
        let want = [[45.0, 179.9], [45.0, 180.1], [45.0, 179.9]];

        assert_eq!(t.iter().copied().unwrapped().collect::<Vec<_>>(), want);
    }

    #[test]
    fn bbox_from_items() {
        let b = test_box();

        assert_eq!(b.min, [1.0, 1.0]);
        assert_eq!(b.max, [5.0, 3.0]);
    }

    #[test]
    fn bbox_center() {
        let b = test_box();
        let center = b.center();

        assert_eq!(center, [3.0, 2.0]);
    }

    #[test]
    fn bbox_diagonal() {
        let b = test_box();

        assert!((b.diagonal(Flat) - 4.472136).abs() < 0.00001);
    }

    #[test]
    fn bbox_empty() {
        let points: Vec<TPoint<f64>> = vec![];
        assert!(BBox::<TPoint<f64>>::from_items(&points).is_none());
    }

    #[test]
    fn bbox_single_point() {
        let points = [[2.0, 3.0]];
        let bbox = BBox::<TPoint<f64>>::from_items(&points).unwrap();
        assert_eq!(bbox.min, [2.0, 3.0]);
        assert_eq!(bbox.max, [2.0, 3.0]);
        assert_eq!(bbox.diagonal(Flat), 0.0);
    }
}
