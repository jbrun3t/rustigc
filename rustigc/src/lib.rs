//! RustiGC - Fast IGC file parser library
//!
//! This library provides parsing and analysis of IGC (International Gliding Commission)
//! flight recorder files used in gliding and paragliding.

mod analysis;
mod decode;
mod error;
mod log;
mod rawlog;
mod score;
mod utils;

pub use analysis::Analysis;
pub use decode::Fix;
pub use error::{LError, LResult};
pub use log::Log;
pub use rawlog::RawLog;
pub use score::{league_names, Scorer, ScoringResult};
