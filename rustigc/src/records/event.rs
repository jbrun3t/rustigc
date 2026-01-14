use std::fmt;

use winnow::error::Result as PResult;
use winnow::prelude::*;
use winnow::{
    ascii::{line_ending, till_line_ending},
    combinator::delimited,
};

use super::utils::ts_to_igc;
use super::utils::ts_to_sec;
use super::Record;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct TimedEvent<'a> {
    pub timestamp: u32,
    pub text: &'a str,
}

impl<'a> fmt::Display for TimedEvent<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}: {}", self.timestamp, self.text)
        } else {
            write!(f, "{}{}", ts_to_igc(self.timestamp), self.text)
        }
    }
}

pub fn d_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('D', till_line_ending, line_ending)
        .map(|text: &str| Record::D(text))
        .parse_next(input)
}

pub fn l_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('L', till_line_ending, line_ending)
        .map(|text: &str| Record::L(text))
        .parse_next(input)
}

pub fn g_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('G', till_line_ending, line_ending)
        .map(|text: &str| Record::G(text))
        .parse_next(input)
}

pub fn e_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('E', (ts_to_sec, till_line_ending), line_ending)
        .map(|(timestamp, text): (_, &str)| Record::E(TimedEvent { timestamp, text }))
        .parse_next(input)
}

pub fn f_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('F', (ts_to_sec, till_line_ending), line_ending)
        .map(|(timestamp, text): (_, &str)| Record::F(TimedEvent { timestamp, text }))
        .parse_next(input)
}

pub fn k_record<'a>(input: &mut &'a str) -> PResult<Record<'a>> {
    delimited('K', (ts_to_sec, till_line_ending), line_ending)
        .map(|(timestamp, text): (_, &str)| Record::K(TimedEvent { timestamp, text }))
        .parse_next(input)
}
