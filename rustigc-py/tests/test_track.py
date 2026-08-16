"""Test Track wrapper"""
import numpy
import pytest
from rustigcpy import Log


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_properties(igc_content):
    """Access latitude, longitude, altitude properties"""
    log = Log.from_bytes(igc_content)
    track = log.track

    assert len(track) == 25459
    assert isinstance(track._latitude, numpy.ndarray)
    assert track._latitude.dtype == numpy.float64
    assert track._longitude.dtype == numpy.float64
    assert track._baro_altitude.dtype == numpy.int32
    assert track._gnss_altitude.dtype == numpy.int32
    assert track._timestamp.dtype == numpy.uint32


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_indexing(igc_content):
    """Support indexing and slicing"""
    log = Log.from_bytes(igc_content)
    track = log.track

    fix = track[0]
    assert fix is not None
    assert hasattr(fix, 'latitude')

    subset = track._npdata[0:100]
    assert len(subset) == 100

@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_fix_object(igc_content):
    """Test Fix object properties"""
    log = Log.from_bytes(igc_content)
    track = log.track

    fix = track[0]
    assert isinstance(fix.latitude, float)
    assert isinstance(fix.longitude, float)
    assert isinstance(fix.baro_altitude, int)
    assert isinstance(fix.gnss_altitude, int)
    assert isinstance(fix.timestamp, int)
    assert "Fix" in repr(fix)


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_numpy_operations(igc_content):
    """Test vectorized numpy operations"""
    log = Log.from_bytes(igc_content)
    track = log.track

    mean_lat = track._latitude.mean()
    max_alt = track._baro_altitude.max()
    min_alt = track._baro_altitude.min()

    assert isinstance(mean_lat, (float, numpy.floating))
    assert isinstance(max_alt, (int, numpy.integer))
    assert max_alt >= min_alt


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_repr(igc_content):
    """Test Track.__repr__"""
    log = Log.from_bytes(igc_content)
    repr_str = repr(log.track)

    assert "Track" in repr_str
    assert "fixes=25459" in repr_str


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_coordinates(igc_content):
    """Test coordinate values are in valid ranges"""
    log = Log.from_bytes(igc_content)
    track = log.track

    assert numpy.all(track._latitude >= -90)
    assert numpy.all(track._latitude <= 90)
    assert numpy.all(track._longitude >= -180)
    assert numpy.all(track._longitude <= 180)


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_track_iteration(igc_content):
    """Test iterating over track"""
    log = Log.from_bytes(igc_content)
    track = log.track

    count = 0
    for fix in track:
        assert hasattr(fix, 'latitude')
        count += 1
        if count >= 10:
            break
