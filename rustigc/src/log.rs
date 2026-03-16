//! Main IGC file parser
//!
//! This module orchestrates parsing of complete IGC files by dispatching
//! each line to the appropriate record parser.

use std::collections::HashMap;

use crate::geometry::BBox;
use crate::records::*;
use crate::{LError, LResult, RawLog};

use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Bare minimum to be exported when parsing an IGC
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Log {
    /// Logger identification
    pub recorder: Recorder,
    /// headers, lazilly stored
    pub headers: HashMap<String, HeaderData>,
    /// Position fixes (B-records)
    pub track: Vec<Fix>,
    /// Declared Task
    pub task: Option<Task>,
}

impl TryFrom<RawLog<'_>> for Log {
    type Error = LError;

    fn try_from(raw: RawLog) -> Result<Self, Self::Error> {
        let mut headers = HashMap::new();
        let mut track = Vec::new();
        let mut task: Option<Task> = None;
        let mut recorder: Option<Recorder> = None;
        let mut _bextensions: Option<Vec<RecordExtension>> = None;

        for rec in raw.records.into_iter() {
            match rec {
                Record::A(inner) => {
                    recorder = Some(inner);
                }
                Record::B(inner) => {
                    if !inner.valid {
                        continue;
                    }

                    // TODO: Handle LOD/LAD, Possibly TDS
                    track.push(inner.fix);
                }
                Record::C(inner) => {
                    task = Some(inner);
                }
                Record::H(inner) => {
                    headers.insert(inner.key, inner.value);
                }
                Record::I(inner) => {
                    _bextensions = Some(inner.vext);
                }
                _ => {}
            }
        }

        // How can one miss this in the IGC spec ?
        // it is literally the first thing !
        let recorder = recorder.unwrap_or_else(|| {
            warn!("Missing Record \"A\". Please fix your flight recorder");
            Recorder {
                manufacturer: "BAD".into(),
                uid: "123456789ABC".into(),
                data: None,
            }
        });

        Ok(Self {
            recorder,
            headers,
            track,
            task,
        })
    }
}

impl Log {
    pub fn new(input: &[u8]) -> LResult<Self> {
        let raw = RawLog::new(input)?;
        raw.try_into()
    }

    pub fn center(&self) -> (f64, f64) {
        use crate::geometry::{Coords, SphericalPoint};

        match BBox::<SphericalPoint<f64>>::from_items(&self.track) {
            Some(bbox) => {
                let center = bbox.center();
                (center.y(), center.x()) // (lat, lon) - lon_round applied in from_coords
            }
            None => (0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_igc_file() {
        let content = b"AFLA1BX\n\
                       HFDTE150120\n\
                       HFPLTPILOT:Tripoux Robert\n\
                       HFGTYGLIDERTYPE:Piegon 12\n\
                       I023638FXA3940SIU\n\
                       B1101355206343N00006198WA005870055801005\n\
                       LXXX This is a comment\n\
                       B1101365206345N00006200WA005890056004208\n\
                       B1101375206347N00006202WA005910056200212\n\
                       GNOWAYINHELLTHISISVALID\n";

        let log = Log::new(content).unwrap();

        assert_eq!(log.recorder.manufacturer, "FLA");
        assert_eq!(log.headers["DTE"].text, "150120");
        assert_eq!(log.track.len(), 3);
        assert_eq!(log.track[0].timestamp, 39695);
    }
}
