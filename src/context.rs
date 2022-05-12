//! Sale context that comes from outside the CSV.

use crate::ticket::batch::BatchPrices;

/// The context needed to derive ticket information from the CSV.
#[derive(Clone, Debug)]
pub(crate) struct SalesContext {
  /// Online fee.
  pub(crate) online_fee: (usize, usize),
  /// Batch prices.
  pub(crate) batches: BatchPrices,
  /// Promo batch limit per person.
  pub(crate) promo_limit: Option<usize>
}
