//! Here we implement the first step of deriving ticket amounts from batch
//! prices and sale values.

use std::collections::{HashSet, HashMap};
use std::ops::Range;
use itertools::Itertools;
use crate::context::SalesContext;
use crate::ticket::batch::{Batch, bp2iter};
use crate::ticket::batchnum::BatchNum;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BatchAmount(Batch, usize);

impl From<(Batch, usize)> for BatchAmount {
  fn from((b, a): (Batch, usize)) -> Self {
    return Self(b, a);
  }
}

/// Total price of a BatchAmount. Just sugar.
fn ba_price(ba: &BatchAmount) -> usize {
  return ba.0.price * ba.1;
}

/// Iterator for BatchAmounts within some amount range.
fn ba_iter(
  batch: Batch,
  range: Range<usize>
) -> impl Iterator<Item = BatchAmount> + Clone {
  return range.into_iter().map(move |n| (batch, n).into());
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
    // worst-case amount and range
    let w = price / mp + 1;
    let wr: Range<usize> = Range { start: 1, end: w };
    // this returns an iterator with all batches' ranges
    let allba: Vec<BatchAmount> = bp2iter(&ctx.batches)
      .map(|b| ba_iter(b, wr.clone()))
      .flatten()
      .collect();
    // first, all multiple matches
    allba.iter()
      .filter_map(|ba| {
        if ba_price(&ba) == price {
          return Some(Self::Multiple(*ba));
        } else {
          return None;
        }
      }).for_each(|pm| v.push(pm));
    // next, all promo combos
    let opt_promo = bp2iter(&ctx.batches)
      .filter(|ba| ba.num == BatchNum::Promo)
      .nth(0);
    let pr: Range<usize> = Range {
      start: 1,
      end: ctx.promo_limit.unwrap_or(w)
    };
    // all non-promo
    let bi = || allba.iter()
      .filter(|ba| ba.0.num > BatchNum::Promo);
    if let Some(promo) = opt_promo {
      // all promo amounts
      let pi = ba_iter(promo, pr);
      // all combinations
      pi.cartesian_product(bi())
        .filter_map(|(pba, ba)| {
          if ba.0.num.inum() != 1 { return None; }
          let cand = Self::PromoCombo(pba, *ba);
          if cand.price() == price {
            return Some(cand);
          } else {
            return None;
          }
        }).for_each(|pm| v.push(pm));
    }
    // finally, all non-promo adjacent combos
    bp2iter(&ctx.batches)
      .cartesian_product(bp2iter(&ctx.batches))
      .filter_map(|(b1, b2)| {
        if (b2.num.inum() as isize) - (b1.num.inum() as isize) == 1 {
          return Some(
            ba_iter(b1, wr.clone())
              .cartesian_product(ba_iter(b2, wr.clone()))
          );
        } else {
          return None;
        }
      }).flatten()
      .for_each(|(ba1, ba2)| {
        let cand = Self::TurnOfBatch(ba1, ba2);
        if cand.price() == price {
          v.push(cand);
        }
      });
    return v;
  }
}

/// All possible matches for a given price.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PricingCandidate {
  /// Only one match, nice!
  Precise(PricingMatch),
  /// Kinda ambiguous!
  Ambiguous(HashSet<PricingMatch>),
  /// No matches? Goddamnit.
  NoMatch
}

impl FromIterator<PricingMatch> for PricingCandidate {
  fn from_iter<T: IntoIterator<Item = PricingMatch>>(iter: T) -> Self {
    let mut hs: HashSet<PricingMatch> = iter.into_iter().collect();
    return match hs.len() {
      0 => Self::NoMatch,
      1 => Self::Precise(hs.drain().nth(0).unwrap()),
      _ => Self::Ambiguous(hs)
    };
  }
}

impl PricingCandidate {
  pub(crate) fn from_price(price: usize, ctx: &SalesContext) -> Self {
    return PricingCandidate::from_iter(PricingMatch::all_priced(price, ctx));
  }
}

/// A caching pricing generator so we avoid re-computing all candidates for a
/// given price more than once.
#[derive(Clone, Debug)]
pub(crate) struct PricingCandidateCache {
  /// Inner storage of pricing candidates for a given
  store: HashMap<usize, PricingCandidate>,
  /// A copy of the sales' context.
  ctx: SalesContext
}

impl From<SalesContext> for PricingCandidateCache {
  fn from(ctx: SalesContext) -> Self {
    return Self {
      store: HashMap::new(),
      ctx
    };
  }
}

impl PricingCandidateCache {
  /// Computes the pricing candidates if absent
  pub(crate) fn from_price(&mut self, price: usize) -> PricingCandidate {
    if let Some(pc) = self.store.get(&price) {
      return pc.clone();
    } else {
      let pc = PricingCandidate::from_price(price, &self.ctx);
      self.store.insert(price, pc.clone());
      return pc;
    }
  }
}

