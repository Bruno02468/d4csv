//! Structs for storing sale data and extra context and derived info.

use std::collections::HashSet;
use std::fmt::Display;
use itertools::Itertools;
use crate::context::SalesContext;
use crate::sale::Sale;
use crate::sale::price_deriving::{PricingCandidate, PricingMatch, PricingCandidateCache};
use crate::ticket::batch::Batch;

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

impl SalePlus {
  /// Resolve this sale's pricing inference.
  pub(crate) fn resolve(&mut self, pm: PricingMatch) {
    self.pricematch = Some(pm);
  }

  /// Generate a line for the "better CSV".
  pub(crate) fn gen_better_csv_line(&self) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    let p = |vr: &mut Vec<String>, s: &dyn Display| {
      vr.push(s.to_string());
    };
    let ps = |vr: &mut Vec<String>, s: Option<&String>| {
      vr.push(s.unwrap_or(&"".into()).to_owned());
    };
    // add fields one by one
    p(&mut v, &self.sale.when);
    ps(&mut v, self.sale.buyer_email.as_ref());
    ps(&mut v, self.sale.buyer_username.as_ref());
    p(&mut v, &(self.sale.value as f64 / 100.0));
    p(&mut v, &self.sale.sale_kind);
    ps(&mut v, self.sale.seller_name.as_ref());
    ps(&mut v, self.sale.seller_id.as_ref());
    ps(&mut v, self.sale.seller_email.as_ref());
    p(&mut v, &self.sale.token);
    p(&mut v, &self.sale.sale_id);
    ps(&mut v, self.sale.card_name.as_ref());
    ps(&mut v, self.sale.card_pfx.as_ref());
    ps(&mut v, self.sale.card_sfx.as_ref());
    // now the extra fields!
    // is this resolved?
    p(&mut v, &{
      if self.pricematch.is_some() {
        "sim"
      } else {
        "não"
      }
    });
    // if resolved, tell ya the batches
    p(&mut v, &{
      if let Some(pm) = self.pricematch {
        pm.to_string()
      } else {
        match &self.pricecand {
          PricingCandidate::Precise(pm) => pm.to_string(),
          PricingCandidate::Ambiguous(hs) => {
            hs.iter()
              .map(|g| g.to_string())
              .join("  ou  ")
          },
          PricingCandidate::NoMatch => "TRAGÉDIA".to_owned(),
        }
      }
    });
    return v;
  }

  /// Returns the header for the better CSV.
  pub(crate) fn better_csv_header() -> Vec<String> {
    return [
      "DataCompra",
      "EmailUsuarioAssociado",
      "NomeUsuarioAssociado",
      "ValorDaCompra",
      "Status",
      "NomeVendedor",
      "IDVendedor",
      "EmailVendedor",
      "Token",
      "ID",
      "NomeCartao",
      "PrimDigitosCartao",
      "UltDigitosCartao",
      "Resolvido?",
      "Decodificação de preço"
    ].iter().map(|s| s.to_string()).collect();
  }
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

  /// Returns an iterator over all sales with a precise pricing conclusion.
  pub(crate) fn oks(&self) -> impl Iterator<Item = &SalePlus> {
    return self.sales.iter()
      .filter(|s| s.pricematch.is_some());
  }

  /// Generates the "better" CSV dude.
  pub(crate) fn gen_csv(&self) -> Vec<Vec<String>> {
    return self.sales.iter()
      .map(|s| s.gen_better_csv_line())
      .collect();
  }

  /// Basic comb: look at adjacencies downwards. Returns the number of
  /// resolutions (i.e. effective changes).
  pub(crate) fn comb_simple(&mut self) -> usize {
    let mut batch: Option<Batch> = None;
    let mut res: usize = 0;
    for sp in self.sales.iter_mut() {
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
}
