//! Ticket batch numbering.

use std::cmp::Ordering;
use std::fmt::Display;

/// The number of a single ticket batch.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum BatchNum {
  /// Promo batch
  Promo,
  /// Numbered batches (start at 1!).
  Numbered(usize)
}

impl BatchNum {
  /// Implicit batch number -- promo is zero.
  pub(crate) fn inum(&self) -> usize {
    return match self {
      BatchNum::Promo => 0,
      BatchNum::Numbered(n) => *n,
    }
  }
}

impl PartialOrd for BatchNum {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    return match (self, other) {
      (BatchNum::Promo, BatchNum::Promo) => Some(Ordering::Equal),
      (BatchNum::Promo, BatchNum::Numbered(_)) => Some(Ordering::Less),
      (BatchNum::Numbered(_), BatchNum::Promo) => Some(Ordering::Greater),
      (BatchNum::Numbered(a), BatchNum::Numbered(b)) => a.partial_cmp(b),
    };
  }
}

impl Ord for BatchNum {
  fn cmp(&self, other: &Self) -> Ordering {
    return self.partial_cmp(other).unwrap();
  }
}

impl Display for BatchNum {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      Self::Promo => write!(f, "lote promocional"),
      Self::Numbered(n) => write!(f, "{}º lote", n)
    };
  }
}

impl From<usize> for BatchNum {
  fn from(n: usize) -> Self {
    return match n {
      0 => Self::Promo,
      n => Self::Numbered(n)
    }
  }
}
