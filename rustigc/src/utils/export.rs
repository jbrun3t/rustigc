// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! GeoJSON output for a track, the flights detected in it, and what they scored.
//!
//! Every feature declares a `role`
//! - `track` for the flown line,
//! - `leg` for a scored side of a task,
//! - `marker` for a fix a rule or a detection picked out,
//! - `score` for what the rules made of it
//! -  `metadata` for track side information such as time reference, pilot name, etc ...

use geojson::{Feature, FeatureCollection, Geometry, JsonObject, JsonValue};

use super::geometry::{
    AntimeridianCheck, BBox, Geodesic, PointCoords, PointDistance, SPoint,
};
use super::iter::pairs;
use super::round_km;
use crate::{Fix, Flight, FlightDetection, FlightSelection, Log, ScoringResult};

/// Header codes a log labels its flown line with.
const LABELS: [&str; 1] = ["PLT"];

/// One layer of a flight, as GeoJSON.
pub trait GeoJson: Sync {
    /// Features for `self`, its fix indices resolved against `track`.
    fn features(&self, track: &[Fix]) -> Vec<Feature>;
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

fn marker(track: &[Fix], fix: &(usize, &str), symbol: &str, color: &str) -> Feature {
    let f = &track[fix.0];
    let mut props = props("marker");
    props.insert("name".into(), fix.1.into());
    props.insert("marker-symbol".into(), symbol.into());
    props.insert("marker-color".into(), color.into());
    props.insert("fix".into(), fix.0.into());
    props.insert("timestamp".into(), f.timestamp.into());

    Feature {
        geometry: Some(Geometry::new_point(position3d(f))),
        properties: Some(props),
        ..Default::default()
    }
}

/// Takeoff and landing markers.
impl GeoJson for Flight {
    fn features(&self, track: &[Fix]) -> Vec<Feature> {
        vec![
            marker(track, &(self.start, "Takeoff"), "airport", "#333333"),
            marker(track, &(self.stop, "Landing"), "cancel", "#333333"),
        ]
    }
}

/// Everything the rules made of the task.
fn score(result: &ScoringResult) -> Feature {
    let mut props = props("score");
    props.insert("rule".into(), result.description.clone().into());
    props.insert("score".into(), result.score.into());
    props.insert("distance".into(), result.distance.into());
    props.insert("raw_distance".into(), result.raw_distance.into());
    props.insert("gap".into(), result.gap.into());
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
) -> Feature {
    let mut props = props(role);
    props.insert("name".into(), name.into());
    props.insert("stroke".into(), color.into());
    props.insert("stroke-width".into(), width.into());
    props.insert("from".into(), from.1.into());
    props.insert("to".into(), to.1.into());
    props.insert(
        "distance".into(),
        round_km(Geodesic::distance(&track[from.0], &track[to.0])).into(),
    );

    Feature {
        geometry: Some(Geometry::new_line_string([
            position(&track[from.0]),
            position(&track[to.0]),
        ])),
        properties: Some(props),
        ..Default::default()
    }
}

/// The score, the legs of the task, and a marker on every fix the rule picked.
impl GeoJson for ScoringResult {
    fn features(&self, track: &[Fix]) -> Vec<Feature> {
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
            features.push(marker(track, tp, "marker", "#0000ff"));
        }

        // Draw entry and exit
        let entry = (self.entry, "Entry");
        let exit = (self.exit, "Exit");
        features.push(marker(track, &entry, "marker", "#00a000"));
        features.push(marker(track, &exit, "marker", "#ff0000"));

        if !self.circuit {
            // Include entry and exit for open tasks
            chain.insert(0, entry);
            chain.push(exit);
        } else {
            // Actual entry and exit legs
            features.push(leg(
                track, &entry, &chain[0], "closing", "entry", "#ff8000", 1,
            ));
            features.push(leg(
                track,
                &chain[chain.len() - 1],
                &exit,
                "closing",
                "exit",
                "#ff8000",
                1,
            ));

            // ... and the closing gap
            features.push(leg(track, &exit, &entry, "closing", "gap", "#00a000", 2));
        }

        // At this point, means a cardinality < 2
        assert!(!chain.is_empty());

        let legs: Vec<Feature> = pairs(&chain, self.circuit)
            .enumerate()
            .map(|(n, (from, to))| {
                let name = format!("leg{n}");
                leg(track, from, to, "leg", &name, "#ffff00", 3)
            })
            .collect();

        // Draw the task line
        features.extend(legs);

        features
    }
}

/// Whether a log's flown line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackLine {
    Draw,
    Skip,
}

impl Log {
    fn metadata_export(&self) -> Feature {
        let mut props = props("metadata");

        // Include the datetime reference
        // All flight should have a reference, guarding just in a record is missing DTE header
        if let Some(origin) = self.datetime() {
            props.insert("datetime".into(), origin.to_string().into());
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

    /// Export Log as GeoJson with the timereference, possibly with the trackline and some features
    pub fn export_with(
        &self,
        layers: &[&dyn GeoJson],
        line: TrackLine,
    ) -> FeatureCollection {
        let mut features = Vec::new();

        features.push(self.metadata_export());

        if line == TrackLine::Draw {
            features.push(self.trackline_export());
        }

        // Extend with all provided layers
        for layer in layers {
            features.extend(layer.features(&self.track));
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

        FeatureCollection {
            bbox,
            features,
            foreign_members: None,
        }
    }

    /// Export Log as GeoJson with the trackline and some features
    pub fn export(&self, layers: &[&dyn GeoJson]) -> FeatureCollection {
        self.export_with(layers, TrackLine::Draw)
    }

    // Factoring common code
    /// Draw `window` and the task `scored` found in it, each when there is one.
    pub fn export_flight(
        &self,
        window: Option<Flight>,
        scored: Option<&ScoringResult>,
        line: TrackLine,
    ) -> FeatureCollection {
        let mut layers: Vec<&dyn GeoJson> = Vec::new();

        if let Some(window) = &window {
            layers.push(window);
        }

        if let Some(result) = scored {
            layers.push(result);
        }

        self.export_with(&layers, line)
    }

    /// Everything this log describes about itself: the longest flight detected in it, and what that
    /// flight scored under `league`.
    pub fn describe(&self, league: &str) -> FeatureCollection {
        let window = self.track.flights().longest().copied();
        let scored = window.and_then(|w| self.score(league, w.start, w.stop));

        self.export_flight(window, scored.as_ref(), TrackLine::Draw)
    }
}
