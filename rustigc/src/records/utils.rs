//! Reusable record parsers

use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::take_while;
use winnow::{combinator::alt, token::take};

/// Parse n decimal digits as a number
/// Uses btoi to parse directly from bytes without UTF-8 validation.
/// This is faster than converting to str first since IGC files are ASCII-only.
pub fn n_digits<'a, T, E>(n: usize) -> impl Parser<&'a [u8], T, E>
where
    T: num_traits::PrimInt + num_traits::FromPrimitive,
    E: winnow::error::ParserError<&'a [u8]>,
{
    take(n).verify_map(|bytes: &[u8]| btoi::btoi(bytes).ok())
}

/// take exactly n alphanum
pub fn n_alphanum<'a>(n: usize) -> impl Fn(&mut &'a [u8]) -> PResult<&'a [u8]> {
    move |input: &mut &[u8]| take_while(n..=n, AsChar::is_alphanum).parse_next(input)
}

/// Parse a HHMMSS timestamp and convert it to a number of seconds
pub fn ts_to_sec(input: &mut &[u8]) -> PResult<u32> {
    (
        n_digits(2).verify(|&h| h < 24),
        n_digits(2).verify(|&m| m < 60),
        n_digits(2).verify(|&s| s < 60),
    )
        .map(|(h, m, s): (u32, u32, u32)| ((h * 60) + m) * 60 + s)
        .parse_next(input)
}

pub fn ts_to_igc(input: u32) -> String {
    let input = input % (24 * 60 * 60);
    let (h, rem) = (input / 3600, input % 3600);
    let (m, s) = (rem / 60, rem % 60);
    format!("{:02}{:02}{:02}", h, m, s)
}

/// Parse a arc value in the form (D)DDMMmmm
fn latlon(islat: bool) -> impl Fn(&mut &[u8]) -> PResult<u32> {
    move |input: &mut &[u8]| {
        (
            n_digits(if islat { 2 } else { 3 })
                .verify(|&d| d < (if islat { 90 } else { 180 })),
            n_digits(2).verify(|&m| m < 60),
            n_digits(3).verify(|&mm| mm < 1000),
        )
            .map(|(d, m, mm): (u32, u32, u32)| ((d * 60) + m) * 1000 + mm)
            .parse_next(input)
    }
}

fn latlon_to_igc(input: u32) -> (u32, u32, u32) {
    let (d, rem) = (input / 60000, input % 60000);
    let (m, mm) = (rem / 1000, rem % 1000);
    (d, m, mm)
}

/// Parse a Latitude in the form DDMMmmmN
pub fn latitude(input: &mut &[u8]) -> PResult<i32> {
    (latlon(true), alt(((b'N').value(1), (b'S').value(-1))))
        .map(|(v, s)| (v as i32) * s)
        .parse_next(input)
}

pub fn latitude_to_igc(input: f64) -> String {
    let lat: u32 = (input.abs() * 60000.0) as u32;
    let (d, m, mm) = latlon_to_igc(lat);
    format!(
        "{:02}{:02}{:03}{}",
        d,
        m,
        mm,
        if input < 0.0 { 'S' } else { 'N' }
    )
}

/// Parse a Longitude in the form DDDMMmmmE
pub fn longitude(input: &mut &[u8]) -> PResult<i32> {
    (latlon(false), alt(((b'E').value(1), (b'W').value(-1))))
        .map(|(v, s)| (v as i32) * s)
        .parse_next(input)
}

pub fn longitude_to_igc(input: f64) -> String {
    let lon: u32 = (input.abs() * 60000.0) as u32;
    let (d, m, mm) = latlon_to_igc(lon);
    format!(
        "{:03}{:02}{:03}{}",
        d,
        m,
        mm,
        if input < 0.0 { 'W' } else { 'E' }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        let time = ts_to_sec.parse(b"110135").unwrap();
        // 11:01:35 = 11*3600 + 1*60 + 35 = 39695 seconds
        assert_eq!(time, 39695);
    }

    #[test]
    fn test_parse_bad_time() {
        assert!(ts_to_sec.parse(b"117135").is_err());
    }

    #[test]
    fn test_parse_latitude() {
        // 52°06.343'N
        let lat = latitude.parse(b"5206343N").unwrap();
        assert_eq!(lat, 3126343);

        // Southern hemisphere
        let lat = latitude.parse(b"5206343S").unwrap();
        assert_eq!(lat, -3126343);
    }

    #[test]
    fn test_parse_bad_latitude() {
        assert!(latitude.parse(b"9206343S").is_err());
    }

    #[test]
    fn test_parse_longitude() {
        // 000°06.198'W
        let lon = longitude.parse(b"00006198W").unwrap();
        assert_eq!(lon, -6198);

        // Eastern hemisphere
        let lon = longitude.parse(b"12034567E").unwrap();
        assert_eq!(lon, 7234567);
    }

    #[test]
    fn test_parse_bad_longitude() {
        assert!(latitude.parse(b"5286343E").is_err());
    }
}
