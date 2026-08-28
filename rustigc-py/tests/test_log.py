# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test Python Log wrapper"""
from datetime import UTC, date, timedelta

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
    """The origin comes from Rust, in the zone the track starts in"""
    log = Log.from_bytes(igc_content)

    origin = log.datetime

    assert origin.date() == date(2022, 8, 5)
    # UTC midnight read in BST, so an hour past it and an hour east, and named after the zone
    assert origin.hour == 1
    assert origin.utcoffset() == timedelta(hours=1)
    assert origin.tzname() == "BST"
    assert str(origin.tzinfo) == "Europe/London"
    # a real zone, so it follows its own rules away from the flight
    assert (origin + timedelta(days=182)).tzname() == "GMT"
    # whatever the zone, the instant is midnight UTC: that is what a fix timestamp counts from
    assert origin.astimezone(UTC).strftime("%Y-%m-%d %H:%M:%S") == "2022-08-05 00:00:00"


def test_datetime_at_without_date():
    """No date header, nothing to date a fix against"""
    log = Log.from_bytes(b"AFLA1BX\n"
                         b"B0000004449144N00643725EA0058700558\n"
                         b"B1000004449144N00643725EA0058700558\n")
    assert log.datetime is None
    assert log.datetime_at(3600) is None


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
def test_flights_fresh(igc_content):
    """Detection runs per call, and a log is immutable so it always agrees with itself"""
    log = Log.from_bytes(igc_content)
    first, second = log.flights(), log.flights()

    assert first is not second
    assert [(f.start, f.stop) for f in first] == [(f.start, f.stop) for f in second]


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_with_track_leaves_the_original(igc_content):
    """The replacement is a new log; the one it came from keeps its track"""
    log = Log.from_bytes(igc_content)
    shorter = log.with_track(log.track._data[1000:])

    assert len(shorter.track) == 25459 - 1000
    assert len(log.track) == 25459
    assert shorter.pilot_name == log.pilot_name
    assert shorter.score("xcontest") is not None


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_with_track_rejects_unordered(igc_content):
    """Timestamps must stay strictly increasing"""
    log = Log.from_bytes(igc_content)
    unordered = log.track._data.copy()
    unordered["timestamp"][5] = unordered["timestamp"][4]

    with pytest.raises(ValueError, match="strictly increasing"):
        log.with_track(unordered)


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
