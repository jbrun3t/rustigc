// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! GeoJSON output for a track, the flights detected in it, and what they scored.

use geojson::{Feature, FeatureCollection, Geometry, JsonObject, JsonValue};

use super::geometry::{
    AntimeridianCheck, BBox, Geodesic, PointCoords, PointDistance, SPoint,
};
use super::iter::pairs;
use super::round_km;
use crate::score::known_league;
use crate::{
    Fix, Flight, FlightDetection, FlightSelection, Log, ScoreError, ScoringResult,
    TrackError,
};

/// Header codes a log labels its flown line with.
const LABELS: [&str; 1] = ["PLT"];

/// One layer of a flight, as GeoJSON.
///
/// Implemented by [`Flight`] and [`ScoringResult`], which is what [`Log::export`] draws. A layer
/// carries fix indices only, so it must be drawn against the track it was produced from.
///
/// # Roles
///
/// Every feature declares a `role`:
///
/// | `role` | geometry | carries |
/// | --- | --- | --- |
/// | `track` | LineString, 3D | the whole flown line and its `coordTimes` |
/// | `marker` | Point, 3D | `name`, `fix`, `timestamp` |
/// | `leg` | LineString | a scored side: `name` (`leg0`…), `from`, `to`, `distance_m` |
/// | `closing` | LineString | a circuit's closing side, named `entry`, `exit` or `gap` |
/// | `metadata` | none | `datetime`, the instant every `timestamp` counts from, and IGC metadata |
/// | `score` | none | `rule`, `score`, `distance_km`, `distance_m`, `gap_km`, `threshold_m`, `penalty`, `multiplier`, `circuit` |
///
/// Markers are named `takeoff`, `landing`, `Entry`, `tp0`…`tp(n-1)` and `Exit`, and that is what a
/// leg's `from`/`to` refer to.
///
/// An open task is drawn as `leg`s alone, `leg0` starting at its entry and the last one ending at
/// its exit. A circuit's `leg`s close over its turnpoints, with `closing` legs — `entry`, `exit`
/// and `gap` — beside them. A leg's `distance_m` is its geodesic length in kilometers, so the legs
/// of a task do not add up to the `score`'s `distance_m`, which is net of the penalty, if any.
///
/// `datetime` is stated once, in ISO8601 format. Every `timestamp` and `coordTimes` entry is
/// milliseconds to add to it. `tzn` beside it, when the log declares one, is the offset to local
/// time in hours — the recorder's own claim, which no dataset here checks.
///
/// Every position over a fix is `[lon, lat, gnss_alt]`, trimmed to eight decimals — finer than
/// any fix records.
pub trait GeoJson: Sync {
    /// Features for `self`, its fix indices resolved against `track`.
    ///
    /// # Errors
    ///
    /// [`TrackError::FixOutOfRange`] when `track` does not hold a fix this layer draws.
    fn features(&self, track: &[Fix]) -> Result<Vec<Feature>, TrackError>;
}

// --- Helpers ---

/// Create properties with a role
fn props(role: &str) -> JsonObject {
    let mut props = JsonObject::new();
    props.insert("role".into(), role.into());
    props
}

/// Eight decimals, about a millimetre
/// It's more than enough and it saves space in the final file
fn round_deg(degrees: f64) -> f64 {
    (degrees * 1e8).round() / 1e8
}

/// Create a 2D position from a fix
fn position(fix: &Fix) -> [f64; 2] {
    [round_deg(fix.lon), round_deg(fix.lat)]
}

/// Create a 3D position from a fix
fn position3d(fix: &Fix) -> [f64; 3] {
    [round_deg(fix.lon), round_deg(fix.lat), fix.gnss_alt as f64]
}

/// Create the timestamp array
fn coord_times(track: &[Fix]) -> Vec<JsonValue> {
    track.iter().map(|fix| fix.timestamp.into()).collect()
}

// --- Geometries ---

/// Fix `index` of `track`, as an error rather than the panic indexing would raise.
fn at(track: &[Fix], index: usize) -> Result<&Fix, TrackError> {
    track.get(index).ok_or(TrackError::FixOutOfRange {
        index,
        len: track.len(),
    })
}

fn marker(
    track: &[Fix],
    fix: &(usize, &str),
    symbol: &str,
    color: &str,
) -> Result<Feature, TrackError> {
    let f = at(track, fix.0)?;
    let mut props = props("marker");
    props.insert("name".into(), fix.1.into());
    props.insert("marker-symbol".into(), symbol.into());
    props.insert("marker-color".into(), color.into());
    props.insert("fix".into(), fix.0.into());
    props.insert("timestamp".into(), f.timestamp.into());

    Ok(Feature {
        geometry: Some(Geometry::new_point(position3d(f))),
        properties: Some(props),
        ..Default::default()
    })
}

/// takeoff and landing markers.
impl GeoJson for Flight {
    fn features(&self, track: &[Fix]) -> Result<Vec<Feature>, TrackError> {
        Ok(vec![
            marker(track, &(self.start, "takeoff"), "airport", "#333333")?,
            marker(track, &(self.stop, "landing"), "cancel", "#333333")?,
        ])
    }
}

/// Everything the rules made of the task.
fn score(result: &ScoringResult) -> Feature {
    let mut props = props("score");
    props.insert("rule".into(), result.description.clone().into());
    props.insert("score".into(), result.score.into());
    props.insert("distance_km".into(), result.distance_km.into());
    props.insert("distance_m".into(), result.distance_m.into());
    props.insert("gap_km".into(), result.gap_km.into());
    props.insert("threshold_m".into(), result.threshold_m.into());
    props.insert("penalty".into(), result.penalty.into());
    props.insert("multiplier".into(), result.multiplier.into());
    props.insert("circuit".into(), result.circuit.into());

    Feature {
        geometry: None,
        properties: Some(props),
        ..Default::default()
    }
}

/// A straight side between two named fixes.
fn leg(
    track: &[Fix],
    from: &(usize, &str),
    to: &(usize, &str),
    role: &str,
    name: &str,
    color: &str,
    width: usize,
) -> Result<Feature, TrackError> {
    let (start, end) = (at(track, from.0)?, at(track, to.0)?);
    let mut props = props(role);
    props.insert("name".into(), name.into());
    props.insert("stroke".into(), color.into());
    props.insert("stroke-width".into(), width.into());
    props.insert("from".into(), from.1.into());
    props.insert("to".into(), to.1.into());
    props.insert(
        "distance_m".into(),
        round_km(Geodesic::distance(start, end)).into(),
    );

    Ok(Feature {
        geometry: Some(Geometry::new_line_string([position(start), position(end)])),
        properties: Some(props),
        ..Default::default()
    })
}

/// The score, the legs of the task, and a marker on every fix the rule picked.
impl GeoJson for ScoringResult {
    fn features(&self, track: &[Fix]) -> Result<Vec<Feature>, TrackError> {
        let mut features = Vec::new();

        // Score as an empty geometry
        features.push(score(self));

        // Create the chain of turnpoints fixes with their names
        let tp_names: Vec<String> = (0..self.turnpoints.len())
            .map(|i| format!("tp{i}"))
            .collect();
        let mut chain: Vec<(usize, &str)> = self
            .turnpoints
            .iter()
            .zip(&tp_names)
            .map(|(&fix, name)| (fix, name.as_str()))
            .collect();

        // Draw them right away
        for tp in chain.iter() {
            features.push(marker(track, tp, "marker", "#0000ff")?);
        }

        // Draw entry and exit
        let entry = (self.entry, "Entry");
        let exit = (self.exit, "Exit");
        features.push(marker(track, &entry, "marker", "#00a000")?);
        features.push(marker(track, &exit, "marker", "#ff0000")?);

        if !self.circuit {
            // Include entry and exit for open tasks
            chain.insert(0, entry);
            chain.push(exit);
        } else {
            // Actual entry and exit legs
            features.push(leg(
                track, &entry, &chain[0], "closing", "entry", "#ff8000", 1,
            )?);
            features.push(leg(
                track,
                &chain[chain.len() - 1],
                &exit,
                "closing",
                "exit",
                "#ff8000",
                1,
            )?);

            // ... and the closing gap
            features.push(leg(track, &exit, &entry, "closing", "gap", "#00a000", 2)?);
        }

        // An empty chain would mean a rule of cardinality below 2.
        assert!(!chain.is_empty());

        let legs: Vec<Feature> = pairs(&chain, self.circuit)
            .enumerate()
            .map(|(n, (from, to))| {
                let name = format!("leg{n}");
                leg(track, from, to, "leg", &name, "#ffff00", 3)
            })
            .collect::<Result<_, _>>()?;

        // Draw the task line
        features.extend(legs);

        Ok(features)
    }
}

/// Whether a log's flown line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackLine {
    /// Include the `track` feature, the whole line as flown.
    Draw,
    /// Leave it out, keeping only the metadata and the layers given.
    Skip,
}

impl Log {
    fn metadata_export(&self) -> Feature {
        let mut props = props("metadata");

        // The reference every timestamp counts from, and whatever shift the log itself declares.
        if let Some(origin) = self.datetime() {
            props.insert("datetime".into(), origin.into());
        }

        if let Some(tzn) = self.tzn() {
            props.insert("tzn".into(), tzn.into());
        }

        let headers: JsonObject = LABELS
            .into_iter()
            .filter_map(|code| {
                Some((code.into(), self.headers.get(code)?.text.clone().into()))
            })
            .collect();

        if !headers.is_empty() {
            props.insert("headers".into(), headers.into());
        }

        Feature {
            geometry: None,
            properties: Some(props),
            ..Default::default()
        }
    }

    fn trackline_export(&self) -> Feature {
        let mut props = props("track");
        props.insert("stroke".into(), "#ff0000".into());
        props.insert("stroke-width".into(), 2.into());

        props.insert("coordTimes".into(), coord_times(&self.track).into());

        Feature {
            geometry: Some(Geometry::new_line_string(self.track.iter().map(position3d))),
            properties: Some(props),
            ..Default::default()
        }
    }

    /// Draws `layers` over this log, `line` deciding whether the flown line comes with them.
    ///
    /// The `metadata` feature is always there.
    ///
    /// # Errors
    ///
    /// [`TrackError::FixOutOfRange`] when a layer reads a fix this log's track does not hold.
    pub fn export_with(
        &self,
        layers: &[&dyn GeoJson],
        line: TrackLine,
    ) -> Result<FeatureCollection, TrackError> {
        let mut features = Vec::new();

        features.push(self.metadata_export());

        if line == TrackLine::Draw {
            features.push(self.trackline_export());
        }

        // Extend with all provided layers
        for layer in layers {
            features.extend(layer.features(&self.track)?);
        }

        // Geojson wants west, south, east, north
        // FIXME: Antimeridian crossing not supported
        let bbox = (!self.track.crosses_antimeridian())
            .then(|| BBox::<SPoint>::from_items(&self.track))
            .flatten()
            .map(|b| {
                [b.min.x(), b.min.y(), b.max.x(), b.max.y()]
                    .map(round_deg)
                    .to_vec()
            });

        Ok(FeatureCollection {
            bbox,
            features,
            foreign_members: None,
        })
    }

    /// Draws `layers` over this log, including the trackline
    pub fn export(
        &self,
        layers: &[&dyn GeoJson],
    ) -> Result<FeatureCollection, TrackError> {
        self.export_with(layers, TrackLine::Draw)
    }

    /// Draws `window` and the task `scored` found in it, each when there is one.
    ///
    /// # Errors
    ///
    /// [`TrackError::FixOutOfRange`] when either was produced from a longer track.
    pub fn export_flight(
        &self,
        window: Option<Flight>,
        scored: Option<&ScoringResult>,
        line: TrackLine,
    ) -> Result<FeatureCollection, TrackError> {
        let mut layers: Vec<&dyn GeoJson> = Vec::new();

        if let Some(window) = &window {
            layers.push(window);
        }

        if let Some(result) = scored {
            layers.push(result);
        }

        self.export_with(&layers, line)
    }

    /// Everything this log describes about itself: the longest flight detected in it, and what
    /// that flight scored under `league`.
    ///
    /// Use [`Log::export`] when the flight and score are already at hand.
    ///
    /// # Errors
    ///
    /// [`ScoreError::UnknownLeague`] when `league` is not one of [`league_names`].
    ///
    /// [`league_names`]: crate::league_names
    pub fn describe(&self, league: &str) -> Result<FeatureCollection, ScoreError> {
        // Detection may find nothing to score, and the name is refused either way.
        known_league(league)?;

        let window = self.track.flights().longest().copied();
        let scored = window
            .map(|w| self.score(league, w.start, w.stop))
            .transpose()?
            .flatten();

        // Nothing here comes from the caller: an out-of-range index would be a bug in detection
        // or scoring, not a failure to report.
        Ok(self
            .export_flight(window, scored.as_ref(), TrackLine::Draw)
            .expect("detection and scoring index this log's own track"))
    }
}
