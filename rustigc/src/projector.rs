//! Forked from https://github.com/Turbo87/flat-projection-rs.git

// #![allow(clippy::derive_partial_eq_without_eq)]

use num_traits::Float;

pub fn distance_squared<T: Float>(dx: T, dy: T) -> T {
    dx.powi(2) + dy.powi(2)
}

pub fn bearing<T: Float>(dx: T, dy: T) -> T {
    (-dx).atan2(-dy).to_degrees()
}

pub fn lon_round<T: Float>(lon: T) -> T {
    let o = T::from(360).unwrap();
    lon - ((lon / o).round() * o)
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct CheapProjection<T: Float> {
    kx: T,
    ky: T,

    lat: T,
    lon: T,
}

impl<T: Float> CheapProjection<T> {
    pub fn new(lon: T, lat: T) -> CheapProjection<T> {
        // see https://github.com/mapbox/cheap-ruler/

        let one = T::one();
        let two = T::from(2).unwrap();

        // Values that define WGS84 ellipsoid model of the Earth
        let re: T = T::from(6378137).unwrap(); // equatorial radius in m
        let fe: T = one / T::from(298.257223563).unwrap(); // flattening
        let e2: T = fe * (two - fe);

        // Curvature formulas from https://en.wikipedia.org/wiki/Earth_radius#Meridional
        let cos_lat = lat.to_radians().cos();
        let w2 = one / (one - e2 * (one - cos_lat * cos_lat));
        let w = w2.sqrt();

        // multipliers for converting longitude and latitude degrees into distance
        let kx = (re * w * cos_lat).to_radians(); // based on normal radius of curvature
        let ky = (re * w * w2 * (one - e2)).to_radians(); // based on meridional radius of curvature

        CheapProjection { kx, ky, lat, lon }
    }

    // /// Converts a longitude and latitude (in degrees) to a [`FlatPoint`]
    // /// instance that can be used for fast geodesic approximations.
    // ///
    // /// [`FlatPoint`]: struct.FlatPoint.html
    // ///
    // /// ```
    // /// # use xyz_crate::CheapProjection;
    // /// #
    // /// let (lon, lat) = (6.186389, 50.823194);
    // ///
    // /// let proj = CheapProjection::new(6., 51.);
    // ///
    // /// let flat_point = proj.project(lon, lat);
    // /// ```
    // pub fn project(&self, longitude: T, latitude: T) -> FlatPoint<T> {
    //     let x = (longitude - self.lon) * self.kx;
    //     let y = (latitude - self.lat) * self.ky;

    //     FlatPoint { x, y }
    // }

    pub fn from_lat(&self, lat: T) -> T {
        (lat - self.lat) * self.ky
    }

    pub fn from_lon(&self, lon: T) -> T {
        lon_round(lon - self.lon) * self.kx
    }

    // /// Converts a [`FlatPoint`] back to a (lon, lat) tuple.
    // ///
    // /// [`FlatPoint`]: struct.FlatPoint.html
    // ///
    // /// ```
    // /// # use xyz_crate::CheapProjection;
    // /// #
    // /// let (lon, lat) = (6.186389, 50.823194);
    // ///
    // /// let proj = CheapProjection::new(6., 51.);
    // ///
    // /// let flat_point = proj.project(lon, lat);
    // ///
    // /// let result = proj.unproject(&flat_point);
    // ///
    // /// assert_eq!(result.0, lon);
    // ///
    // /// assert_eq!(result.1, lat);
    // /// ```
    // pub fn unproject(&self, p: &FlatPoint<T>) -> (T, T) {
    //     (p.x / self.kx + self.lon, p.y / self.ky + self.lat)
    // }

    pub fn to_lat(&self, y: T) -> T {
        y / self.ky + self.lat
    }

    pub fn to_lon(&self, x: T) -> T {
        lon_round(x / self.kx + self.lon)
    }
}

// /// Representation of a geographical point on Earth as projected
// /// by a [`CheapProjection`] instance.
// ///
// /// [`CheapProjection`]: struct.CheapProjection.html
// ///
// /// ```
// /// # #[macro_use]
// /// # extern crate assert_approx_eq;
// /// #
// /// # use xyz_crate::CheapProjection;
// /// #
// /// # fn main() {
// /// let (lon, lat) = (6.186389, 50.823194);
// ///
// /// let proj = CheapProjection::new(6., 51.);
// ///
// /// let flat_point = proj.project(lon, lat);
// /// #
// /// # assert_approx_eq!(flat_point.x, 13.0845f64, 0.001);
// /// # assert_approx_eq!(flat_point.y, -19.6694f64, 0.001);
// /// # }
// /// ```
// #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
// pub struct FlatPoint<T> {
//     /// X-axis component of the flat-surface point in kilometers
//     pub x: T,
//     /// Y-axis component of the flat-surface point in kilometers
//     pub y: T,
// }

// impl<T: Float> FlatPoint<T> {
//     /// Calculates the approximate distance in kilometers from
//     /// this `FlatPoint` to another.
//     ///
//     /// ```
//     /// # #[macro_use]
//     /// # extern crate assert_approx_eq;
//     /// # extern crate num_traits;
//     /// # extern crate xyz_crate;
//     /// #
//     /// # use num_traits::float::Float;
//     /// # use xyz_crate::CheapProjection;
//     /// #
//     /// # fn main() {
//     /// let (lon1, lat1) = (6.186389, 50.823194);
//     /// let (lon2, lat2) = (6.953333, 51.301389);
//     ///
//     /// let proj = CheapProjection::new(6.5, 51.05);
//     ///
//     /// let p1 = proj.project(lon1, lat1);
//     /// let p2 = proj.project(lon2, lat2);
//     ///
//     /// let distance = p1.distance(&p2);
//     /// // -> 75.648 km
//     /// #
//     /// # assert_approx_eq!(distance, 75.635_595, 0.02);
//     /// # }
//     /// ```
//     pub fn distance(&self, other: &FlatPoint<T>) -> T {
//         self.distance_squared(other).sqrt()
//     }

//     /// Calculates the approximate squared distance from this `FlatPoint` to
//     /// another.
//     ///
//     /// This method can be used for fast distance comparisons.
//     pub fn distance_squared(&self, other: &FlatPoint<T>) -> T {
//         let (dx, dy) = self.delta(other);
//         distance_squared(dx, dy)
//     }

//     /// Calculates the approximate average bearing in degrees
//     /// between -180 and 180 from this `FlatPoint` to another.
//     ///
//     /// ```
//     /// # #[macro_use]
//     /// # extern crate assert_approx_eq;
//     /// # extern crate num_traits;
//     /// # extern crate xyz_crate;
//     /// #
//     /// # use num_traits::float::Float;
//     /// # use xyz_crate::CheapProjection;
//     /// #
//     /// # fn main() {
//     /// let (lon1, lat1) = (6.186389, 50.823194);
//     /// let (lon2, lat2) = (6.953333, 51.301389);
//     ///
//     /// let proj = CheapProjection::new(6.5, 51.05);
//     ///
//     /// let p1 = proj.project(lon1, lat1);
//     /// let p2 = proj.project(lon2, lat2);
//     ///
//     /// let bearing = p1.bearing(&p2);
//     /// // -> 45.3°
//     /// #
//     /// # assert_approx_eq!(bearing, 45.312, 0.001);
//     /// # }
//     /// ```
//     pub fn bearing(&self, other: &FlatPoint<T>) -> T {
//         let (dx, dy) = self.delta(other);
//         bearing(dx, dy)
//     }

//     /// Calculates the approximate [`distance`] and average [`bearing`]
//     /// from this `FlatPoint` to another.
//     ///
//     /// [`distance`]: #method.distance
//     /// [`bearing`]: #method.bearing
//     ///
//     /// ```
//     /// # #[macro_use]
//     /// # extern crate assert_approx_eq;
//     /// # extern crate num_traits;
//     /// # extern crate xyz_crate;
//     /// #
//     /// # use num_traits::float::Float;
//     /// # use xyz_crate::CheapProjection;
//     /// #
//     /// # fn main() {
//     /// let (lon1, lat1) = (6.186389, 50.823194);
//     /// let (lon2, lat2) = (6.953333, 51.301389);
//     ///
//     /// let proj = CheapProjection::new(6.5, 51.05);
//     ///
//     /// let p1 = proj.project(lon1, lat1);
//     /// let p2 = proj.project(lon2, lat2);
//     ///
//     /// let (distance, bearing) = p1.distance_bearing(&p2);
//     /// // -> 75.648 km and 45.3°
//     /// #
//     /// # assert_approx_eq!(distance, 75.635_595, 0.02);
//     /// # assert_approx_eq!(bearing, 45.312, 0.001);
//     /// # }
//     /// ```
//     pub fn distance_bearing(&self, other: &FlatPoint<T>) -> (T, T) {
//         let (dx, dy) = self.delta(other);
//         (distance_squared(dx, dy).sqrt(), bearing(dx, dy))
//     }

//     fn delta(&self, other: &FlatPoint<T>) -> (T, T) {
//         (self.x - other.x, self.y - other.y)
//     }

//     /// Returns a new `FlatPoint` given [`distance`] and [`bearing`] from this `FlatPoint`.
//     ///
//     /// [`distance`]: #method.distance (kilometers)
//     /// [`bearing`]: #method.bearing (degrees)
//     ///
//     /// ```
//     /// # #[macro_use]
//     /// # extern crate assert_approx_eq;
//     /// # extern crate num_traits;
//     /// # extern crate xyz_crate;
//     /// #
//     /// # use num_traits::float::Float;
//     /// # use xyz_crate::CheapProjection;
//     /// #
//     /// # fn main() {
//     /// let (lon, lat) = (30.5, 50.5);
//     ///
//     /// let proj = CheapProjection::new(31., 50.);
//     ///
//     /// let p1 = proj.project(lon, lat);
//     /// let (distance, bearing) = (1., 45.0);
//     /// let p2 = p1.destination(distance, bearing);
//     /// #
//     /// # let res_distance = p1.distance(&p2);
//     /// # let (dest_lon, dest_lat) = proj.unproject(&p2);
//     /// #
//     /// # assert_approx_eq!(dest_lon, 30.5098622, 0.00001);
//     /// # assert_approx_eq!(dest_lat, 50.5063572, 0.00001);
//     /// # }
//     /// ```
//     pub fn destination(&self, dist: T, bearing: T) -> FlatPoint<T> {
//         let a = bearing.to_radians();
//         self.offset(a.sin() * dist, a.cos() * dist)
//     }

//     /// Returns a new `FlatPoint` given easting and northing offsets
//     /// (in kilometers) from this `FlatPoint`.
//     ///
//     /// ```
//     /// # #[macro_use]
//     /// # extern crate assert_approx_eq;
//     /// # extern crate num_traits;
//     /// # extern crate xyz_crate;
//     /// #
//     /// # use num_traits::float::Float;
//     /// # use xyz_crate::CheapProjection;
//     /// #
//     /// # fn main() {
//     /// let (lon, lat) = (30.5, 50.5);
//     ///
//     /// let proj = CheapProjection::new(31., 50.);
//     ///
//     /// let p1 = proj.project(lon, lat);
//     /// let p2 = p1.offset(10., 10.);
//     /// #
//     /// # let (dest_lon, dest_lat) = proj.unproject(&p2);
//     /// # assert_approx_eq!(dest_lon, 30.6394736, 0.00001);
//     /// # assert_approx_eq!(dest_lat, 50.5899044, 0.00001);
//     /// # }
//     /// ```
//     pub fn offset(&self, dx: T, dy: T) -> FlatPoint<T> {
//         FlatPoint {
//             x: self.x + dx,
//             y: self.y + dy,
//         }
//     }
// }

// #[cfg(test)]
// #[macro_use]
// extern crate assert_approx_eq;

// #[cfg(test)]
// mod tests {
//     use num_traits::Float;
//     use CheapProjection;

//     #[test]
//     fn flatpoint_destination_ne() {
//         let (lon, lat) = (30.5, 50.5);
//         let proj = CheapProjection::new(31., 50.);
//         let p1 = proj.project(lon, lat);

//         let (distance, bearing) = (1., 45.0);
//         let p2 = p1.destination(distance, bearing);
//         let res_distance = p1.distance(&p2);
//         let (dest_lon, dest_lat) = proj.unproject(&p2);
//         assert_approx_eq!(dest_lon, 30.5098622, 0.00001);
//         assert_approx_eq!(dest_lat, 50.5063572, 0.00001);
//         assert_approx_eq!(distance, res_distance, 0.00001);
//     }

//     #[test]
//     fn flatpoint_destination_se() {
//         let (lon, lat) = (30.5, 50.5);
//         let proj = CheapProjection::new(31., 50.);
//         let p1 = proj.project(lon, lat);

//         let (distance, bearing) = (1., 135.0);

//         let p2 = p1.destination(distance, bearing);
//         let res_distance = p1.distance(&p2);
//         let (dest_lon, dest_lat) = proj.unproject(&p2);
//         assert_approx_eq!(dest_lon, 30.5098622, 0.00001);
//         assert_approx_eq!(dest_lat, 50.4936427, 0.00001);
//         assert_approx_eq!(distance, res_distance, 0.00001);
//     }

//     #[test]
//     fn flatpoint_destination_sw() {
//         let (lon, lat) = (30.5, 50.5);
//         let proj = CheapProjection::new(31., 50.);
//         let p1 = proj.project(lon, lat);

//         let (distance, bearing) = (1., 225.0);
//         let p2 = p1.destination(distance, bearing);
//         let res_distance = p1.distance(&p2);
//         let (dest_lon, dest_lat) = proj.unproject(&p2);
//         assert_approx_eq!(dest_lon, 30.4901377, 0.00001);
//         assert_approx_eq!(dest_lat, 50.4936427, 0.00001);
//         assert_approx_eq!(distance, res_distance, 0.00001);
//     }

//     #[test]
//     fn flatpoint_destination_nw() {
//         let (lon, lat) = (30.5, 50.5);
//         let proj = CheapProjection::new(31., 50.);
//         let p1 = proj.project(lon, lat);

//         let (distance, bearing) = (1., 315.0);
//         let p2 = p1.destination(distance, bearing);
//         let res_distance = p1.distance(&p2);
//         let (dest_lon, dest_lat) = proj.unproject(&p2);
//         assert_approx_eq!(dest_lon, 30.4901377, 0.00001);
//         assert_approx_eq!(dest_lat, 50.5063572, 0.00001);
//         assert_approx_eq!(distance, res_distance, 0.00001);
//     }
// }
