# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test GeoJSON output"""
import json

import pytest
from rustigcpy import Log


def roles(geojson: str) -> list[str]:
    return [f["properties"]["role"] for f in json.loads(geojson)["features"]]


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_describe(igc_content):
    """Describe picks the flight and scores it"""
    described = json.loads(Log.from_bytes(igc_content).describe("xcontest"))
    counts = {}
    for f in described["features"]:
        role = f["properties"]["role"]
        counts[role] = counts.get(role, 0) + 1

    assert counts == {
        "track": 1, "metadata": 1, "score": 1,
        "leg": 3, "closing": 3, "marker": 7,
    }

    scored = next(f for f in described["features"] if f["properties"]["role"] == "score")
    assert scored["geometry"] is None
    assert scored["properties"]["rule"] == "closed fai triangle"
    assert scored["properties"]["distance_km"] == 622.85


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_export_matches_describe(igc_content):
    """Handing export what describe would pick reaches the same collection"""
    log = Log.from_bytes(igc_content)
    flight = log.flights().longest
    score = log.score("xcontest")

    assert log.export(flight, score) == log.describe("xcontest")


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_export_bare(igc_content):
    """Nothing to draw still describes the track"""
    assert roles(Log.from_bytes(igc_content).export()) == ["metadata", "track"]


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_export_without_track(igc_content):
    """Skipping the line keeps the metadata the markers resolve against"""
    log = Log.from_bytes(igc_content)
    without = roles(log.export(score=log.score("xcontest"), track=False))

    assert "track" not in without
    assert "metadata" in without


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_export_rejects_other(igc_content):
    """Reading the layer back is what refuses it, and it names the field it wanted"""
    log = Log.from_bytes(igc_content)

    with pytest.raises(ValueError, match="Not a score: missing field `league`"):
        log.export(score=log.flights().longest)

    with pytest.raises(ValueError, match="Not a flight: missing field `start`"):
        log.export(flight=log.score("xcontest"))

    # A fix carries no layer to serialize, so it never reaches Rust
    with pytest.raises(TypeError):
        log.export(flight=log.track[0])


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_export_foreign_layer(igc_content):
    """A layer drawn against another log's track, and nothing checks that

    Its indices still point where they did, so a shorter track puts them out of range. Today that
    surfaces as a Rust panic rather than a Python error, hence the broad catch.
    """
    log = Log.from_bytes(igc_content)
    flight = log.flights().longest
    shorter = log.with_track(log.track._data[1000:])

    with pytest.raises(BaseException, match="index out of bounds"):
        shorter.export(flight)
