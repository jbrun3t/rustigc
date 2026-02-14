"""Test fixtures"""
import pytest
from pathlib import Path


def pytest_generate_tests(metafunc):
    """Parametrize tests with all real IGC files"""
    if "real_igc_file" in metafunc.fixturenames:
        real_dir = Path(__file__).parent.parent.parent / "test_data/real"
        real_files = sorted(real_dir.rglob("*.igc"))
        real_files = [f for f in real_files if f.is_file()]

        # Use relative paths for cleaner test output
        metafunc.parametrize(
            "real_igc_file",
            real_files,
            ids=[f.relative_to(real_dir).as_posix() for f in real_files]
        )

@pytest.fixture
def real_igc_content(real_igc_file):
    return real_igc_file.read_bytes()

@pytest.fixture
def test_data_dir():
    """Path to test_data directory"""
    return Path(__file__).parent.parent.parent / "test_data"


@pytest.fixture
def igc_content(request, test_data_dir):
    """IGC file content"""
    return (test_data_dir / "real" / request.param).read_bytes()
