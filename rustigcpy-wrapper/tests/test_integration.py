"""Integration tests with real IGC files"""
from rustigcpy_wrapper import Log, FIX_DTYPE

def test_numpy_dtype_size():
    """Verify FIX_DTYPE matches struct size (32 bytes)"""
    assert FIX_DTYPE.itemsize == 32

def test_real_files(real_igc_content):
    """Parse all real IGC files"""
    log = Log.from_bytes(real_igc_content)

    assert log is not None
    assert len(log.track) > 0
    assert log.date is not None
