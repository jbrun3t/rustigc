// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! FCC-vs-geodesic slack the search leaves on every distance it scores, so the floor never rises
//! above what `report` credits geodesically. The error depends on distance and latitude, both known
//! during the search.

/// Latitude band of a flight: row of [`TABLE`] covering every leg it can contain.
#[derive(Debug, Clone, Copy)]
pub struct Band(usize);

/// Upper edge of each column, in meters.
const KEYS_M: [f64; 12] = [
    50e3, 75e3, 100e3, 150e3, 250e3, 400e3, 600e3, 800e3, 1200e3, 1600e3, 2000e3, 3200e3,
];

/// Worst-case relative error `|fcc - geo| / geo`. Row `i` covers legs with both ends within
/// `10 * (i + 2)` degrees of the equator, column `k` distances up to `KEYS_M[k]`.
///
/// Derived offline against `geographiclib_rs` WGS84: per cell, the worst over every whole degree of
/// latitude up to the row's ceiling and every whole degree of azimuth, dropping legs that land past
/// the ceiling; then a running maximum along the row, ×1.02, rounded up to two digits. A change to
/// `Fcc::distance` or `cos_lat` invalidates them.
#[rustfmt::skip]
const TABLE: [[f64; KEYS_M.len()]; 7] = [
    [6.5e-5, 6.5e-5, 6.6e-5, 6.6e-5, 6.7e-5, 6.9e-5, 1.3e-4, 2.3e-4, 5.0e-4, 8.6e-4, 1.3e-3, 3.1e-3], // <= 20 deg
    [6.5e-5, 6.5e-5, 6.6e-5, 6.6e-5, 6.7e-5, 9.6e-5, 2.1e-4, 3.6e-4, 7.7e-4, 1.3e-3, 2.0e-3, 4.4e-3], // <= 30 deg
    [6.5e-5, 6.5e-5, 6.6e-5, 6.6e-5, 7.8e-5, 1.8e-4, 3.6e-4, 6.2e-4, 1.3e-3, 2.2e-3, 3.3e-3, 7.9e-3], // <= 40 deg
    [6.5e-5, 6.5e-5, 6.6e-5, 6.8e-5, 1.4e-4, 3.1e-4, 6.5e-4, 1.1e-3, 2.4e-3, 3.9e-3, 6.1e-3, 1.7e-2], // <= 50 deg
    [6.5e-5, 6.5e-5, 7.3e-5, 1.2e-4, 2.6e-4, 5.9e-4, 1.3e-3, 2.2e-3, 4.6e-3, 8.3e-3, 1.3e-2, 3.6e-2], // <= 60 deg
    [6.6e-5, 9.3e-5, 1.4e-4, 2.4e-4, 5.8e-4, 1.4e-3, 3.0e-3, 5.2e-3, 1.2e-2, 2.2e-2, 3.5e-2, 1.1e-1], // <= 70 deg
    [1.4e-4, 2.6e-4, 4.1e-4, 8.5e-4, 2.2e-3, 5.5e-3, 1.3e-2, 2.3e-2, 5.6e-2, 1.2e-1, 2.4e-1, 6.0e-1], // <= 80 deg
];

impl Band {
    /// Band of a track reaching `max_abs_lat` degrees from the equator, clamped to the last row.
    pub fn of(max_abs_lat: f64) -> Self {
        Self(
            ((max_abs_lat / 10.0) as usize)
                .saturating_sub(1)
                .min(TABLE.len() - 1),
        )
    }

    /// Slack to allow on a distance of `d_m` metres. Distances past the last key clamp to it.
    #[inline]
    pub fn margin(&self, d_m: f64) -> f64 {
        let k = KEYS_M.partition_point(|&key| key < d_m);
        TABLE[self.0][k.min(KEYS_M.len() - 1)]
    }
}
