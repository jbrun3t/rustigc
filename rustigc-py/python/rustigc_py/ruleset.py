# SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

"""The rules a track is scored against."""

from __future__ import annotations

import rustigc_py._bindings as rib


class Ruleset:
    """A set of rules to score against: a preset, or a caller's own in the presets' JSON format."""

    def __init__(self, ruleset: rib.RustRuleset):
        """Wrap a raw ruleset; use `preset` or `from_json` instead."""
        self._ruleset = ruleset

    @classmethod
    def preset(cls, id: str) -> Ruleset:
        """The preset `id`, one of `preset_names()`.

        Raises:
            ValueError: `id` is not one of `preset_names()`.
        """
        return cls(rib.RustRuleset.preset(id))

    @classmethod
    def from_json(cls, text: str) -> Ruleset:
        """A ruleset from JSON content in the presets' format.

        Raises:
            ValueError: `text` is not a usable ruleset; the message gives the line and column.
        """
        return cls(rib.RustRuleset.from_json(text))

    @property
    def name(self) -> str:
        """The name reported in every result this ruleset scores."""
        return self._ruleset.name

    def __repr__(self) -> str:
        return repr(self._ruleset)
