// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use serde::Deserialize;

use super::rule::Rule;
use crate::ScoreError;

/// A named set of rules; scoring reports the best result any of them finds.
///
/// [`Ruleset::preset`] reads an embedded one, [`Ruleset::from_json`] a caller's.
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
    /// Reads a ruleset in the format of the files under `rulesets/` in this crate.
    ///
    /// # Errors
    ///
    /// Malformed JSON, an unknown shape, or rules a search cannot use: none at all, a variant
    /// disagreeing with its shape on closing, or a rate that improves down a variant list.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// The preset `id`, one of [`preset_names`], parsed on each call.
    ///
    /// # Errors
    ///
    /// [`ScoreError::UnknownPreset`] when `id` is not one of [`preset_names`].
    pub fn preset(id: &str) -> Result<Self, ScoreError> {
        let (_, text) = PRESETS
            .iter()
            .find(|(name, _)| *name == id)
            .ok_or(ScoreError::UnknownPreset)?;

        Ok(Self::from_json(text).unwrap_or_else(|e| panic!("preset {id}: {e}")))
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

/// The presets as `(id, JSON)`. The id is how a caller asks for one; the reported name is the
/// file's `"name"`.
const PRESETS: &[(&str, &str)] = &[
    // https://parapente.ffvl.fr/sites/parapente.ffvl.fr/files/Reglement_competitions_CFD-2024_25vdef.pdf
    ("cfd", include_str!("../../../rulesets/presets/cfd.json")),
    // https://www.xcontest.org/world/en/rules/
    (
        "xcontest",
        include_str!("../../../rulesets/presets/xcontest.json"),
    ),
];

/// Ids of the presets, in the order the CLI lists them.
pub fn preset_names() -> impl Iterator<Item = &'static str> {
    PRESETS.iter().map(|(id, _)| *id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses every preset, so a bad one fails CI.
    #[test]
    fn presets_all_load() {
        for id in preset_names() {
            Ruleset::preset(id).unwrap();
        }
    }

    /// Parses every sample, which are not built in, so a bad one fails CI.
    #[test]
    fn samples_all_load() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/rulesets/samples");
        let mut count = 0;

        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            Ruleset::from_json(&text)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            count += 1;
        }

        assert!(count > 0, "no sample in {dir}");
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
    fn preset_refuses_unknown() {
        assert_eq!(
            Ruleset::preset("nope").unwrap_err(),
            ScoreError::UnknownPreset
        );
    }

    #[test]
    fn from_json_refuses_empty_rules() {
        assert!(Ruleset::from_json(r#"{ "name": "custom", "rules": [] }"#).is_err());
    }
}
