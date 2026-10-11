// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Parsers shared by the record types.

use winnow::combinator::{eof, not, peek, repeat};
use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{any, take_while};
use winnow::{combinator::alt, token::take};

/// Consumes trailing spaces, then carriage returns.
pub fn trailing_space_cr(input: &mut &[u8]) -> PResult<()> {
    (
        take_while(0.., |c| c == b' '),
        take_while(0.., |c| c == b'\r'),
    )
        .map(|(_, _)| ())
        .parse_next(input)
}

/// A line ending: trailing spaces and `\r`, then `\n` or the end of input, which is not consumed.
pub fn robust_ending_eof<'a>(input: &mut &'a [u8]) -> PResult<&'a [u8]> {
    (trailing_space_cr, alt(("\n".as_bytes(), peek(eof))))
        .map(|(_, ending)| ending)
        .parse_next(input)
}

/// Everything up to the line ending.
pub fn till_robust_ending<'a>(input: &mut &'a [u8]) -> PResult<&'a [u8]> {
    repeat(0.., (not(robust_ending_eof), any))
        .with_taken()
        .map(|(_, taken): ((), &[u8])| taken)
        .parse_next(input)
}

/// Exactly `n` decimal digits, as a number.
pub fn n_digits<'a, T, E>(n: usize) -> impl Parser<&'a [u8], T, E>
where
    T: num_traits::PrimInt + num_traits::FromPrimitive,
    E: winnow::error::ParserError<&'a [u8]>,
{
    take(n).verify_map(|bytes: &[u8]| btoi::btoi(bytes).ok())
}

/// Exactly `n` ASCII alphanumerics.
pub fn n_alphanum<'a>(n: usize) -> impl Fn(&mut &'a [u8]) -> PResult<&'a [u8]> {
    move |input: &mut &[u8]| take_while(n..=n, AsChar::is_alphanum).parse_next(input)
}

/// `HHMMSS` as milliseconds since midnight.
pub fn ts_to_ms(input: &mut &[u8]) -> PResult<u32> {
    (
        n_digits(2).verify(|&h| h < 24),
        n_digits(2).verify(|&m| m < 60),
        n_digits(2).verify(|&s| s < 60),
    )
        .map(|(h, m, s): (u32, u32, u32)| (((h * 60) + m) * 60 + s) * 1000)
        .parse_next(input)
}

/// `DDMMYY` as `(year, month, day)`, the year in 2000–2099.
pub fn date_to_ymd(input: &mut &[u8]) -> PResult<(i16, i8, i8)> {
    (
        n_digits(2).verify(|&d| (1..=31).contains(&d)),
        n_digits(2).verify(|&m| (1..=12).contains(&m)),
        n_digits(2),
    )
        .map(|(d, m, y): (u8, u8, u8)| (2000 + i16::from(y), m as i8, d as i8))
        .parse_next(input)
}

/// Milliseconds as `HHMMSS`, wrapped to one day.
pub fn ts_to_igc(input: u32) -> String {
    let input = (input / 1000) % (24 * 60 * 60);
    let (h, rem) = (input / 3600, input % 3600);
    let (m, s) = (rem / 60, rem % 60);
    format!("{:02}{:02}{:02}", h, m, s)
}

/// An angle `(D)DDMMmmm`, in thousandths of a minute: two degree digits below 90 for a latitude,
/// three below 180 for a longitude.
fn latlon(islat: bool) -> impl Fn(&mut &[u8]) -> PResult<u32> {
    move |input: &mut &[u8]| {
        (
            n_digits(if islat { 2 } else { 3 })
                .verify(|&d| d < (if islat { 90 } else { 180 })),
            n_digits(5).verify(|&mm| mm < 60000),
        )
            .map(|(d, mm): (u32, u32)| d * 60000 + mm)
            .parse_next(input)
    }
}

/// Thousandths of a minute as `(degrees, thousandths of a minute)`.
fn latlon_to_igc(input: u32) -> (u32, u32) {
    let (d, mm) = (input / 60000, input % 60000);
    (d, mm)
}

/// `DDMMmmm` then `N`/`S` (or `+`/`-`), in signed thousandths of a minute.
pub fn latitude(input: &mut &[u8]) -> PResult<i32> {
    (
        latlon(true),
        alt((
            (b'N').value(1),
            (b'S').value(-1),
            (b'+').value(1),
            (b'-').value(-1),
        )),
    )
        .map(|(v, s)| (v as i32) * s)
        .parse_next(input)
}

/// Degrees as `DDMMmmmN` or `DDMMmmmS`.
pub fn latitude_to_igc(input: f64) -> String {
    let lat: u32 = (input.abs() * 60000.0).round() as u32;
    let (d, mm) = latlon_to_igc(lat);
    format!("{:02}{:05}{}", d, mm, if input < 0.0 { 'S' } else { 'N' })
}

/// `DDDMMmmm` then `E`/`W` (or `+`/`-`), in signed thousandths of a minute.
pub fn longitude(input: &mut &[u8]) -> PResult<i32> {
    (
        latlon(false),
        alt((
            (b'E').value(1),
            (b'W').value(-1),
            (b'+').value(1),
            (b'-').value(-1),
        )),
    )
        .map(|(v, s)| (v as i32) * s)
        .parse_next(input)
}

/// Degrees as `DDDMMmmmE` or `DDDMMmmmW`.
pub fn longitude_to_igc(input: f64) -> String {
    let lon: u32 = (input.abs() * 60000.0).round() as u32;
    let (d, mm) = latlon_to_igc(lon);
    format!("{:03}{:05}{}", d, mm, if input < 0.0 { 'W' } else { 'E' })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time() {
        let time = ts_to_ms.parse(b"110135").unwrap();
        // 11:01:35 = 11*3600 + 1*60 + 35 = 39695 seconds = 39695000 ms
        assert_eq!(time, 39_695_000);
    }

    #[test]
    fn test_parse_bad_time() {
        assert!(ts_to_ms.parse(b"117135").is_err());
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(date_to_ymd.parse(b"150120").unwrap(), (2020, 1, 15));
    }

    #[test]
    fn test_parse_date_nn() {
        let (rest, ymd) = date_to_ymd.parse_peek(&b"180825,01"[..]).unwrap();
        assert_eq!(ymd, (2025, 8, 18));
        assert_eq!(rest, b",01");
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
        assert!(longitude.parse(b"5286343N").is_err());
    }
}
