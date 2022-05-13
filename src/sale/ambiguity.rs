//! Implements ways to resolve ambiguities in pricing candidates.

use std::collections::HashSet;
use std::fmt::Display;
use crate::sale::kind::Seller;
use crate::sale::plus::SalesPlus;
use crate::sale::price_deriving::{PricingCandidate, PricingMatch};
use crate::ticket::batch::Batch;

/// A function that resolves ambiguities.
pub(crate) type AmbiguitySolverFn = fn(&mut SalesPlus) -> usize;

/// Defines a way to resolve ambiguities.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AmbiguitySolver {
  /// Does nothing.
  DoNothing,
  /// Resolves ambiguities by looking behind in time.
  TemporalLookbehind,
  /// Resolves ambiguities by looking behind in time, but accounting for
  /// different sellers (batch changes can be asynchronous.)
  SellerLookBehind
}

impl Default for AmbiguitySolver {
  /// Returns the "best" ambiguity solver currently available.
  fn default() -> Self {
    return Self::SellerLookBehind;
  }
}

impl Display for AmbiguitySolver {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", match self {
      AmbiguitySolver::DoNothing => "nenhum",
      AmbiguitySolver::TemporalLookbehind => "olhar anteriores",
      AmbiguitySolver::SellerLookBehind => "olhar anteriores do mesmo ponto",
    });
  }
}

/// Implementation of the DoNothing solver.
fn do_nothing(_sp: &mut SalesPlus) -> usize {
  return 0;
}

/// Implementation of the TemporalLookbehind solver.
fn temporal_lookbehind(sp: &mut SalesPlus) -> usize {
  let mut batch: Option<Batch> = None;
  let mut res: usize = 0;
  for sp in sp.sales.iter_mut() {
    if let Some(pm) = sp.pricematch {
      // we're now sure of the batch
      batch = Some(pm.batch_after());
    } else if let Some(b) = batch {
      // we can use the known batch to solve an ambiguity
      if let PricingCandidate::Ambiguous(hs) = &sp.pricecand {
        let mut compat: HashSet<PricingMatch> = hs.clone().into_iter()
          .filter(|pc| pc.batch_after() == b)
          .collect();
        match compat.len() {
          0 => continue,
          1 => {
            sp.resolve(compat.drain().nth(0).unwrap());
            res += 1;
          },
          _ => sp.pricecand = PricingCandidate::Ambiguous(compat)
        }
      }
    }
  }
  return res;
}

/// Implementation of the SellerLookBehind solver.
fn seller_lookbehind(sp: &mut SalesPlus) -> usize {
  let mut total: usize = 0;
  let sellers: HashSet<Seller> = sp.sales.iter()
    .filter_map(|s| { s.sale.seller() })
    .collect();
  for seller in sellers {
    let theirs = sp.sales.iter_mut().filter(|s| {
      if let Some(slr) = &s.sale.seller() {
        return slr == &seller;
      }
      return false;
    });
    let mut batches: Option<HashSet<Batch>> = None;
    let mut accbatches: HashSet<Batch> = HashSet::new();
    for mut sale in theirs {
      if let Some(pm) = sale.pricematch {
        accbatches.extend(pm.batches());
        batches = Some(pm.batches());
      } else if let Some(ref bhs) = batches {
        if let PricingCandidate::Ambiguous(cands) = sale.pricecand.clone() {
          // remove candidates without batches in common to the above
          let newcands: HashSet<&PricingMatch> = cands.iter()
            .filter(|pcm| {
              !pcm.batches().is_disjoint(&accbatches)
            }).collect();
          if newcands.len() > 0 {
            // ambiguity diminished (maybe)
            // log::info!("{:#?} virou {:#?}", cands, newcands);
            if newcands.len() == 1 {
              // ambiguity resolved!
              (*sale).pricematch = Some(
                *newcands.iter().nth(0).unwrap().clone()
              );
              total += 1;
            } else {
              // try for no new batches.
              let nonews: HashSet<&&PricingMatch> = newcands.iter()
                .filter(|pm| pm.batches().is_subset(&bhs))
                .collect();
              if nonews.len() == 1 {
                // only one with no new batches. nice!
                (*sale).pricematch = Some(
                  *newcands.iter().nth(0).unwrap().clone()
                );
                total += 1;
              }
            }
            // write the new candidates anyway
            (*sale).pricecand = PricingCandidate::from_iter(
              newcands.into_iter()
                .map(|s| s.clone())
            );
          }
        }
      }
    }
  }
  return total;
}

impl TryFrom<&str> for AmbiguitySolver {
  type Error = ();
  fn try_from(s: &str) -> Result<Self, Self::Error> {
    return match s.to_lowercase().as_str() {
      "nothing" => Ok(AmbiguitySolver::DoNothing),
      "temporal" => Ok(AmbiguitySolver::TemporalLookbehind),
      "seller" => Ok(AmbiguitySolver::SellerLookBehind),
      _ => Err(())
    };
  }
}

impl AmbiguitySolver {
  pub(crate) fn name(&self) -> &'static str {
    return match self {
      AmbiguitySolver::DoNothing => "nothing",
      AmbiguitySolver::TemporalLookbehind => "temporal",
      AmbiguitySolver::SellerLookBehind => "seller",
    };
  }

  pub(crate) fn available() -> impl Iterator<Item = Self> {
    return [
      Self::DoNothing,
      Self::TemporalLookbehind,
      Self::SellerLookBehind
    ].into_iter();
  }
}

impl From<AmbiguitySolver> for AmbiguitySolverFn {
  fn from(solv: AmbiguitySolver) -> Self {
    return match solv {
      AmbiguitySolver::DoNothing => do_nothing,
      AmbiguitySolver::TemporalLookbehind => temporal_lookbehind,
      AmbiguitySolver::SellerLookBehind => seller_lookbehind,
    };
  }
}
