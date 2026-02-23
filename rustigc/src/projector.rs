//! Forked from https://github.com/Turbo87/flat-projection-rs.git

use num_traits::Float;

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
    pub fn project(&self, lat: T, lon: T) -> (T, T) {
        (
            lon_round(lon - self.lon) * self.kx,
            (lat - self.lat) * self.ky,
        )
    }

    /// Converts a [`CheapPoint`] back to a (lon, lat) tuple.
    pub fn unproject(&self, x: T, y: T) -> (T, T) {
        (y / self.ky + self.lat, lon_round(x / self.kx + self.lon))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_projection_on_ref() {
        let (lat, lon) = (50., 31.);
        let proj = CheapProjection::new(50., 31.);
        let (x, y) = proj.project(lat, lon);

        assert_eq!(x, 0.);
        assert_eq!(y, 0.);
    }

    #[test]
    fn flat_projection() {
        let (lat, lon) = (50.5, 30.8);
        let proj = CheapProjection::new(50., 31.);
        let (x, y) = proj.project(lat, lon);

        assert!((x - -14339.15072).abs() < 0.00001);
        assert!((y - 55614.53199).abs() < 0.00001);
    }

    #[test]
    fn flat_unprojection() {
        let (x, y) = (10000., -30000.);
        let proj = CheapProjection::new(50., 31.);
        let (lat, lon) = proj.unproject(x, y);

        assert!((lat - 49.730286).abs() < 0.00001);
        assert!((lon - 31.139478).abs() < 0.00001);
    }
}
