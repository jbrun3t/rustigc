//! RustiGC - Fast IGC file parser library
//!
//! This library provides parsing and analysis of IGC (International Gliding Commission)
//! flight recorder files used in gliding and paragliding.

mod analysis;
mod error;
mod geometry;
mod log;
mod projector;
mod rawlog;
mod records;
mod scoring;

pub use analysis::Analysis;
pub use error::{LError, LResult};
pub use log::Log;
pub use rawlog::RawLog;
pub use records::Fix;
pub use scoring::{league_names, Scorer, ScoringResult};
