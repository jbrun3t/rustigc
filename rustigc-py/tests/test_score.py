# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test Python Score wrapper"""
import pytest
from rustigcpy import Fix, Log

# Window of fai-01.xcontest.json, the blessed reference's own, not our detection
FAI01_WINDOW = (125, 25457)


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_score_output(igc_content):
    """Score returns properly"""
    log = Log.from_bytes(igc_content)
    score = log.score("xcontest", FAI01_WINDOW)

    assert score.description == "closed fai triangle"
    assert score.distance_km == 622.85
    assert score.distance_m == 622852.252
    assert score.gap_km == 0.07
    assert score.penalty == 0.07
    assert score.score == 996.56
    assert score.multiplier == 1.6
    assert score.circuit is True
    assert isinstance(score.entry, Fix)
    assert score.takeoff.index == 125
    assert score.entry.index == 125
    assert score.exit.index == 25421
    assert score.landing.index == 25457
    assert [tp.index for tp in score.turnpoints] == [1374, 11790, 19270]
    assert score.entry == log.track[125]


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_unknown_league(igc_content):
    """A league name is the caller's, so a wrong one is refused"""
    with pytest.raises(ValueError, match="unknown league"):
        Log.from_bytes(igc_content).score("xkontest")


def test_score_without_a_detected_flight():
    """No flight means no default window, which is refused like any other bad window"""
    tiny = (b"AFLA1BX\nHFDTE150120\n"
            b"B1101355206343N00006198WA005870055801005\n"
            b"B1101365206345N00006200WA005890056004208\n")
    log = Log.from_bytes(tiny)
    assert len(log.flights()) == 0

    with pytest.raises(ValueError, match="no flight detected"):
        log.score("xcontest")


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_repr(igc_content):
    """Test Score.__repr__"""
    score = Log.from_bytes(igc_content).score("xcontest", FAI01_WINDOW)
    repr_str = repr(score)

    assert "Score" in repr_str
    assert "closed fai triangle" in repr_str
    assert "996.56" in repr_str
