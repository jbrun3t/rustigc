// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Rules: what a league scores, and over which shape.
//!
//! One file per league. A league's numbers belong in that file and nowhere else.

use std::fmt::Debug;

use super::shapes::{ShapeBuilder, ShapeKind};

pub mod cfd;
pub mod misc;
pub mod xcontest;

#[cfg(feature = "crazy-test")]
pub mod crazy;

pub use super::shapes::balanced::BalancedCircuit;
pub use super::shapes::closed::ClosedCircuit;
pub use super::shapes::polyline::OpenPolyline;

/// What a rule made of a geometry, in meters, unrounded.
#[derive(Debug)]
pub struct Scored {
    /// Distance net of the penalty.
    pub distance: f64,
    pub penalty: f64,
    /// 0 when the minimum is not met: there is a path, it is worth nothing.
    pub score: f64,
    pub multiplier: f64,
    pub description: &'static str,
}

pub trait RuleShape: Debug {
    fn cardinality(&self) -> usize;
    fn shape(&self) -> ShapeBuilder;
}

pub trait RuleScore: Debug {
    /// Score in meters, unrounded — the search prunes on this.
    ///
    /// Must be non-decreasing in `distance` and non-increasing in `gap`: that is what makes the
    /// largest distance against the smallest gap an upper bound of the subtree. Breaking it,
    /// including with a multiplier that worsens as the gap shrinks, silently breaks the pruning.
    fn score(&self, distance: f64, gap: f64) -> f64;
}

pub trait RuleReport: Debug {
    /// How the rule presents that score, once on the winning leaf.
    fn scored(&self, distance: f64, gap: f64) -> Scored;
}

/// One scoring rule of a league.
pub trait Rule: RuleShape + RuleScore + RuleReport + Sync {}
impl<T: RuleShape + RuleScore + RuleReport + Sync> Rule for T {}

/// A league: how a caller asks for it, what it scores, and the numbers its rules take as defaults.
pub trait League {
    const NAME: &'static str;
    /// Nothing ties a listed rule's [`RuleDescription::League`] back to this one.
    const RULES: Ruleset;

    /// Charge for the closing leg. Return INFINITY for a gap this league will not accept.
    fn penalty(_distance: f64, _gap: f64) -> f64 {
        0.0
    }
    /// Least score a rule may report, on the meter scale the search works in.
    fn minimum() -> f64 {
        0.0
    }
}

/// The geometry a rule scores over.
pub trait RuleGeometry: Debug {
    type Shape: ShapeKind;
}

/// A rule paying a multiplier on the distance, net of the closing penalty. Overriding `penalty` or
/// `minimum` diverges from the league without disturbing its other rules.
pub trait RuleDescription: Debug {
    type League: League;

    /// Multiplier and identity, possibly one of several variants of the rule.
    ///
    /// `distance` is raw, before the penalty.
    fn variant(&self, distance: f64, gap: f64) -> (f64, &'static str);

    fn penalty(&self, distance: f64, gap: f64) -> f64 {
        Self::League::penalty(distance, gap)
    }
    fn minimum(&self) -> f64 {
        Self::League::minimum()
    }
}

impl<T: RuleGeometry> RuleShape for T {
    fn cardinality(&self) -> usize {
        T::Shape::CARDINALITY
    }

    fn shape(&self) -> ShapeBuilder {
        <T::Shape as ShapeKind>::create
    }
}

impl<T: RuleDescription> RuleScore for T {
    fn score(&self, distance: f64, gap: f64) -> f64 {
        let (multiplier, _) = self.variant(distance, gap);

        // An infinite penalty leaves 0, which is what a candidate that cannot close is worth.
        (distance - self.penalty(distance, gap)).max(0.0) * multiplier
    }
}

impl<T: RuleDescription> RuleReport for T {
    fn scored(&self, distance: f64, gap: f64) -> Scored {
        let (multiplier, description) = self.variant(distance, gap);
        let penalty = self.penalty(distance, gap);
        let net = (distance - penalty).max(0.0);
        let score = net * multiplier;

        Scored {
            distance: net,
            penalty,
            score: if score >= self.minimum() { score } else { 0.0 },
            multiplier,
            description,
        }
    }
}

/// The rules a caller scores against.
pub type Ruleset = &'static [&'static dyn Rule];

/// A league as the registry sees it.
pub trait LeagueInfo: Sync {
    fn name(&self) -> &'static str;
    fn rules(&self) -> Ruleset;
}

impl<L: League + Sync> LeagueInfo for L {
    fn name(&self) -> &'static str {
        L::NAME
    }

    fn rules(&self) -> Ruleset {
        L::RULES
    }
}

/// League list in the order the CLI will show them
static LEAGUES: &[&'static dyn LeagueInfo] = &[
    &cfd::Cfd,
    &xcontest::Xcontest,
    &misc::OneTurnpoint,
    &misc::TwoTurnpoints,
    &misc::Line,
    &misc::Oar,
    #[cfg(feature = "crazy-test")]
    &crazy::Crazy,
];

pub(crate) fn league_rules(name: &str) -> Option<Ruleset> {
    LEAGUES.iter().find(|l| l.name() == name).map(|l| l.rules())
}

/// Every league `Log::score` accepts, for listing (e.g. a CLI's `--help`).
pub fn league_names() -> impl Iterator<Item = &'static str> {
    LEAGUES.iter().map(|l| l.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leagues_names_unique() {
        let mut names: Vec<_> = league_names().collect();
        let listed = names.len();

        names.sort_unstable();
        names.dedup();

        assert_eq!(names.len(), listed);
    }

    #[test]
    fn leagues_rules_reachable() {
        for name in league_names() {
            let rules = league_rules(name).expect("league unreachable by its own name");
            assert!(!rules.is_empty(), "{name} scores nothing");
        }
    }
}
