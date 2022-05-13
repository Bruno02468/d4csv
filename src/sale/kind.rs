//! Seller abstractions: online or... somewhere.

use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SaleKind {
  /// Online sale, with some integer fraction as the fee.
  Online((usize, usize)),
  /// Face-to-face sale, by someone.
  Offline
}

impl SaleKind {
  /// Apply the online fee if online.
  pub(crate) fn apply_fee(&self, price: usize) -> usize {
    if let Self::Online((k, d)) = self {
      return price * k / d;
    } else {
      return price;
    }
  }

  /// Undo the online fee if online.
  pub(crate) fn undo_fee(&self, price: usize) -> usize {
    if let Self::Online((k, d)) = self {
      return price * d / k;
    } else {
      return price;
    }
  }
}

impl Display for SaleKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "Paga {}", match self {
      SaleKind::Online(_) => "Online",
      SaleKind::Offline => "Físico",
    });
  }
}

/// An alternative version of SaleKind -- more suited to store actual seller
/// information.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Seller {
  /// Onlne sale.
  Online,
  /// Offline sale, containing seller name.
  Offline(String)
}
