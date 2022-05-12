//! Abstractions for ticket batches.

use std::collections::HashMap;
use crate::ticket::batchnum::BatchNum;

/// A single ticket batch.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Batch {
  /// Batch number.
  pub(crate) num: BatchNum,
  /// Price in cents.
  pub(crate) price: usize
}

/// A list of batch prices.
pub(crate) type BatchPrices = HashMap<BatchNum, usize>;

/// Generates a BatchPrices from a list of prices (in cents).
pub(crate) fn iter2bp<T: IntoIterator<Item = usize>>(iter: T) -> BatchPrices {
  let mut bp = BatchPrices::new();
  for (i, n) in iter.into_iter().enumerate() {
    bp.insert(i.into(), n);
  }
  return bp;
}


