// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Wall-clock date-times for fixes
//!
//! A `Fix` timestamp counts seconds from UTC midnight of the flight's first day and carries no
//! date, so rendering one needs an origin: [`Log::datetime`] builds it from the `HFDTE` header and
//! the zone the track starts in, and [`Fix::datetime`] offsets it.
//!
//! The zone comes from the track, not from a header. Two datasets with two jobs get it: `utz`
//! maps a position to an IANA name against timezone-boundary polygons, `jiff` turns that name into
//! offsets against tzdb. They are versioned apart, so a name one knows the other may not — hence
//! every step below falls back to UTC and warns rather than failing. `HFTZN` is not read; a
//! recorder's declared offset is not a zone, and it cannot say what the rules were that day.

use std::sync::OnceLock;

use jiff::civil;
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Zoned};
use log::warn;

use super::utils::date_to_ymd;
use super::Fix;
use crate::Log;

/// Zone the track starts in, UTC when the lookup finds nothing.
///
/// Create the zone finder lazily but pay for it only once.
fn zone(first: &Fix) -> TimeZone {
    static FINDER: OnceLock<Option<utz::Finder>> = OnceLock::new();

    let finder = FINDER.get_or_init(|| {
        utz::Finder::from_static(utz::data::TINY_STATIC)
            .inspect_err(|e| warn!("No timezone finder, reading times as UTC: {e}"))
            .ok()
    });

    let Some(finder) = finder.as_ref() else {
        return TimeZone::UTC;
    };

    let position = utz::Position {
        lon: first.lon,
        lat: first.lat,
    };

    let name = match finder.lookup(position) {
        Ok(Some(name)) => name,
        Ok(None) => {
            warn!(
                "No timezone covers {},{}, reading times as UTC",
                first.lat, first.lon
            );
            return TimeZone::UTC;
        }
        Err(e) => {
            warn!("First fix is not a position, reading times as UTC: {e}");
            return TimeZone::UTC;
        }
    };

    TimeZone::get(name).unwrap_or_else(|_| {
        // The boundary data and the tzdb are versioned apart, so this is the skew showing.
        warn!("Timezone {name} is not in the tzdb, reading times as UTC");
        TimeZone::UTC
    })
}

impl Log {
    /// UTC midnight of the flight's date, in the zone the track starts in
    /// `None` without a usable `HFDTE` header.
    pub fn datetime(&self) -> Option<Zoned> {
        let (y, m, d) =
            date_to_ymd(&mut self.headers.get("DTE")?.text.as_bytes()).ok()?;
        let date = civil::Date::new(y, m, d).ok()?;
        let zone = zone(self.track.first()?);
        Some(date.to_zoned(TimeZone::UTC).ok()?.with_time_zone(zone))
    }
}

impl Fix {
    /// Wall-clock time of this fix, `origin` coming from [`Log::datetime`].
    pub fn datetime(&self, origin: &Zoned) -> Zoned {
        origin + SignedDuration::from_secs(i64::from(self.timestamp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_local() {
        let content = b"AFLA1BX\n\
                        HFDTE150724\n\
                        B1101354449144N00643725EA0058700558\n\
                        B2330004449144N00643725EA0058700558\n\
                        B0030004449144N00643725EA0058700558\n";
        let log = Log::new(content).unwrap();
        let origin = log.datetime().unwrap();
        let shown = |f: &Fix| {
            f.datetime(&origin)
                .strftime("%Y-%m-%d %H:%M:%S %:z")
                .to_string()
        };

        assert_eq!(shown(&log.track[0]), "2024-07-15 13:01:35 +02:00");
        assert_eq!(log.track[2].timestamp, 24 * 3600 + 30 * 60);
        assert_eq!(shown(&log.track[2]), "2024-07-16 02:30:00 +02:00");
    }
}
