# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test rustigc_py Rust extension (minimal low-level API)"""

import json

import numpy
import pytest
import rustigc_py._bindings as rib

FIX_DTYPE = rib.FIX_DTYPE

# Window of fai-01.xcontest.json, the blessed reference's own, not our detection
FAI01_WINDOW = (125, 25457)


def test_import():
    """Module imports and has version"""
    assert rib.__version__ is not None


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_bytes_is_fix_dtype(igc_content):
    """FIX_DTYPE mirrors `#[repr(C)] Fix`, so numpy reads the raw bytes without a copy"""
    log = rib.RustLog.from_bytes(igc_content)
    track = numpy.frombuffer(log.track_bytes, dtype=FIX_DTYPE)

    assert len(log.track_bytes) == len(track) * FIX_DTYPE.itemsize
    assert FIX_DTYPE.itemsize == 32
    assert {name: offset for name, (_, offset) in FIX_DTYPE.fields.items()} == {
        "timestamp": 0,
        "_pad": 4,
        "latitude": 8,
        "longitude": 16,
        "baro_altitude": 24,
        "gnss_altitude": 28,
    }


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_header_origin(igc_content):
    """get_header renders HeaderOrigin as the string the wrapper reads"""
    log = rib.RustLog.from_bytes(igc_content)

    assert log.get_header("PLT") == ("Mike Young", "Flight Recorder")
    assert log.get_header("XXX") is None


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_flights_shape(igc_content):
    """Detection hands back one JSON array of sections"""
    log = rib.RustLog.from_bytes(igc_content)

    assert [set(f) for f in json.loads(log.flights())] == [{"start", "stop"}]


def test_invalid_content():
    """Test error handling"""
    with pytest.raises(ValueError, match="Failed to parse IGC file"):
        rib.RustLog.from_bytes(b"INVALID")


def test_league_names():
    """Registry is reachable and holds the real leagues"""
    names = rib.league_names()
    assert "cfd" in names
    assert "xcontest" in names


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_score_shape(igc_content):
    """Result carries exactly the fields the wrapper expects"""
    log = rib.RustLog.from_bytes(igc_content)
    scored = log.score("xcontest", *FAI01_WINDOW)

    assert set(json.loads(scored)) == {
        "league",
        "description",
        "distance_m",
        "distance_km",
        "gap_km",
        "threshold_m",
        "penalty",
        "score",
        "multiplier",
        "takeoff",
        "entry",
        "turnpoints",
        "exit",
        "landing",
        "circuit",
    }


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_repr(igc_content):
    """Test Log.__repr__"""
    log = rib.RustLog.from_bytes(igc_content)
    repr_str = repr(log)
    assert "Log" in repr_str
    assert "fixes=25459" in repr_str
    assert "Mike Young" in repr_str
