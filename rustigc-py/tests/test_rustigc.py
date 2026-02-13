"""Tests for rustigc Python bindings"""
import pytest
import rustigcpy
import datetime


def test_import():
    """Test that the module can be imported"""
    assert rustigcpy.__version__ is not None

def test_parse_minimal_igc():
    """Test parsing a minimal IGC file"""
    content = b"""AFLA1BX
HFDTE150120
HFPLTPILOT:John Smith
HFGTYGLIDERTYPE:ASW 27
B1101355206343N00006198WA0058700558
B1101365206345N00006200WA0058900560
"""

    log = rustigcpy.Log.from_bytes(content)

    # Test metadata
    assert log.pilot_name() == "John Smith"
    assert log.glider_type() == "ASW 27"

    # Date returns (year, month, day) tuple
    flight_date = datetime.datetime.strptime(log.date(), '%d%m%y').date()
    # Can convert to datetime.date
    assert flight_date == datetime.date(2020, 1, 15)

    # Test fixes
    assert len(log) == 2

    # Test first fix
    fix0 = log[0]
    # Timestamp returns seconds since midnight
    # 11:01:35 = 11*3600 + 1*60 + 35 = 39695 seconds
    assert fix0.timestamp == 39695

    assert abs(fix0.latitude - 52.10571666666667) < 0.0001
    assert abs(fix0.longitude - -0.10330) < 0.0001
    assert fix0.gnss_altitude == 558

    # Test second fix (11:01:36)
    fix1 = log[1]
    assert fix1.timestamp == 39696

    # Test negative indexing
    assert log[-1].timestamp == 39696
    assert log[-2].timestamp == 39695


def test_fixes_list():
    """Test getting all fixes as a list"""
    content = b"""AFLA1BX
HFDTE150120
B1101355206343N00006198WA0058700558
B1101365206345N00006200WA0058900560
"""

    log = rustigcpy.Log.from_bytes(content)
    fixes = log.fixes()

    assert len(fixes) == 2
    assert fixes[0].timestamp == 39695  # 11:01:35 in seconds
    assert fixes[1].timestamp == 39696  # 11:01:36 in seconds


def test_invalid_file():
    """Test parsing an invalid IGC file"""
    with pytest.raises(ValueError, match="Failed to parse IGC file"):
        rustigcpy.Log.from_bytes(b"INVALID CONTENT")


def test_index_out_of_range():
    """Test accessing fix with invalid index"""
    content = b"""AFLA1BX
B1101355206343N00006198WA0058700558
"""

    log = rustigcpy.Log.from_bytes(content)

    with pytest.raises(ValueError, match="Index out of range"):
        _ = log[10]

    with pytest.raises(ValueError, match="Index out of range"):
        _ = log[-10]


def test_repr():
    """Test __repr__ methods"""
    content = b"""AFLA1BX
HFDTE150120
HFPLTPILOT:Test
B1101355206343N00006198WA0058700558
"""

    log = rustigcpy.Log.from_bytes(content)

    # Test Log repr
    repr_str = repr(log)
    assert "Log" in repr_str
    assert "fixes=1" in repr_str

    # Test Fix repr
    fix_repr = repr(log[0])
    assert "Fix" in fix_repr

    
def test_real_igc(real_content):
    log = rustigcpy.Log.from_bytes(real_content)

    assert log.takeoff is not None
    assert log.landing is not None
