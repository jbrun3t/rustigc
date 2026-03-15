//! Geometry primitives and operations
//!
//! This module provides generic 2D geometry support for both Euclidean and spherical
//! coordinate systems. Euclidean points are represented as plain `[T; 2]` arrays for
//! zero-cost abstraction and direct kdtree compatibility.

use num_traits::Float;

// ============= Traits =============

/// Provides access to 2D coordinates
pub trait Coords<T: Float> {
    /// X coordinate (or longitude for spherical)
    fn x(&self) -> T;

    /// Y coordinate (or latitude for spherical)
    fn y(&self) -> T;
}

/// Geometric operations on points (distance, bearing)
pub trait PointOps<T: Float>: Coords<T> {
    /// Distance between this point and another
    fn distance(&self, other: &Self) -> T;

    /// Bearing from this point to another (degrees, 0=North, 90=East)
    fn bearing(&self, other: &Self) -> T;
}

/// Constructible 2D point from coordinates
pub trait Point2D<T: Float>: Coords<T> + Copy {
    /// Create a point from coordinates [x, y]
    fn from_coords(coords: [T; 2]) -> Self;
}

// ============= Euclidean: plain [T; 2] =============

impl<T: Float> Coords<T> for [T; 2] {
    fn x(&self) -> T {
        self[0]
    }

    fn y(&self) -> T {
        self[1]
    }
}

impl<T: Float> PointOps<T> for [T; 2] {
    fn distance(&self, other: &Self) -> T {
        let dx = self[0] - other[0];
        let dy = self[1] - other[1];
        (dx * dx + dy * dy).sqrt()
    }

    fn bearing(&self, other: &Self) -> T {
        let dx = self[0] - other[0];
        let dy = self[1] - other[1];
        let mut bearing = (-dx).atan2(-dy).to_degrees();
        // Normalize to [0, 360)
        if bearing < T::zero() {
            bearing = bearing + T::from(360).unwrap();
        }
        bearing
    }
}

impl<T: Float> Point2D<T> for [T; 2] {
    fn from_coords(coords: [T; 2]) -> Self {
        coords
    }
}

// ============= Spherical: lon/lat coordinates =============

/// Spherical point (longitude, latitude in degrees)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SphericalPoint<T> {
    pub lon: T,
    pub lat: T,
}

impl<T: Float> SphericalPoint<T> {
    pub fn new(lon: T, lat: T) -> Self {
        Self { lon, lat }
    }
}

impl<T: Float> Coords<T> for SphericalPoint<T> {
    fn x(&self) -> T {
        self.lon
    }

    fn y(&self) -> T {
        self.lat
    }
}

impl<T: Float> Point2D<T> for SphericalPoint<T> {
    fn from_coords(coords: [T; 2]) -> Self {
        use crate::projector::lon_round;
        Self {
            lon: lon_round(coords[0]),
            lat: coords[1],
        }
    }
}

// TODO: Implement PointOps for SphericalPoint with haversine distance and great circle bearing

// ============= Bounding Box =============

/// Axis-aligned bounding box
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BBox<P> {
    /// Bottom-left corner (or southwest for spherical)
    pub bl: P,
    /// Top-right corner (or northeast for spherical)
    pub tr: P,
}

impl<P> BBox<P> {
    /// Create bounding box from a slice of items with coordinates
    pub fn from_items<T, I>(items: &[I]) -> Option<Self>
    where
        T: Float,
        P: Point2D<T>,
        I: Coords<T>,
    {
        let mut iter = items.iter();
        let first = iter.next()?;
        let (first_x, first_y) = (first.x(), first.y());

        let (min, max) = iter.fold(
            ([first_x, first_y], [first_x, first_y]),
            |(mut min, mut max), item| {
                let (x, y) = (item.x(), item.y());
                min[0] = T::min(min[0], x);
                min[1] = T::min(min[1], y);
                max[0] = T::max(max[0], x);
                max[1] = T::max(max[1], y);
                (min, max)
            },
        );

        Some(BBox {
            bl: P::from_coords(min),
            tr: P::from_coords(max),
        })
    }

    /// Get the center point of the bounding box
    pub fn center<T>(&self) -> P
    where
        T: Float,
        P: Point2D<T>,
    {
        let two = T::from(2).unwrap();
        P::from_coords([
            (self.bl.x() + self.tr.x()) / two,
            (self.bl.y() + self.tr.y()) / two,
        ])
    }

    /// Diagonal length of the bounding box
    pub fn diagonal<T>(&self) -> T
    where
        T: Float,
        P: PointOps<T>,
    {
        self.bl.distance(&self.tr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_box() -> BBox<[f64; 2]> {
        let v = vec![[1.0, 1.0], [1.0, 3.0], [1.4, 2.7], [5.0, 1.0], [5.0, 3.0]];

        BBox::from_items(v.as_slice()).unwrap()
    }

    #[test]
    fn euclidean_distance() {
        let p1: [f64; 2] = [0.0, 0.0];
        let p2: [f64; 2] = [3.0, 4.0];
        assert_eq!(p1.distance(&p2), 5.0);
    }

    #[test]
    fn euclidean_bearing() {
        let p1: [f64; 2] = [0.0, 0.0];
        let p2: [f64; 2] = [0.0, 1.0]; // Due north
        assert_eq!(p1.bearing(&p2), 0.0);

        let p3: [f64; 2] = [1.0, 0.0]; // Due east
        assert_eq!(p1.bearing(&p3), 90.0);

        let p4: [f64; 2] = [0.0, -1.0]; // Due south
        assert_eq!(p1.bearing(&p4), 180.0);

        let p5: [f64; 2] = [-1.0, 0.0]; // Due west
        assert_eq!(p1.bearing(&p5), 270.0);
    }

    #[test]
    fn bbox_from_items() {
        let b = test_box();

        assert_eq!(b.bl, [1.0, 1.0]);
        assert_eq!(b.tr, [5.0, 3.0]);
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

        assert!((b.diagonal() - 4.472136).abs() < 0.00001);
    }

    #[test]
    fn bbox_empty() {
        let points: Vec<[f64; 2]> = vec![];
        assert!(BBox::<[f64; 2]>::from_items(&points).is_none());
    }

    #[test]
    fn bbox_single_point() {
        let points = vec![[2.0, 3.0]];
        let bbox = BBox::<[f64; 2]>::from_items(&points).unwrap();
        assert_eq!(bbox.bl, [2.0, 3.0]);
        assert_eq!(bbox.tr, [2.0, 3.0]);
        assert_eq!(bbox.diagonal(), 0.0);
    }
}
