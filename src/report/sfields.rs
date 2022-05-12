//! Basic string fields for the report.

use std::fmt::Display;

use crate::report::{StringField, FieldFn};
use crate::sale::plus::SalesPlus;

/// Quick sugar for making string fields.
fn sf<T>(name: &str, value: T) -> StringField where T: Display {
  return StringField(name.to_owned(), format!("{}", value));
}

/// All the functions below.
pub(crate) static SFIELDS: &[FieldFn] = &[
  total_sales,
  ambiguous_sales,
  evil_sales
];

/// Total sales in list.
fn total_sales(sp: &SalesPlus) -> StringField {
  return sf("Total de vendas", sp.sales.len());
}

/// Number of ambiguous sales.
fn ambiguous_sales(sp: &SalesPlus) -> StringField {
  return sf("Vendas ambíguas", sp.ambiguous().count());
}

/// Number of unsolvable sales.
fn evil_sales(sp: &SalesPlus) -> StringField {
  return sf("Vendas sem solução", sp.villains().count());
}
