# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test Python Log wrapper"""
from datetime import date

import pytest
from rustigcpy import Log


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_from_bytes(igc_content):
    """Parse from bytes"""
    log = Log.from_bytes(igc_content)
    assert log is not None
    assert len(log.track) == 25459


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_from_file(igc_content, test_data_dir):
    """Parse from file"""
    log = Log.from_file(str(test_data_dir / "real" / "fai-01.igc"))
    assert log is not None
    assert len(log.track) == 25459


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_metadata(igc_content):
    """Access pilot name and glider type"""
    log = Log.from_bytes(igc_content)
    assert log.pilot_name == "Mike Young"
    assert log.glider_type == "Ventus 3T"


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_date_parsing(igc_content):
    """Parse date to datetime.date"""
    log = Log.from_bytes(igc_content)
    assert log.datetime.date() == date(2022, 8, 5)


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_flights_bounds_as_fixes(igc_content):
    """Section bounds come back as Fix objects, not indices"""
    log = Log.from_bytes(igc_content)

    flight = log.flights().longest
    assert flight is not None

    assert flight.takeoff == log.track[125]
    assert flight.landing == log.track[25425]


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_cached(igc_content):
    """Track is cached on first access"""
    log = Log.from_bytes(igc_content)
    track1 = log.track
    track2 = log.track
    # Same object (cached)
    assert track1 is track2


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_flights_cached(igc_content):
    """Detection runs once per log"""
    log = Log.from_bytes(igc_content)
    assert log.flights() is log.flights()


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_push_replaces_track(igc_content):
    """A pushed track lands in Rust, fix count and detection follow"""
    log = Log.from_bytes(igc_content)
    before = log.flights()

    log.push(log.track._data[1000:])

    assert len(log.track) == 25459 - 1000
    assert log.flights() is not before
    assert log.score("xcontest") is not None


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_push_rejects_unordered(igc_content):
    """Timestamps must stay strictly increasing, and a refused push changes nothing"""
    log = Log.from_bytes(igc_content)
    unordered = log.track._data.copy()
    unordered["timestamp"][5] = unordered["timestamp"][4]

    with pytest.raises(ValueError, match="strictly increasing"):
        log.push(unordered)

    assert len(log.track) == 25459


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
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
