// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Dating a track.
//!
//! A fix timestamp counts milliseconds from UTC midnight of the `HFDTE` date, which is a UTC date
//! (A2.5.4): west of Greenwich, the local day is the one before. `TZN` is the recorder's declared
//! offset, which may be wrong when set by hand.

use super::utils::date_to_ymd;
use crate::Log;

/// Largest plausible offset from UTC, in hours; the spec states no range. Also rejects minutes
/// written into the hours field.
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
        // minutes in the hours field
        assert_eq!(tzn("120"), None);
        assert_eq!(tzn("well past noon"), None);
    }
}
