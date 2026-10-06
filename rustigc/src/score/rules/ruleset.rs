// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

use std::sync::OnceLock;

use super::rule::Rule;
use super::rulesets::RULESET_FILES;
use crate::ScoreError;

/// The rules a flight is scored against, the best of which is reported.
///
/// The embedded ones are reached by name through [`Ruleset::named`].
#[derive(Debug)]
pub struct Ruleset {
    name: String,
    rules: Vec<Rule>,
}

impl Ruleset {
    /// A ruleset called `name`, refused when it has no rule to score.
    pub(super) fn new(name: &str, rules: Vec<Rule>) -> Result<Self, String> {
        if rules.is_empty() {
            return Err("scores nothing".into());
        }

        Ok(Self {
            name: name.into(),
            rules,
        })
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

static REGISTRY: OnceLock<Vec<Ruleset>> = OnceLock::new();

/// Builds every embedded ruleset, once.
///
/// Panics rather than answering with an error: the rulesets are built into the binary, so anything
/// wrong with them is a build-time mistake, and the alternative is a panic at a caller's first score. The
/// registry test walks all of it so CI fails instead.
fn load() -> Vec<Ruleset> {
    let mut registry: Vec<Ruleset> = Vec::with_capacity(RULESET_FILES.len());

    for (file, build) in RULESET_FILES {
        let ruleset = build().unwrap_or_else(|e| panic!("{file}: {e}"));

        assert!(
            !registry.iter().any(|r| r.name == ruleset.name),
            "{file}: ruleset {:?} is already registered",
            ruleset.name
        );

        registry.push(ruleset);
    }

    registry
}

/// The embedded rulesets. A `static` `OnceLock` hands back `&'static`, so nothing has to be leaked to
/// outlive the call that first asked for a ruleset.
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

    /// Forces the registry, so every `load` assert — an unknown shape, an empty rule or
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

    #[test]
    fn named_refuses_unknown() {
        assert_eq!(
            Ruleset::named("nope").unwrap_err(),
            ScoreError::UnknownRuleset
        );
    }

    #[test]
    fn new_refuses_empty_rules() {
        assert!(Ruleset::new("custom", Vec::new()).is_err());
    }
}
