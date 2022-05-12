//! Here we implement the first step of deriving ticket amounts from batch
//! prices and sale values.

use std::collections::HashSet;

use crate::ticket::batch::Batch;

/// A match for a price and some kind of sale.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PricingMatch {
  /// A multiple of a batch.
  Multiple(Batch, usize),
  /// Some promos and the following batch.
  PromoCombo(usize, (Batch, usize))
}

/// All possible matches for a given price.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PricingCandidate {
  /// Only one match, nice!
  Precise(PricingMatch),
  /// Kinda ambiguous!
  Ambiguous(HashSet<PricingMatch>),
  /// Ambiguous, but resolved from adjacencies.
  Resolved(PricingMatch),
  /// No matches?
  NoMatch
}

