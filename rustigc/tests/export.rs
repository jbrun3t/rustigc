// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use std::fs;

use rustigc::{FlightDetection, FlightSelection, Log, Ruleset, TrackError, TrackLine};
use rustigc_test_data::real;
use serde_json::Value;

fn fixture(stem: &str) -> Vec<u8> {
    fs::read(real().join(format!("{stem}.igc"))).expect("fixture")
}

fn roles(features: &[Value]) -> Vec<&str> {
    features
        .iter()
        .map(|f| f["properties"]["role"].as_str().unwrap())
        .collect()
}

#[test]
fn describe_fai_01() {
    let log = Log::new(&fixture("fai-01")).unwrap();
    let described =
        serde_json::to_value(log.describe(Ruleset::named("xcontest").unwrap()).unwrap())
            .unwrap();

    let features = described["features"].as_array().unwrap();
    let roles = roles(features);
    for role in ["track", "metadata", "score"] {
        assert_eq!(
            roles.iter().filter(|r| **r == role).count(),
            1,
            "{role} of a closed triangle"
        );
    }

    assert_eq!(roles.iter().filter(|r| **r == "closing").count(), 3);
    assert_eq!(roles.iter().filter(|r| **r == "leg").count(), 3);
    // takeoff and landing, then the entry, three turnpoints and the exit
    assert_eq!(roles.iter().filter(|r| **r == "marker").count(), 7);

    // the reference every timestamp counts from
    let metadata = features
        .iter()
        .find(|f| f["properties"]["role"] == "metadata")
        .unwrap();
    assert_eq!(
        metadata["properties"]["datetime"],
        "2022-06-21T00:00:00.000Z"
    );
    // what the recorder itself declared, `HFTZNTIMEZONE:1.0`
    assert_eq!(metadata["properties"]["tzn"], 1.0);

    // the same numbers `--format json` pins, over the window detection picks
    let scored = features
        .iter()
        .find(|f| f["properties"]["role"] == "score")
        .unwrap();
    assert_eq!(scored["geometry"], Value::Null);
    assert_eq!(scored["properties"]["rule"], "closed fai triangle");
    assert_eq!(scored["properties"]["distance_km"], 622.85);
    assert_eq!(scored["properties"]["score"], 996.56);
    assert_eq!(scored["properties"]["gap_m"], 67.275);

    // every leg names its ends after markers that are actually there
    let markers: Vec<&str> = features
        .iter()
        .filter(|f| f["properties"]["role"] == "marker")
        .map(|f| f["properties"]["name"].as_str().unwrap())
        .collect();
    for f in features {
        if let Some(from) = f["properties"]["from"].as_str() {
            let to = f["properties"]["to"].as_str().unwrap();
            assert!(markers.contains(&from), "{from} names no marker");
            assert!(markers.contains(&to), "{to} names no marker");
        }
    }
}

/// A layer produced from a longer track is refused, not drawn past the end.
#[test]
fn export_layer_out_of_range() {
    let log = Log::new(&fixture("fai-01")).unwrap();
    let window = log.track.flights().longest().copied().unwrap();
    let scored = log
        .score(
            Ruleset::named("xcontest").unwrap(),
            window.start,
            window.stop,
        )
        .unwrap();

    let shorter = Log {
        track: log.track[20_000..].to_vec(),
        ..log.clone()
    };

    let err = shorter
        .export(Some(window), scored.as_ref(), TrackLine::Draw)
        .unwrap_err();

    assert_eq!(
        err,
        TrackError::FixOutOfRange {
            index: window.stop,
            len: shorter.track.len()
        }
    );
    // its own layers still draw
    assert!(
        log.export(Some(window), scored.as_ref(), TrackLine::Draw)
            .is_ok()
    );
}
