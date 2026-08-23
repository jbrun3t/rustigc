// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

#[cfg(feature = "geojson")]
pub mod export;
pub mod geometry;
pub mod iter;
pub mod projector;

pub fn round_km(meters: f64) -> f64 {
    (meters / 10.0).round() / 100.0
}
