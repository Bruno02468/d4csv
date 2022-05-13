//! Basic table fields for the report.

use std::collections::HashMap;
use std::fmt::Display;
use core::hash::Hash;
use crate::report::{TableField, TableFn};
use crate::sale::plus::SalesPlus;

/// Quick sugar for making string fields.
fn tf<K, V>(
  name: &str,
  hm: HashMap<K, V>
) -> TableField where K: Display + Hash, V: Display {
  return TableField(
    name.to_owned(),
    hm.into_iter()
      .map(|(k, v)| (k.to_string(), v.to_string()))
      .collect()
  );
}

/// All the functions below.
pub(crate) static TFIELDS: &[TableFn] = &[
  sales_per_seller
];

/// Offline sales per seller.
pub(crate) fn sales_per_seller(sp: &SalesPlus) -> TableField {
  let mut hm: HashMap<String, usize> = HashMap::new();
  sp.oks()
    .for_each(|s| {
      let t = s.pricematch.unwrap().tickets();
      if let Some(sn) = &s.sale.seller_name {
        if let Some(r) = hm.get_mut(sn) {
          *r += 1;
        } else {
          hm.insert(sn.clone(), t);
        }
      }
      return ();
    });
  return tf(
    "Ingressos físicos por ponto de venda",
    hm
  );
}
