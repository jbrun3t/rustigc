// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Parsing and cross-country scoring of IGC flight recorder files, for free-flying sports.
//!
//! # Parsing
//!
//! [`Log`] is the entry point: it decodes an IGC file into flight metadata and a track of
//! [`Fix`]es, dropping the fixes that are invalid or out of order.
//!
//! ```
//! use rustigc::Log;
//!
//! let content = b"AFLA1BX\n\
//!                 HFDTE150120\n\
//!                 HFPLTPILOT:Tripoux Robert\n\
//!                 B1101355206343N00006198WA005870055801005\n\
//!                 B1101365206345N00006200WA005890056004208\n";
//!
//! let log = Log::new(content)?;
//!
//! assert_eq!(log.headers["PLT"].text, "Tripoux Robert");
//! assert_eq!(log.track.len(), 2);
//! assert_eq!(log.track[0].timestamp, 39_695_000); // milliseconds from UTC midnight
//! # Ok::<(), rustigc::DecodeError>(())
//! ```
//!
//! [`RawLog`] is the record-level view. It borrows from the input, keeps every record including
//! the ones [`Log`] discards, prints back valid IGC, and converts into a [`Log`].
//!
//! # Scoring
//!
//! Scoring works on a fix window: [`FlightDetection::flights`] cuts a track into [`Flight`]
//! sections and [`FlightSelection::longest`] picks one. [`Log::score`] scores that window against
//! a league and reports the best rule as a [`ScoringResult`]. [`league_names`] lists the leagues.
//!
//! ```no_run
//! use rustigc::{FlightDetection, FlightSelection, Log};
//!
//! let log = Log::new(&std::fs::read("flight.igc")?)?;
//! let flights = log.track.flights();
//! let flight = flights.longest().expect("no flight detected");
//!
//! if let Some(result) = log.score("xcontest", flight.start, flight.stop) {
//!     println!("{}: {} points over {} km", result.description, result.score, result.distance_km);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [`Scorer`] needs no [`Log`]: it reads coordinates through [`PointCoords`], so a table of bare
//! `[latitude, longitude]` pairs scores too.
//!
//! ```
//! use rustigc::Scorer;
//!
//! let points = vec![[45.00, 6.00], [45.05, 6.10], [45.20, 6.30],
//!                   [45.35, 6.10], [45.20, 5.90], [45.01, 6.01]];
//!
//! let scorer = Scorer::new(&points, 0, points.len() - 1).expect("unusable window");
//! let result = scorer.solve("xcontest").expect("nothing scored");
//!
//! assert_eq!(result.description, "closed free triangle");
//! ```
//!
//! # Feature flags
//!
//! - `serde`: `Serialize`/`Deserialize` on the parsed records and on [`ScoringResult`].
//! - `geojson`: drawing, through `Log::describe` and `Log::export`. See `GeoJson`.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod analysis;
mod decode;
mod error;
mod log;
mod rawlog;
mod score;
mod utils;

pub use analysis::flight::{Flight, FlightDetection, FlightSelection};
pub use decode::{
    Declaration, Extensions, Fix, Header, HeaderData, HeaderOrigin, RawFix, Record,
    RecordExtension, Recorder, Task, TextEvent, TimedEvent, TurnPoint,
};
pub use error::{DecodeError, TrackError};
pub use log::Log;
pub use rawlog::RawLog;
pub use score::{league_names, Scorer, ScoringResult};
#[cfg(feature = "geojson")]
#[cfg_attr(docsrs, doc(cfg(feature = "geojson")))]
pub use utils::export::{GeoJson, TrackLine};
pub use utils::geometry::{PointCoords, PointNew, PointSetCoords};
