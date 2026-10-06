// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Flight sections of a track, detected from ground speed.

use log::{debug, warn};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::Fix;
use crate::utils::geometry::{Flat, PointDistance};
use crate::utils::projector::CheapProjection;

/// Ground speed, in m/s, at or above which a fix counts as flying: ~15 km/h.
const FGSLIM: f64 = 4.2;

/// Milliseconds a track must hold above `FGSLIM` to count as flying.
const TTHRES: f64 = 30_000.0;

/// Longest time gap, in milliseconds, a flight may contain; a longer one splits the track.
/// FIXME: different rulesets, different rules
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
    /// Flight sections of this track, at most one per stretch free of a gap longer than five
    /// minutes. Empty when nothing was detected.
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

/// Flight sections of `track`, at most one per stretch free of a gap longer than `max_gap`.
fn detect_flights(track: &[Fix], max_gap: f64) -> Vec<Flight> {
    if track.is_empty() {
        return Vec::new();
    }

    // The projection's scale depends on latitude only, and only differences of projected points
    // are read, so the origin's longitude cancels out and the antimeridian needs no unwrapping.
    let (south, north) = track.iter().fold((f64::MAX, f64::MIN), |(s, n), fix| {
        (s.min(fix.lat), n.max(fix.lat))
    });
    let projection = CheapProjection::new(&[(south + north) / 2.0, track[0].lon]);
    let mut bad: usize = 0;

    let flatp: Vec<[f64; 2]> = track
        .iter()
        .map(|fix| {
            let point: [f64; 2] = projection.project(fix);
            point
        })
        .collect();

    // Ground speed in m/s; `t` and `dt` are milliseconds.
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

    // Each gap-free stretch is detected on its own, so no speed or smoothing spans a gap. The last
    // cut closes the final stretch.
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

/// The flight in one gap-free stretch: from the first run holding at least `FGSLIM` for `TTHRES`,
/// to the first slow sample after the last fast one, or the last sample. Empty when no run holds
/// that long.
// FIXME: crude ground-speed detection, to replace with a real detector.
fn detect(gs: &[f64], dt: &[f64]) -> Vec<[usize; 2]> {
    let mut result = Vec::new();

    // Ground speed averaged over 5 intervals.
    let sgs: Vec<f64> = gs.windows(5).map(|w| w.iter().sum::<f64>() / 5.0).collect();

    let mut held = 0.0;
    let mut start = 0;

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
