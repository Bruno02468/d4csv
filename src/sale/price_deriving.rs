//! Here we implement the first step of deriving ticket amounts from batch
//! prices and sale values.

use std::collections::HashSet;
use std::ops::Range;
use itertools::Itertools;
use crate::context::SalesContext;
use crate::ticket::batch::Batch;

pub(crate) type BatchAmount = (Batch, usize);

/// Total price of a BatchAmount. Just sugar.
fn ba_price(ba: &BatchAmount) -> usize {
  return ba.0.price * ba.1;
}

/// Iterator for BatchAmounts within some amount range.
fn ba_iter(
  batch: &Batch,
  range: &Range<usize>
) -> impl Iterator<Item = BatchAmount> + '_ {
  return range.into_iter().map(|n| (*batch, n));
}

/// A match for a price and some kind of sale.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PricingMatch {
  /// A multiple of a batch.
  Multiple(BatchAmount),
  /// Some promos and the following batch.
  PromoCombo(BatchAmount, BatchAmount),
  /// Turn-of-batch purchase. Those are hairy.
  TurnOfBatch(BatchAmount, BatchAmount)
}

impl PricingMatch {
  /// Returns the sum price of this match.
  pub(crate) fn price(&self) -> usize {
    return match self {
      PricingMatch::Multiple(ba) => ba_price(ba),
      PricingMatch::PromoCombo(pba, ba) => ba_price(pba) + ba_price(ba),
      PricingMatch::TurnOfBatch(ba1, ba2) => ba_price(ba1) + ba_price(ba2),
    }    
  }
  /// Returns the number of tickets in this match.
  pub(crate) fn tickets(&self) -> usize {
    return match self {
      PricingMatch::Multiple(ba) => ba.1,
      PricingMatch::PromoCombo(pba, ba) => pba.1 + ba.1,
      PricingMatch::TurnOfBatch(ba1, ba2) => ba1.1 + ba2.1,
    }
  }
  /// Returns all pricing matches for a certain price in cents.
  pub(crate) fn all_priced(price: usize, ctx: &SalesContext) -> Vec<Self> {
    let mut v: Vec<Self> = Vec::new();
    // min price
    let mp: usize;
    match ctx.batches.iter().map(|(_, p)| *p).min() {
      Some(k) => mp = k,
      // no minimum, return nothin'
      None => return v,
    }
    // worst-case amount
    let w = price / mp + 1;
    // first, all matches
    bp2iter(&ctx.batches);
    todo!();
  }
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
  /// No matches? Goddamnit.
  NoMatch
}


