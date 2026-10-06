// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use std::sync::OnceLock;

use serde::Deserialize;

use super::rule::Rule;
use crate::ScoreError;

/// The rules a flight is scored against, the best of which is reported.
///
/// The embedded ones are reached by name through [`Ruleset::named`]; [`Ruleset::from_json`] reads a
/// caller's own, in the same format.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ruleset {
    name: String,
    #[serde(deserialize_with = "rules")]
    rules: Vec<Rule>,
}

/// Reads the rules a ruleset states and checks what their types cannot. A defect is a
/// deserialization error, so it is reported against the text that holds it like any other.
fn rules<'de, D>(deserializer: D) -> Result<Vec<Rule>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let rules: Vec<Rule> = Deserialize::deserialize(deserializer)?;

    if rules.is_empty() {
        return Err(D::Error::custom("scores nothing"));
    }
    for rule in &rules {
        rule.check().map_err(D::Error::custom)?;
    }

    Ok(rules)
}

impl Ruleset {
    /// Reads a ruleset in the format of the embedded ones.
    ///
    /// # Errors
    ///
    /// Malformed JSON, an unknown shape, or rules a search cannot use: none at all, a variant
    /// disagreeing with its shape on closing, or a rate that improves down a variant list.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// The embedded ruleset called `name`.
    ///
    /// # Errors
    ///
    /// [`ScoreError::UnknownRuleset`] when `name` is not one of [`ruleset_names`].
    pub fn named(name: &str) -> Result<&'static Self, ScoreError> {
        registry()
            .iter()
            .find(|r| r.name == name)
            .ok_or(ScoreError::UnknownRuleset)
    }

    /// The name the ruleset states, which every result it scores reports.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn rules(&self) -> &[Rule] {
        &self.rules
    }
}

/// Embeds each ruleset file, paired with its name for the panics `load` raises.
///
/// The rulebook each one encodes is linked here: JSON takes no comments, and the numbers are no use
/// without the text they came from.
macro_rules! rulesets {
    ($( $(#[$attr:meta])* $file:literal ),* $(,)?) => {
        static RULESET_FILES: &[(&str, &str)] = &[
            $(
                $(#[$attr])*
                ($file, include_str!(concat!("rulesets/", $file))),
            )*
        ];
    };
}

rulesets! {
    // https://parapente.ffvl.fr/sites/parapente.ffvl.fr/files/Reglement_competitions_CFD-2024_25vdef.pdf
    "cfd.json",
    // https://www.xcontest.org/world/en/rules/
    "xcontest.json",
    // Single-rule rulesets, each exercising one shape on its own.
    "1tp.json",
    "2tp.json",
    "line.json",
    "out-and-return.json",
}

static REGISTRY: OnceLock<Vec<Ruleset>> = OnceLock::new();

/// Parses every embedded ruleset, once.
///
/// Panics rather than answering with an error: the JSON ships inside the binary, so anything wrong
/// with it is a build-time mistake, and the alternative is a panic at a caller's first score. The
/// registry test walks all of it so CI fails instead.
fn load() -> Vec<Ruleset> {
    let mut registry: Vec<Ruleset> = Vec::with_capacity(RULESET_FILES.len());

    for (file, text) in RULESET_FILES {
        let ruleset = Ruleset::from_json(text).unwrap_or_else(|e| panic!("{file}: {e}"));

        assert!(
            !registry.iter().any(|r| r.name == ruleset.name),
            "{file}: ruleset {:?} is already registered",
            ruleset.name
        );

        registry.push(ruleset);
    }

    registry
}

/// The embedded rulesets, parsed on first use.
fn registry() -> &'static [Ruleset] {
    REGISTRY.get_or_init(load)
}

/// Every name [`Ruleset::named`] accepts, in the order the CLI shows them.
pub fn ruleset_names() -> impl Iterator<Item = &'static str> {
    registry().iter().map(|r| r.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Forces the registry, so every `load` assert — bad JSON, an unknown shape, an empty rule or
    /// variant list, a duplicate ruleset, a variant disagreeing with its shape on closing, a rate
    /// that improves down a list — fails CI rather than a caller's first score.
    #[test]
    fn rulesets_all_load() {
        let names: Vec<_> = ruleset_names().collect();
        assert!(!names.is_empty(), "no ruleset is embedded");

        for name in &names {
            let ruleset =
                Ruleset::named(name).expect("ruleset unreachable by its own name");
            assert_eq!(ruleset.name(), *name);
        }
    }

    /// A caller's ruleset is held to the same checks as an embedded one: a closing priced over an
    /// open shape would go uncharged.
    #[test]
    fn from_json_refuses_closing_on_open_shape() {
        let text = r#"{
            "name": "custom",
            "rules": [{
                "shape": { "type": "open_polyline", "points": 3 },
                "variants": [{
                    "name": "free",
                    "multiplier": 1.0,
                    "closing": { "type": "ratio", "ratio": 0.2, "penalized": true }
                }]
            }]
        }"#;

        let err = Ruleset::from_json(text).unwrap_err();
        assert!(err.to_string().contains("on closing"), "{err}");
    }

    #[test]
    fn named_refuses_unknown() {
        assert_eq!(
            Ruleset::named("nope").unwrap_err(),
            ScoreError::UnknownRuleset
        );
    }

    #[test]
    fn from_json_refuses_empty_rules() {
        assert!(Ruleset::from_json(r#"{ "name": "custom", "rules": [] }"#).is_err());
    }
}
