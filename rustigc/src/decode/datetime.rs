// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Wall-clock date-times for fixes
//!
//! A `Fix` timestamp counts seconds from UTC midnight of the flight's first day and carries no
//! date, so rendering one needs an origin: [`Log::datetime`] builds it from the `HFDTE` header and
//! the zone the track starts in, and [`Fix::datetime`] offsets it.

use std::sync::OnceLock;

use jiff::civil;
use jiff::tz::TimeZone;
use jiff::{SignedDuration, Zoned};

use super::utils::date_to_ymd;
use super::Fix;
use crate::Log;

/// Zone the track starts in, UTC when the lookup finds nothing.
///
/// Create the zone finder lazily but pay for it only once.
fn zone(first: &Fix) -> TimeZone {
    static FINDER: OnceLock<Option<utz::Finder>> = OnceLock::new();

    FINDER
        .get_or_init(|| utz::Finder::from_static(utz::data::TINY_STATIC).ok())
        .as_ref()
        .and_then(|f| {
            f.lookup(utz::Position {
                lon: first.lon,
                lat: first.lat,
            })
            .ok()
            .flatten()
        })
        .and_then(|name| TimeZone::get(name).ok())
        .unwrap_or(TimeZone::UTC)
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
