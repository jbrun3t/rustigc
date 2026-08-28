# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

from pathlib import Path
import pytest

@pytest.fixture
def test_data_dir():
    """Path to test_data directory"""
    return Path(__file__).parent.parent.parent / "test_data"


@pytest.fixture
def igc_content(request, test_data_dir):
    """IGC file content"""
    return (test_data_dir / "real" / request.param).read_bytes()
