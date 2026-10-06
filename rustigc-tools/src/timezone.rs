// SPDX-License-Identifier: GPL-2.0-or-later

//! Local wall-clock time for a track.
//!
//! `utz` names the zone of the first fix from boundary polygons, and `jiff` gives that zone's
//! offsets from the tzdb. The log's `TZN` offset is the fallback, then UTC.

use jiff::tz::{Offset, TimeZone};
use jiff::{SignedDuration, Timestamp};
use log::warn;

use rustigc::{Fix, Log};

/// Zoned instant, as [`LocalTime::at`] returns. Re-exported so a caller needs no direct
/// dependency on [`jiff`].
pub use jiff::Zoned;

/// The instant a track's timestamps count from, in the zone the track starts in.
pub struct LocalTime(Zoned);

impl LocalTime {
    /// `None` when the log has no usable date.
    pub fn new(log: &Log) -> Option<Self> {
        let origin: Timestamp = log
            .datetime()?
            .parse()
            .inspect_err(|e| warn!("Log states no usable date: {e}"))
            .ok()?;

        Some(LocalTime(origin.to_zoned(zone(log))))
    }

    /// Wall clock at `timestamp`, a [`Fix::timestamp`](rustigc::Fix::timestamp) in milliseconds.
    pub fn at(&self, timestamp: u32) -> Zoned {
        &self.0 + SignedDuration::from_millis(i64::from(timestamp))
    }

    /// The instant timestamps count from.
    pub fn origin(&self) -> &Zoned {
        &self.0
    }
}

/// Zone the track starts in, falling back to the offset the log declares and then to UTC.
fn zone(log: &Log) -> TimeZone {
    if let Some(name) = log.track.first().and_then(lookup) {
        match TimeZone::get(&name) {
            Ok(tz) => return tz,
            // Boundary data and tzdb release separately; a name may be missing from the tzdb.
            Err(_) => warn!("Timezone {name} is not in the tzdb"),
        }
    }

    if let Some(hours) = log.tzn() {
        let seconds = (hours * 3600.0).round() as i32;
        if let Ok(offset) = Offset::from_seconds(seconds) {
            return TimeZone::fixed(offset);
        }
    }

    warn!("No timezone for this track, reading times as UTC");
    TimeZone::UTC
}

/// IANA name of the zone holding `first`.
fn lookup(first: &Fix) -> Option<String> {
    let finder = utz::Finder::new()
        .inspect_err(|e| warn!("No timezone finder: {e}"))
        .ok()?;

    let position = utz::Position {
        lon: first.lon,
        lat: first.lat,
    };

    match finder.lookup(position) {
        Ok(Some(name)) => Some(name.to_string()),
        Ok(None) => {
            warn!("No timezone covers {},{}", first.lat, first.lon);
            None
        }
        Err(e) => {
            warn!("First fix is not a position: {e}");
            None
        }
    }
}
