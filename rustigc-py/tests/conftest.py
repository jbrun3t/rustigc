from pathlib import Path
import pytest

def read_path(path: Path) -> bytes:
    return path.read_bytes()
    
def pytest_generate_tests(metafunc):
    if "real_file" in metafunc.fixturenames:
        real_dir = Path(__file__).parent.parent.parent / "test_data/real"
        real_files = sorted(real_dir.rglob("*.igc"))
        real_files = [f for f in real_files if f.is_file()]

        # Beautify the output a bit relative path to the data_dir
        metafunc.parametrize("real_file", real_files,
            ids=[f.relative_to(real_dir).as_posix() for f in real_files])

@pytest.fixture
def test_data_dir(request):
    return request.config.getoption('--data-dir')

@pytest.fixture
def real_content(real_file):
    """Prepare test file before benchmarking (not timed)."""
    return read_path(real_file)
