// SPDX-License-Identifier: GPL-2.0-or-later WITH Classpath-exception-2.0

//! Rules: what a league scores, and over which shape.
//!
//! One file per league. A league's numbers belong in that file and nowhere else.

use std::fmt::Debug;

use super::shapes::{ShapeBuilder, ShapeKind};
#[cfg(feature = "geojson")]
use crate::ScoreError;

pub mod cfd;
pub mod misc;
pub mod xcontest;

#[cfg(feature = "crazy-test")]
pub mod crazy;

pub use super::shapes::balanced::BalancedCircuit;
pub use super::shapes::closed::ClosedCircuit;
pub use super::shapes::polyline::OpenPolyline;

/// A gap limit, as a rulebook states it.
#[derive(Debug, Clone, Copy)]
pub enum Limit {
    /// No limit: nothing is free, or nothing closes once charged.
    None,
    /// An absolute distance, in meters.
    Fixed(f64),
    /// A share of the distance.
    Ratio(f64),
}

impl Limit {
    /// What the limit comes to against `distance`, in meters.
    fn of(&self, distance: f64) -> f64 {
        match self {
            Limit::None => 0.0,
            Limit::Fixed(meters) => *meters,
            Limit::Ratio(share) => share * distance,
        }
    }
}

/// A closing law: what a gap costs, and how large it may get.
///
/// The two limits are independent, and either can be absolute or relative. `penalty` and `limit`
/// derive from both, which is what keeps them in step. A law wanting the greater of an absolute and
/// a share *within* one of them needs a variant [`Limit`] does not have; none does.
#[derive(Debug, Clone, Copy)]
pub struct Closing {
    /// Gap allowed free of charge.
    free: Limit,
    /// Largest gap that closes once charged in full.
    charged: Limit,
}

impl Closing {
    pub const fn new(free: Limit, charged: Limit) -> Self {
        Self { free, charged }
    }

    /// Largest gap this law accepts, in meters, against the raw distance.
    ///
    /// A free gap closes whatever `charged` says, hence the larger of the two.
    pub fn limit(&self, distance: f64) -> f64 {
        self.free.of(distance).max(self.charged.of(distance))
    }

    /// Charge for the gap. INFINITY for a gap this law does not accept.
    pub fn penalty(&self, distance: f64, gap: f64) -> f64 {
        if gap <= self.free.of(distance) {
            0.0
        } else if gap <= self.charged.of(distance) {
            gap
        } else {
            f64::INFINITY
        }
    }
}

/// What a rule made of a geometry, in meters, unrounded.
#[derive(Debug)]
pub struct Scored {
    /// Distance net of the penalty.
    pub distance: f64,
    pub penalty: f64,
    /// 0 when the minimum is not met: there is a path, it is worth nothing.
    pub score: f64,
    pub multiplier: f64,
    /// Largest gap this result's description would still hold at, on the raw distance; 0 when
    /// there is no closing leg.
    pub threshold: f64,
    pub description: &'static str,
    pub league: &'static str,
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

    /// Identity of the scoring league
    fn league() -> &'static str {
        Self::NAME
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

/// One way a rule prices a geometry: what the gap costs, if anything, and what the price is worth.
#[derive(Debug, Clone, Copy)]
pub struct Variant {
    pub name: &'static str,
    pub multiplier: f64,
    pub kind: VariantKind,
}

#[derive(Debug, Clone, Copy)]
pub enum VariantKind {
    /// A shape with no closing leg.
    Open,
    /// A circuit closing under this law, which also charges the gap.
    Closing(Closing),
}

impl VariantKind {
    /// Whether this variant's closing law still holds at this gap, against the raw distance. An
    /// open shape holds only with nothing to close.
    fn accepts(&self, distance: f64, gap: f64) -> bool {
        match self {
            VariantKind::Open => gap == 0.0,
            VariantKind::Closing(closing) => gap <= closing.limit(distance),
        }
    }

    /// Charge for the gap under this variant. INFINITY once the variant no longer accepts it.
    fn penalty(&self, distance: f64, gap: f64) -> f64 {
        match self {
            VariantKind::Open => {
                if gap == 0.0 {
                    0.0
                } else {
                    f64::INFINITY
                }
            }
            VariantKind::Closing(closing) => closing.penalty(distance, gap),
        }
    }

    /// Largest gap this variant accepts, in meters, against the raw distance; 0 with no closing leg.
    fn limit(&self, distance: f64) -> f64 {
        match self {
            VariantKind::Open => 0.0,
            VariantKind::Closing(closing) => closing.limit(distance),
        }
    }
}

impl Variant {
    fn score(&self, distance: f64, gap: f64) -> f64 {
        (distance - self.kind.penalty(distance, gap)).max(0.0) * self.multiplier
    }
}

trait VariantList {
    fn pick(&self, distance: f64, gap: f64) -> &Variant;
}

impl VariantList for [Variant] {
    /// The variant a rule prices a geometry at: the first, listed strictest first, whose closing
    /// law holds at this gap — the loosest one if none do.
    fn pick(&self, distance: f64, gap: f64) -> &Variant {
        self.iter()
            .find(|v| v.kind.accepts(distance, gap))
            .unwrap_or_else(|| {
                self.last().expect("a rule always has at least one variant")
            })
    }
}

/// A rule pricing a geometry through one or more [`Variant`]s, listed from the strictest (tightest
/// gap allowance, best rate) to the loosest.
pub trait RuleDescription: Debug {
    type League: League;

    fn variants(&self) -> &'static [Variant];

    fn minimum(&self) -> f64 {
        Self::League::minimum()
    }
    fn league(&self) -> &'static str {
        Self::League::league()
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
        self.variants().pick(distance, gap).score(distance, gap)
    }
}

impl<T: RuleDescription> RuleReport for T {
    fn scored(&self, distance: f64, gap: f64) -> Scored {
        let variant = self.variants().pick(distance, gap);
        let penalty = variant.kind.penalty(distance, gap);
        let score = variant.score(distance, gap);

        Scored {
            distance: (distance - penalty).max(0.0),
            penalty,
            score: if score >= self.minimum() { score } else { 0.0 },
            multiplier: variant.multiplier,
            threshold: variant.kind.limit(distance),
            description: variant.name,
            league: self.league(),
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

/// Refuses a name the registry does not hold, for an entry point that takes a league but may not
/// reach [`Scorer::solve`].
///
/// [`Scorer::solve`]: crate::Scorer::solve
#[cfg(feature = "geojson")]
pub fn known_league(name: &str) -> Result<(), ScoreError> {
    league_rules(name)
        .map(|_| ())
        .ok_or(ScoreError::UnknownLeague)
}

/// Every league name [`Log::score`] and [`Scorer::solve`] accept.
///
/// [`Log::score`]: crate::Log::score
/// [`Scorer::solve`]: crate::Scorer::solve
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
