// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Forked from https://github.com/Turbo87/flat-projection-rs.git

use num_traits::Float;

use crate::utils::geometry::{PointCoords, PointNew};

/// Wraps a longitude *difference* into [-180, 180]. Callers pass the difference of two longitudes,
/// so the input is in [-360, 360]; a larger one is wrapped only once and comes out wrong.
/// That limitation is what keeps a division off the hot path.
pub fn lon_round<T: Float>(lon: T) -> T {
    let full = T::from(360).unwrap();
    let half = T::from(180).unwrap();

    if lon >= half {
        lon - full
    } else if lon <= -half {
        lon + full
    } else {
        lon
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct CheapProjection<T: Float> {
    ky: T,
    kx: T,

    lat: T,
    lon: T,
}

impl<T: Float> CheapProjection<T> {
    pub fn new<P>(point: &P) -> CheapProjection<T>
    where
        P: PointCoords<T>,
    {
        // see https://github.com/mapbox/cheap-ruler/

        let one = T::one();
        let two = T::from(2).unwrap();

        // Values that define WGS84 ellipsoid model of the Earth
        let re: T = T::from(6378137).unwrap(); // equatorial radius in m
        let fe: T = one / T::from(298.257223563).unwrap(); // flattening
        let e2: T = fe * (two - fe);

        // Curvature formulas from https://en.wikipedia.org/wiki/Earth_radius#Meridional
        let cos_lat = point.y().to_radians().cos();
        let w2 = one / (one - e2 * (one - cos_lat * cos_lat));
        let w = w2.sqrt();

        // multipliers for converting longitude and latitude degrees into distance
        let kx = (re * w * cos_lat).to_radians(); // based on normal radius of curvature
        let ky = (re * w * w2 * (one - e2)).to_radians(); // based on meridional radius of curvature

        CheapProjection {
            kx,
            ky,
            lat: point.y(),
            lon: point.x(),
        }
    }

    /// Projects a (lon, lat) point to metres on the plane centred on this projection's origin.
    pub fn project<P, R>(&self, input: &P) -> R
    where
        P: PointCoords<T>,
        R: PointNew<T>,
    {
        R::new(
            lon_round(input.x() - self.lon) * self.kx,
            (input.y() - self.lat) * self.ky,
        )
    }

    /// Inverse of [`CheapProjection::project`], back to (lon, lat).
    #[allow(dead_code)]
    pub fn unproject<P, R>(&self, input: &P) -> R
    where
        P: PointCoords<T>,
        R: PointNew<T>,
    {
        R::new(
            lon_round(input.x() / self.kx + self.lon),
            input.y() / self.ky + self.lat,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The rounding form this replaced, kept as the oracle: the two must agree over every input a
    // difference of two parsed longitudes can take.
    #[test]
    fn lon_round_matches_the_rounding_form() {
        let rounding = |l: f64| l - (l / 360.0).round() * 360.0;

        for i in -360_000..=360_000 {
            let l = i as f64 / 1000.0;
            assert_eq!(lon_round(l), rounding(l), "at {l}");
        }

        // Ties, where the two forms are easiest to get wrong and the sweep's step may not land
        for l in [-360., -180., -0., 0., 180., 360.] {
            assert_eq!(lon_round(l), rounding(l), "at {l}");
        }
    }

    #[test]
    fn flat_projection_on_ref() {
        let refp = [50., 31.];
        let proj = CheapProjection::new(&refp);
        let out: [f64; 2] = proj.project(&refp);

        assert_eq!(out.x(), 0.);
        assert_eq!(out.y(), 0.);
    }

    #[test]
    fn flat_projection() {
        let input = [50.5, 30.8];
        let proj = CheapProjection::new(&[50., 31.]);
        let out: [f64; 2] = proj.project(&input);

        assert!((out.x() - -14339.15072).abs() < 0.00001);
        assert!((out.y() - 55614.53199).abs() < 0.00001);
    }

    #[test]
    fn flat_unprojection() {
        let input = [-30000., 10000.];
        let proj = CheapProjection::new(&[50., 31.]);
        let out: [f64; 2] = proj.unproject(&input);

        assert!((out.y() - 49.730286).abs() < 0.00001);
        assert!((out.x() - 31.139478).abs() < 0.00001);
    }
}
