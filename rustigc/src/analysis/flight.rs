// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Flight sections of a track.
//!
//! Detection is a crude ground-speed placeholder (see FIXME), meant to be replaced without touching
//! anything else.

use log::{debug, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::utils::geometry::{Flat, PointDistance};
use crate::utils::projector::CheapProjection;
use crate::Fix;

/// Minimal flight speed
const FGSLIM: f64 = 4.2; // ~15kph

/// Milliseconds a track must hold above `FGSLIM` to count as flying.
const TTHRES: f64 = 30_000.0;

/// Longest gap interval, in milliseconds, a flight may contain.
/// FIXME: different leagues, different rules
const MAX_GAP: f64 = 300_000.0;

/// One flight section, as fix indices into the track it was detected in.
///
/// The window [`Log::score`] takes. With the `geojson` feature it is also a layer `Log::export`
/// can draw.
///
/// [`Log::score`]: crate::Log::score
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Flight {
    /// Takeoff, as an index into the track.
    pub start: usize,
    /// Landing, as an index into the track.
    pub stop: usize,
}

/// Flight detection over a track, implemented for `[Fix]`.
pub trait FlightDetection {
    /// Flight sections of this track, one per stretch free of a gap longer than five minutes.
    /// Empty when nothing was detected.
    ///
    /// ```no_run
    /// use rustigc::{FlightDetection, Log};
    ///
    /// let log = Log::new(&std::fs::read("flight.igc")?)?;
    /// for flight in log.track.flights() {
    ///     println!("{}..{}", flight.start, flight.stop);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn flights(&self) -> Vec<Flight>;
}

impl FlightDetection for [Fix] {
    fn flights(&self) -> Vec<Flight> {
        detect_flights(self, MAX_GAP)
    }
}

/// Picking one section out of a detection result, implemented for `[Flight]`.
pub trait FlightSelection {
    /// Longest section by fix span, `None` when there is none.
    fn longest(&self) -> Option<&Flight>;
}

impl FlightSelection for [Flight] {
    fn longest(&self) -> Option<&Flight> {
        self.iter().max_by_key(|f| f.stop - f.start)
    }
}

/// Detect the flight sections of a track, one per stretch free of a gap longer than `max_gap`.
/// Empty when no flight is detected.
fn detect_flights(track: &[Fix], max_gap: f64) -> Vec<Flight> {
    if track.is_empty() {
        return Vec::new();
    }

    // Avoid using a BBox here so we don't to unwrap the coordinates first.
    // The projections only care for the latitude. longitude is just an offset carried around
    // This saves a bit of runtime while handling the antimeridian correctly
    let (south, north) = track.iter().fold((f64::MAX, f64::MIN), |(s, n), fix| {
        (s.min(fix.lat), n.max(fix.lat))
    });
    let projection = CheapProjection::new(&[(south + north) / 2.0, track[0].lon]);
    let mut bad: usize = 0;

    // Project the track on a flat surface
    let flatp: Vec<[f64; 2]> = track
        .iter()
        .map(|fix| {
            let point: [f64; 2] = projection.project(fix);
            point
        })
        .collect();

    // Get the ground speed. `t`/`dt` are milliseconds; `FGSLIM` is m/s, so the speed calc
    // converts `duration` to seconds at that one point.
    let t: Vec<f64> = track.iter().map(|fix| fix.timestamp as f64).collect();
    let d: Vec<f64> = flatp
        .windows(2)
        .map(|w| Flat::distance(&w[0], &w[1]))
        .collect();
    let dt: Vec<f64> = t.windows(2).map(|w| w[1] - w[0]).collect();

    let gs: Vec<f64> = d
        .iter()
        .zip(dt.iter())
        .map(|(dist, duration)| {
            if *duration > 0.0 {
                dist / (duration / 1000.0)
            } else {
                bad += 1;
                0.0
            }
        })
        .collect();

    if bad > 0 {
        warn!("{bad} fix interval(s) carry no time, ground speed read as 0");
    }

    // Cut at every gap and detect each stretch on its own: a speed spanning a gap is meaningless, and
    // `detect` must not smooth across one either. `cuts` ends past the last interval to close the
    // final stretch.
    let mut cuts: Vec<usize> = dt
        .iter()
        .enumerate()
        .filter(|(_, gap)| **gap > max_gap)
        .map(|(i, _)| i)
        .collect();
    cuts.push(gs.len());

    let mut flights = Vec::new();
    let mut first = 0;
    for cut in cuts {
        // `detect` works in the slice's own index space, hence the shift back
        flights.extend(
            detect(&gs[first..cut], &dt[first..cut])
                .iter()
                .map(|f| Flight {
                    start: f[0] + first,
                    stop: f[1] + first,
                }),
        );
        first = cut + 1;
    }

    debug!("Detected flights: {flights:?}");

    flights
}

// FIXME: This detection algorithm clearly sucks and is meant to be replaced
fn detect(gs: &[f64], dt: &[f64]) -> Vec<[usize; 2]> {
    let mut result = Vec::new();

    // We do not yet have proper track smoothing, just average the speed to start with.
    let sgs: Vec<f64> = gs.windows(5).map(|w| w.iter().sum::<f64>() / 5.0).collect();

    let mut held = 0.0;
    let mut start = 0;

    // Look for the start of the flight: the first stretch holding above the threshold for `TTHRES`
    // milliseconds. Yes it sucks, but it is not here to stay.
    for (i, gs) in sgs.iter().enumerate() {
        if *gs >= FGSLIM {
            if held == 0.0 {
                start = i;
            }
            held += dt[i];
            if held >= TTHRES {
                break;
            }
        } else {
            held = 0.0;
        }
    }

    if held < TTHRES {
        return result;
    }

    let mut count = 0;
    let mut stop = start;

    for (i, gs) in sgs.iter().enumerate().skip(start) {
        if *gs < FGSLIM {
            if count == 0 {
                stop = i;
            }
            count += 1;
        } else {
            count = 0;
            stop = i;
        }
    }

    result.push([start, stop]);
    result
}
