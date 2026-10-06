// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use std::sync::OnceLock;

use serde::Deserialize;

use super::rule::Rule;
use crate::ScoreError;

/// A named set of rules; scoring reports the best result any of them finds.
///
/// [`Ruleset::named`] returns an embedded one, [`Ruleset::from_json`] reads a caller's.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ruleset {
    name: String,
    #[serde(deserialize_with = "rules")]
    rules: Vec<Rule>,
}

/// Deserializes the rule list and runs [`Rule::check`] on each rule. An empty list is an error.
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
    /// Reads a ruleset in the format of the files under `rulesets/`.
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

    /// The ruleset's name, reported in every result it scores.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn rules(&self) -> &[Rule] {
        &self.rules
    }
}

/// Embeds each file under `rulesets/`, with its name for `load`'s panic messages. Each entry's
/// rulebook URL sits beside it, since JSON has no comments.
macro_rules! rulesets {
    ($( $file:literal ),* $(,)?) => {
        static RULESET_FILES: &[(&str, &str)] = &[
            $(
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
    // Single-rule rulesets, one shape each.
    "1tp.json",
    "2tp.json",
    "line.json",
    "out-and-return.json",
}

static REGISTRY: OnceLock<Vec<Ruleset>> = OnceLock::new();

/// Parses every embedded ruleset. Panics on a bad file or a duplicate name, which
/// `rulesets_all_load` catches in CI.
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

/// Names of the embedded rulesets, in the order the CLI lists them.
pub fn ruleset_names() -> impl Iterator<Item = &'static str> {
    registry().iter().map(|r| r.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses every embedded file, so a bad one fails CI.
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

    /// A caller's ruleset goes through the same checks as an embedded one.
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
