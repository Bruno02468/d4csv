//! Structs for storing sale data and extra context and derived info.

use crate::context::SalesContext;
use crate::sale::Sale;
use crate::sale::price_deriving::{PricingCandidate, PricingMatch, PricingCandidateCache};

/// Sale plus inferred data.
#[derive(Clone, Debug)]
pub(crate) struct SalePlus {
  /// The sale itself.
  pub(crate) sale: Sale,
  /// The price candidate inferred from price alone. Can be calculated
  /// immediately.
  pub(crate) pricecand: PricingCandidate,
  /// The price match resolved from adjacencies and extra info.
  pub(crate) pricematch: Option<PricingMatch>
}

impl AsRef<Sale> for SalePlus {
  fn as_ref(&self) -> &Sale {
    return &self.sale;
  }
}

impl From<(Sale, PricingCandidate)> for SalePlus {
  fn from((s, cnd): (Sale, PricingCandidate)) -> Self {
    return Self {
      sale: s,
      pricecand: cnd.clone(),
      pricematch: match cnd {
        PricingCandidate::Precise(pm) => Some(pm),
        PricingCandidate::Ambiguous(_) => None,
        PricingCandidate::NoMatch => None,
      }
    };
  }
}

/// Stores loads of sales, and resolves pricing ambiguities.
#[derive(Clone, Debug)]
pub(crate) struct SalesPlus {
  /// A vec full of SalePlus.
  pub(crate) sales: Vec<SalePlus>,
  /// A copy of the context.
  pub(crate) context: SalesContext
}

impl AsRef<Vec<SalePlus>> for SalesPlus {
  fn as_ref(&self) -> &Vec<SalePlus> {
    return &self.sales;
  }
}

impl SalesPlus {
  /// Convert a vector of sales into a SalesPlus, using a caching dude to save
  /// time on pricing inference.
  pub(crate) fn from_sales<T>(
    iter: T, ctx: SalesContext
  ) -> Self where T: Iterator<Item = Sale> {
    let mut sp = Self {
      sales: Vec::new(),
      context: ctx.clone()
    };
    let mut dude = PricingCandidateCache::from(ctx);
    for sale in iter {
      let pc = dude.from_price(sale.real_price());
      sp.sales.push(SalePlus::from((sale, pc)))
    }
    return sp;
  }

  /// Returns an iterator over all sales with ambiguous pricing conclusions.
  pub(crate) fn ambiguous(&self) -> impl Iterator<Item = &SalePlus> {
    return self.sales.iter()
      .filter(|sp| match sp.pricecand {
        PricingCandidate::Ambiguous(_) => true,
        _ => false,
      });
  }

  /// Returns an iterator over all sales with no pricing conclusions.
  pub(crate) fn villains(&self) -> impl Iterator<Item = &SalePlus> {
    return self.sales.iter()
      .filter(|sp| match sp.pricecand {
        PricingCandidate::NoMatch => true,
        _ => false,
      });
  }
}
