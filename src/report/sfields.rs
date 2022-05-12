//! Basic string fields for the report.

use std::fmt::Display;

use crate::report::{StringField, FieldFn};
use crate::sale::kind::SaleKind;
use crate::sale::plus::SalesPlus;

/// Quick sugar for making string fields.
fn sf<T>(name: &str, value: T) -> StringField where T: Display {
  return StringField(name.to_owned(), format!("{}", value));
}

/// All the functions below.
pub(crate) static SFIELDS: &[FieldFn] = &[
  total_sales,
  total_ok,
  total_tickets,
  online_tickets,
  ambiguous_sales,
  evil_sales
];

/// Total sales in list.
fn total_sales(sp: &SalesPlus) -> StringField {
  return sf("Total de vendas", sp.sales.len());
}

/// Total sales for which we found a pricing match.
fn total_ok(sp: &SalesPlus) -> StringField {
  let nok = sp.oks().count();
  let perc = ((nok as f64) / (sp.sales.len() as f64) * 100.0).round() as usize;
  return sf(
    "Vendas ok",
    format!("{} ({}%)", nok, perc)
  );
}

/// Total tickets sold.
fn total_tickets(sp: &SalesPlus) -> StringField {
  return sf(
    "Total de ingressos: ",
    sp.oks()
      .map(|s| s.pricematch.unwrap().tickets())
      .sum::<usize>()
  );
}

/// Total tickets sold online.
fn online_tickets(sp: &SalesPlus) -> StringField {
  return sf(
    "Ingressos online: ",
    sp.oks()
      .filter_map(|s| {
        if let SaleKind::Online((_, _)) = &s.sale.sale_kind {
          return Some(s.pricematch.unwrap().tickets());
        }
        return None;
      }).sum::<usize>()
  );
}

/// Number of ambiguous sales.
fn ambiguous_sales(sp: &SalesPlus) -> StringField {
  return sf("Vendas (inicialmente) ambíguas", sp.ambiguous().count());
}

/// Number of unsolvable sales.
fn evil_sales(sp: &SalesPlus) -> StringField {
  return sf("Vendas sem nenhuma solução", sp.villains().count());
}

