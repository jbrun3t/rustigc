# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test Python Scorer wrapper"""
import numpy
import pytest
from rustigcpy import Log, Scorer

# Window of fai-01.xcontest.json, the blessed reference's own, not our detection
FAI01_WINDOW = (125, 25457)


def coords(log, window):
    """The window's fixes as the (N, 2) [latitude, longitude] table Scorer takes"""
    track = log.track._data[window[0]:window[1] + 1]

    return numpy.stack([track["latitude"], track["longitude"]], axis=1)


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_matches_log_score(igc_content):
    """A table of the window's coordinates scores exactly as the log does over that window"""
    log = Log.from_bytes(igc_content)
    expected = log.score("xcontest", FAI01_WINDOW)

    score = Scorer(coords(log, FAI01_WINDOW)).score("xcontest")

    assert score._data == {
        # A Scorer's window is the whole table, so its indices start at the window
        **expected._data,
        "takeoff": 0,
        "entry": expected.entry.index - FAI01_WINDOW[0],
        "turnpoints": [tp.index - FAI01_WINDOW[0] for tp in expected.turnpoints],
        "exit": expected.exit.index - FAI01_WINDOW[0],
        "landing": expected.landing.index - FAI01_WINDOW[0],
    }


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_positions_are_indices(igc_content):
    """Without a track behind it, a Score reports plain indices rather than fixes"""
    log = Log.from_bytes(igc_content)

    score = Scorer(coords(log, FAI01_WINDOW)).score("xcontest")

    assert score.takeoff == 0
    assert score.landing == FAI01_WINDOW[1] - FAI01_WINDOW[0]
    assert all(isinstance(tp, int) for tp in score.turnpoints)


TRIANGLE = numpy.array([[45.00, 6.00], [45.05, 6.10], [45.20, 6.30],
                        [45.35, 6.10], [45.20, 5.90], [45.01, 6.01]])


def test_accepts_any_layout():
    """Whatever numpy hands over is made C-contiguous float64 first"""
    strided = numpy.repeat(TRIANGLE, 2, axis=0)[::2]
    assert not strided.flags["C_CONTIGUOUS"]

    assert Scorer(TRIANGLE).score("xcontest").distance_km == 93.64
    assert Scorer(strided).score("xcontest").distance_km == 93.64
    assert Scorer(TRIANGLE.astype(numpy.float32)).score("xcontest").distance_km == 93.64


def test_unknown_league():
    """Unknown league scores nothing, as Log.score reports it"""
    table = numpy.array([[45.0, 6.0], [45.1, 6.1]])

    assert Scorer(table).score("xkontest") is None


@pytest.mark.parametrize("table, message", [
    (numpy.zeros(4), "2-dimensional"),
    (numpy.zeros((4, 3)), r"\(N, 2\)"),
    (numpy.array([[45.0, 6.0]]), "not scorable"),
    (numpy.array([[45.0, 6.0], [90.5, 6.1]]), "not scorable"),
    (numpy.array([[45.0, 6.0], [numpy.nan, 6.1]]), "not scorable"),
])
def test_rejects_unusable_tables(table, message):
    """Sanity is checked on the way in, not silently scored"""
    with pytest.raises(ValueError, match=message):
        Scorer(table)
