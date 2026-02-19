//! Tracklog analysis

use crate::geometry::EPoint;
use crate::projector::CheapProjection;
use crate::Log;

#[derive(Debug, Clone)]
pub struct FRawData {
    pub projection: CheapProjection<f64>,
    /// timestamp
    pub t: Vec<f64>,
    /// Projected Coordinates
    pub p: Vec<EPoint<f64>>,
    /// Distance
    pub d: Vec<f64>,
    /// Ground Speed
    pub gs: Vec<f64>,
}

const FGSLIM: f64 = 4.2; // ~15kph
const TTHRES: usize = 30;

impl FRawData {
    pub fn new(log: &Log) -> Self {
        // Create Project from track mean point
        let (lat0, lon0) = log.center();
        let projection = CheapProjection::new(lat0, lon0);

        // Get timestamps
        let t: Vec<f64> = log.track.iter().map(|fix| fix.timestamp as f64).collect();

        // Project coordinates
        let p: Vec<EPoint<f64>> = log
            .track
            .iter()
            .map(|fix| projection.project(fix.lat, fix.lon))
            .collect();

        // Distances and Speeds
        let d: Vec<f64> = p.windows(2).map(|w| w[1].distance(&w[0])).collect();
        let dt: Vec<f64> = t.windows(2).map(|w| w[1] - w[0]).collect();
        let gs: Vec<f64> = d
            .iter()
            .zip(dt.iter())
            .map(|(dist, duration)| dist / duration)
            .collect();

        Self {
            projection,
            t,
            p,
            d,
            gs,
        }
    }

    pub fn phases(&self) -> Option<(usize, usize)> {
        // We do not yet have proper track smoothing, just average the speed to start wiht
        let sgs: Vec<f64> = self
            .gs
            .windows(5)
            .map(|w| w.iter().sum::<f64>() / 5.0)
            .collect();

        let mut count = 0;
        let mut start = 0;

        // Look for the start of the flight. Do not even bother with timestamps, just look for
        // n consecutive sample above the threshold. Yes it sucks, but it is not here to stay.
        for (i, gs) in sgs.iter().enumerate() {
            if *gs >= FGSLIM {
                if count == 0 {
                    start = i;
                }
                count += 1;
                if count >= TTHRES {
                    break;
                }
            } else {
                count = 0;
            }
        }

        if count < TTHRES {
            return None;
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

        Some((start, stop))
    }
}
