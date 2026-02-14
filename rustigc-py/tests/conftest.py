from pathlib import Path
import pytest


@pytest.fixture
def test_data_dir():
    """Path to test_data directory"""
    return Path(__file__).parent.parent.parent / "test_data"


@pytest.fixture
def igc_file(request, test_data_dir):
    """IGC file content"""
    return (test_data_dir / "real" / request.param).read_bytes()
