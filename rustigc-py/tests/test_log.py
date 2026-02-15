"""Test Python Log wrapper"""
from datetime import date

import pytest
from rustigcpy import Log


@pytest.mark.parametrize("igc_content", ["complex_example_lxn.igc"], indirect=True)
def test_from_bytes(igc_content):
    """Parse from bytes"""
    log = Log.from_bytes(igc_content)
    assert log is not None
    assert len(log.track) == 25459


@pytest.mark.parametrize("igc_content", ["complex_example_lxn.igc"], indirect=True)
def test_from_file(igc_content, test_data_dir):
    """Parse from file"""
    log = Log.from_file(str(test_data_dir / "real" / "complex_example_lxn.igc"))
    assert log is not None
    assert len(log.track) == 25459


@pytest.mark.parametrize("igc_content", ["complex_example_lxn.igc"], indirect=True)
def test_metadata(igc_content):
    """Access pilot name and glider type"""
    log = Log.from_bytes(igc_content)
    assert log.pilot_name == "Mike Young"
    assert log.glider_type == "Ventus 3T"


@pytest.mark.parametrize("igc_content", ["complex_example_lxn.igc"], indirect=True)
def test_date_parsing(igc_content):
    """Parse date to datetime.date"""
    log = Log.from_bytes(igc_content)
    assert log.date == date(2022, 8, 5)


@pytest.mark.parametrize("igc_content", ["complex_example_lxn.igc"], indirect=True)
def test_flight_phases(igc_content):
    """Test takeoff/landing detection"""
    log = Log.from_bytes(igc_content)

    # Should return Fix objects, not indices
    takeoff = log.takeoff
    landing = log.landing

    assert takeoff is not None
    assert landing is not None
    assert hasattr(takeoff, 'latitude')
    assert hasattr(landing, 'latitude')

    # Verify they correspond to expected indices
    assert takeoff == log.track[124]
    assert landing == log.track[25426]


@pytest.mark.parametrize("igc_content", ["complex_example_lxn.igc"], indirect=True)
def test_track_cached(igc_content):
    """Track is cached on first access"""
    log = Log.from_bytes(igc_content)
    track1 = log.track
    track2 = log.track
    # Same object (cached)
    assert track1 is track2


@pytest.mark.parametrize("igc_content", ["complex_example_lxn.igc"], indirect=True)
def test_repr(igc_content):
    """Test Log.__repr__"""
    log = Log.from_bytes(igc_content)
    repr_str = repr(log)
    assert "Log" in repr_str
    assert "fixes=25459" in repr_str
    assert "Mike Young" in repr_str


def test_invalid_content():
    """Handle invalid IGC content"""
    with pytest.raises(ValueError, match="Failed to parse"):
        Log.from_bytes(b"INVALID")
