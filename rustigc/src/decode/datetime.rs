// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! The instant a track's timestamps count from
//!
//! A `Fix` timestamp counts milliseconds from UTC midnight of the flight's first day and carries
//! no date, so dating one needs an origin. [`Log::datetime`] comes from `HFDTE` as an UTC date,
//! "not necessarily the local date" (A2.5.4), so west of Greenwich the local day the flight belong
//! to is the one before.
//!
//! The `TZN` header is the recorder's own declared offset. It may be wrong when the pilot set it by
//! hand and left it behind.

use super::utils::date_to_ymd;
use crate::Log;

/// Largest real offset from UTC, in hours. Ours, not the spec's, which states no range: it
/// rejects a recorder writing minutes into an hours field as much as it rejects nonsense.
const TZN_LIMIT: f64 = 14.0;

impl Log {
    /// UTC midnight of the flight's date, in the form `Date.prototype.toISOString` produces —
    /// `2022-08-05T00:00:00.000Z`. `None` without a usable `HFDTE` header.
    pub fn datetime(&self) -> Option<String> {
        let (y, m, d) =
            date_to_ymd(&mut self.headers.get("DTE")?.text.as_bytes()).ok()?;
        Some(format!("{y:04}-{m:02}-{d:02}T00:00:00.000Z"))
    }

    /// Offset from UTC to local time in hours, as the `TZN` header declares it. `None` when the
    /// header is absent or states no plausible offset.
    pub fn tzn(&self) -> Option<f64> {
        let hours: f64 = self.headers.get("TZN")?.text.parse().ok()?;
        (hours.is_finite() && hours.abs() <= TZN_LIMIT).then_some(hours)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADERS: &[u8] = b"AFLA1BX\n\
                             HFDTE150724\n\
                             B1101354449144N00643725EA0058700558\n";

    #[test]
    fn datetime_from_dte() {
        let log = Log::new(HEADERS).unwrap();
        assert_eq!(log.datetime().unwrap(), "2024-07-15T00:00:00.000Z");
        assert_eq!(log.tzn(), None);
    }

    #[test]
    fn datetime_without_dte() {
        let log = Log::new(b"AFLA1BX\nB1101354449144N00643725EA0058700558\n").unwrap();
        assert_eq!(log.datetime(), None);
    }

    #[test]
    fn tzn_spellings() {
        let tzn = |text: &str| {
            let content = format!("AFLA1BX\nHFDTE150724\nHFTZNTIMEZONE:{text}\n");
            Log::new(content.as_bytes()).unwrap().tzn()
        };

        assert_eq!(tzn("1"), Some(1.0));
        assert_eq!(tzn("1.0"), Some(1.0));
        assert_eq!(tzn("2"), Some(2.0));
        assert_eq!(tzn("-5"), Some(-5.0));
        assert_eq!(tzn("5.75"), Some(5.75));
        // hours, so a recorder writing minutes states no offset anyone can use
        assert_eq!(tzn("120"), None);
        assert_eq!(tzn("well past noon"), None);
    }
}
