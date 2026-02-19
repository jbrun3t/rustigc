//! Forked from https://github.com/Turbo87/flat-projection-rs.git

use num_traits::Float;
use crate::geometry::EPoint;

pub fn lon_round<T: Float>(lon: T) -> T {
    let o = T::from(360).unwrap();
    lon - ((lon / o).round() * o)
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct CheapProjection<T: Float> {
    ky: T,
    kx: T,

    lat: T,
    lon: T,
}

impl<T: Float> CheapProjection<T> {
    pub fn new(lat: T, lon: T) -> CheapProjection<T> {
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

    /// Converts a (lon, lat) tuple to a [`CheapPoint`] projection
    pub fn project(&self, lat: T, lon: T) -> EPoint<T> {
        let y = (lat - self.lat) * self.ky;
        let x = lon_round(lon - self.lon) * self.kx;

        EPoint { x, y }
    }

    /// Converts a [`CheapPoint`] back to a (lon, lat) tuple.
    pub fn unproject(&self, p: &EPoint<T>) -> (T, T) {
        (
            p.y / self.ky + self.lat,
            lon_round(p.x / self.kx + self.lon),
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatpoint_destination() {
        let (lat, lon) = (50.5, 30.5);
        let proj = CheapProjection::new(50., 31.);
        let p1 = proj.project(lat, lon);
        let (distance, bearing) = (1000., 45.0);
        let p2 = p1.destination(distance, bearing);
        let res_distance = p1.distance(&p2);
        let (dest_lat, dest_lon) = proj.unproject(&p2);

        assert!((dest_lat - 50.5063572).abs() < 0.00001);
        assert!((dest_lon - 30.5098622).abs() < 0.00001);
        assert!((distance - res_distance).abs() < 0.00001);
    }
}
