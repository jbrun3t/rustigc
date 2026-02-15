"""Test rustigcpy Rust extension (minimal low-level API)"""
import pytest
import rustigcpy
import numpy

FIX_DTYPE = rustigcpy.FIX_DTYPE


def test_import():
    """Module imports and has version"""
    assert rustigcpy.__version__ is not None


@pytest.mark.parametrize("igc_file", ["complex_example_lxn.igc"], indirect=True)
def test_parse(igc_file):
    """Parse IGC file"""
    log = rustigcpy.Log.from_bytes(igc_file)
    assert log is not None
    assert len(log.track_bytes) == 814688


@pytest.mark.parametrize("igc_file", ["complex_example_lxn.igc"], indirect=True)
def test_track_bytes_numpy(igc_file):
    """Verify numpy conversion works"""
    log = rustigcpy.Log.from_bytes(igc_file)
    track = numpy.frombuffer(log.track_bytes, dtype=FIX_DTYPE)

    assert len(track) == 25459
    assert FIX_DTYPE.itemsize == 32
    assert len(track['latitude']) == 25459
    assert len(track['longitude']) == 25459
    assert len(track['timestamp']) == 25459


@pytest.mark.parametrize("igc_file", ["complex_example_lxn.igc"], indirect=True)
def test_metadata(igc_file):
    """Test metadata access via get_header"""
    log = rustigcpy.Log.from_bytes(igc_file)

    # get_header returns tuple of (text, origin)
    pilot = log.get_header("PLT")
    assert pilot == ("Mike Young", "Flight Recorder")

    glider = log.get_header("GTY")
    assert glider == ("Ventus 3T", "Flight Recorder")

    date = log.get_header("DTE")
    assert date == ("050822", "Flight Recorder")


@pytest.mark.parametrize("igc_file", ["complex_example_lxn.igc"], indirect=True)
def test_flight_phases(igc_file):
    """Test takeoff/landing detection"""
    log = rustigcpy.Log.from_bytes(igc_file)
    assert log.takeoff == 124
    assert log.landing == 25426


def test_invalid_content():
    """Test error handling"""
    with pytest.raises(ValueError, match="Failed to parse IGC file"):
        rustigcpy.Log.from_bytes(b"INVALID")


@pytest.mark.parametrize("igc_file", ["complex_example_lxn.igc"], indirect=True)
def test_repr(igc_file):
    """Test Log.__repr__"""
    log = rustigcpy.Log.from_bytes(igc_file)
    repr_str = repr(log)
    assert "Log" in repr_str
    assert "fixes=25459" in repr_str
    assert "Mike Young" in repr_str
