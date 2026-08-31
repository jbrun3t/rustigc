# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test the Scorer binding: that a numpy table reaches Rust intact.

Whether the scores themselves are right is `rustigc`'s own business, so nothing here pins one.
"""
import numpy
import pytest
from rustigc_py import Log, Scorer

TRIANGLE = numpy.array([[45.00, 6.00], [45.05, 6.10], [45.20, 6.30],
                        [45.35, 6.10], [45.20, 5.90], [45.01, 6.01]])


@pytest.mark.parametrize("igc_content", ["triangle-01.igc"], indirect=True)
def test_matches_log_score(igc_content):
    """The same points score the same, whether they arrive as a track or as a table"""
    log = Log.from_bytes(igc_content)
    table = numpy.stack([log.track._latitude, log.track._longitude], axis=1)
    last = len(log.track) - 1

    expected = log.score("xcontest", (0, last))
    score = Scorer(table).score("xcontest")

    assert score._data == expected._data
    # Nothing but coordinates behind a table, so positions come back as the indices themselves
    assert (score.takeoff, score.landing) == (0, last)


def test_accepts_a_strided_table():
    """A table numpy hands over as a stride is made contiguous, and that changes no value"""
    strided = numpy.repeat(TRIANGLE, 2, axis=0)[::2]
    assert not strided.flags["C_CONTIGUOUS"]

    assert Scorer(strided).score("xcontest")._data == Scorer(TRIANGLE).score("xcontest")._data


def test_unknown_league():
    """Refuse unknown leagues"""
    with pytest.raises(ValueError, match="unknown league"):
        Scorer(TRIANGLE).score("xkontest")


@pytest.mark.parametrize("table, message", [
    (numpy.zeros(4), "2-dimensional"),
    (numpy.zeros((4, 3)), r"\(N, 2\)"),
    (numpy.zeros((1, 2)), "2 are the minimum"),
    (numpy.array([[45.0, 6.0], [numpy.nan, 6.1]]), "point 1 is not a finite coordinate"),
    (numpy.array([[45.0, 6.0], [90.5, 6.1]]), "point 1 is not a finite coordinate"),
])
def test_rejects_unusable_tables(table, message):
    """The shape checks are this binding's; the rest is the core naming what is wrong"""
    with pytest.raises(ValueError, match=message):
        Scorer(table)
