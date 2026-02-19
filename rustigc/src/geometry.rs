use num_traits::Float;

pub fn delta_distance<T: Float>(dx: T, dy: T) -> T {
    (dx.powi(2) + dy.powi(2)).sqrt()
}

pub fn bearing<T: Float>(dx: T, dy: T) -> T {
    (-dx).atan2(-dy).to_degrees()
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EPoint<T> {
    /// X-axis (East/West) component of the flat-surface point in meters
    pub x: T,
    /// Y-axis (North/South) component of the flat-surface point in meters
    pub y: T,
}

impl<T: Float> EPoint<T> {
    fn delta(&self, other: &EPoint<T>) -> (T, T) {
        (self.x - other.x, self.y - other.y)
    }

    /// Distance between this point and the other
    pub fn distance(&self, other: &EPoint<T>) -> T {
        let (dx, dy) = self.delta(other);
        delta_distance(dx, dy)
    }

    /// Bearing from this point to the other
    pub fn bearing(&self, other: &EPoint<T>) -> T {
        let (dx, dy) = self.delta(other);
        bearing(dx, dy)
    }

    /// New CheapPoint given a distance a bearing from this point
    pub fn destination(&self, dist: T, bearing: T) -> EPoint<T> {
        let a = bearing.to_radians();
        self.offset(a.sin() * dist, a.cos() * dist)
    }

    /// Returns a new `CheapPoint` given easting and northing offsets
    pub fn offset(&self, dx: T, dy: T) -> EPoint<T> {
        EPoint {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EBox<T> {
    pub top: T,
    pub bottom: T,
    pub right: T,
    pub left: T,
}

impl<T: Float> EBox<T> {
    /// Create the bounding Box from a vector of points
    pub fn from_slice(points: &[EPoint<T>]) -> EBox<T> {
        let top = points.iter().fold(T::neg_infinity(), |m, p| T::max(m, p.y));
        let bottom = points.iter().fold(T::infinity(), |m, p| T::min(m, p.y));
        let right = points.iter().fold(T::neg_infinity(), |m, p| T::max(m, p.x));
        let left = points.iter().fold(T::infinity(), |m, p| T::min(m, p.x));
        EBox {
            top,
            bottom,
            right,
            left,
        }
    }

    /// Get the intersection point of the diagonals
    pub fn center(&self) -> EPoint<T> {
        let two = T::from(2).unwrap();
        EPoint {
            x: (self.right + self.left) / two,
            y: (self.top + self.bottom) / two,
        }
    }

    /// Diagonal length of the Box
    pub fn diagonal(&self) -> T {
        ((self.right - self.left).powi(2) + (self.top - self.bottom).powi(2)).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_box() -> EBox<f64> {
        let v = vec![
            EPoint { x: 1.0, y: 1.0 },
            EPoint { x: 1.0, y: 3.0 },
            EPoint { x: 1.4, y: 2.7 },
            EPoint { x: 5.0, y: 1.0 },
            EPoint { x: 5.0, y: 3.0 },
        ];

        EBox::from_slice(v.as_slice())
    }

    #[test]
    fn slice_creation() {
        let b = test_box();

        assert_eq!(b.bottom, 1.0);
        assert_eq!(b.top, 3.0);
        assert_eq!(b.left, 1.0);
        assert_eq!(b.right, 5.0);
    }

    #[test]
    fn box_center() {
        let b = test_box();
        let center = b.center();

        assert_eq!(center.x, 3.0);
        assert_eq!(center.y, 2.0);
    }

    #[test]
    fn box_diagonal() {
        let b = test_box();

        assert!((b.diagonal() - 4.472136).abs() < 0.00001);
    }
}
