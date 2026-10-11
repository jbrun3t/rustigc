# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""Test Python Ruleset wrapper"""

from pathlib import Path

import pytest
from rustigc_py import Log, Ruleset

PRESETS = Path(__file__).parents[2] / "rustigc/src/score/rules/rulesets"


def test_unknown_preset():
    """A preset id that does not exist is refused"""
    with pytest.raises(ValueError, match="unknown preset"):
        Ruleset.preset("xkontest")


@pytest.mark.parametrize("igc_content", ["fai-01.igc"], indirect=True)
def test_from_json_scores_like_its_preset(igc_content):
    """A preset's JSON content read with from_json scores as the preset does"""
    log = Log.from_bytes(igc_content)
    text = (PRESETS / "xcontest.json").read_text()

    assert log.score(Ruleset.from_json(text))._data == log.score(Ruleset.preset("xcontest"))._data


def test_from_json_refuses_bad_content():
    """Content that is not a usable ruleset is refused, with where it went wrong"""
    with pytest.raises(ValueError, match="line 1 column"):
        Ruleset.from_json('{ "name": "custom", "rules": [] }')
