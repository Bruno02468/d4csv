//! Implements ways to resolve ambiguities in pricing candidates.

use std::collections::HashSet;
use std::fmt::Display;
use crate::sale::kind::SaleKind;
use crate::sale::plus::SalesPlus;
use crate::sale::price_deriving::{PricingCandidate, PricingMatch};
use crate::ticket::batch::Batch;
use crate::ticket::batchnum::BatchNum;

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
  log::info!("comb_simple did {}", res);
  return res;
}

/// Implementation of the SellerLookBehind solver.
fn seller_lookbehind(sp: &mut SalesPlus) -> usize {
  let sellers: HashSet<String> = sp.sales.iter()
    .filter_map(|s| {
      if s.sale.sale_kind == SaleKind::Offline {
        if let Some(ref sn) = s.sale.seller_name {
          return Some(sn.clone());
        }
      }
      return None;
    }).collect();
  for seller in sellers {
    let theirs = sp.sales.iter().filter(|s| {
      if let Some(sn) = &s.sale.seller_name {
        return sn == &seller;
      }
      return false;
    });
    let batch: Option<BatchNum> = None;
    for sale in theirs {
      todo!();
    }
  }
  return 0;
}

impl From<AmbiguitySolver> for AmbiguitySolverFn {
  fn from(_: AmbiguitySolver) -> Self {
    todo!()
  }
}
