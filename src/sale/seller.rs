//! Seller abstractions: online or... somewhere.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Seller {
  /// Online sale, with some integer fraction as the fee.
  Online((usize, usize)),
  /// Face-to-face sale, by someone.
  Offline(String)
}

impl Seller {
  /// Apply the online fee if online.
  pub(crate) fn apply_fee(&self, price: usize) -> usize {
    if let Self::Online((k, d)) = self {
      return price * k / d;
    } else {
      return price;
    }
  }
}
