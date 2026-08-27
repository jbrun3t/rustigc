# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test rustigcpy Rust extension (minimal low-level API)"""
import json
from concurrent.futures import ThreadPoolExecutor

import numpy
import pytest
import rustigcpy._bindings as rib

FIX_DTYPE = rib.FIX_DTYPE

# Window of fai-01.xcontest.json, the blessed reference's own, not our detection
FAI01_WINDOW = (125, 25457)


def test_import():
    """Module imports and has version"""
    assert rib.__version__ is not None


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_parse(igc_content):
    """Parse IGC file"""
    log = rib.RustLog.from_bytes(igc_content)
    assert log is not None
    assert len(log.track_bytes) == 814688


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_bytes_numpy(igc_content):
    """Verify numpy conversion works"""
    log = rib.RustLog.from_bytes(igc_content)
    track = numpy.frombuffer(log.track_bytes, dtype=FIX_DTYPE)

    assert len(track) == 25459
    assert FIX_DTYPE.itemsize == 32
    assert len(track['latitude']) == 25459
    assert len(track['longitude']) == 25459
    assert len(track['timestamp']) == 25459


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_metadata(igc_content):
    """Test metadata access via get_header"""
    log = rib.RustLog.from_bytes(igc_content)

    # get_header returns tuple of (text, origin)
    pilot = log.get_header("PLT")
    assert pilot == ("Mike Young", "Flight Recorder")

    glider = log.get_header("GTY")
    assert glider == ("Ventus 3T", "Flight Recorder")

    date = log.get_header("DTE")
    assert date == ("050822", "Flight Recorder")


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_flights_detection(igc_content):
    """Detection reports the section bounds, one handle each"""
    log = rib.RustLog.from_bytes(igc_content)
    flights = log.flights()

    assert [json.loads(f.json()) for f in flights] == [{"start": 125, "stop": 25425}]


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
    handle = log.score("xcontest", *FAI01_WINDOW)

    assert set(json.loads(handle.json())) == {
        "league", "description", "distance_m", "distance_km", "gap_km", "penalty",
        "score", "multiplier", "takeoff", "entry", "turnpoints",
        "exit", "landing", "circuit",
    }


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_repr(igc_content):
    """Test Log.__repr__"""
    log = rib.RustLog.from_bytes(igc_content)
    repr_str = repr(log)
    assert "Log" in repr_str
    assert "fixes=25459" in repr_str
    assert "Mike Young" in repr_str
