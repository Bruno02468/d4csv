//! Useful information to report after looking at the sales list.

pub(crate) mod sfields;

use std::collections::HashMap;
use yew::{Component, Properties, html, html_nested};
use crate::report::sfields::SFIELDS;
use crate::sale::plus::SalesPlus;

/// A report field made out to be a single string.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct StringField(String, String);

/// A function that computes a string field from sales data.
pub(crate) type FieldFn = fn(&SalesPlus) -> StringField;

/// A report field that's a string-string table.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct TableField(String, HashMap<String, String>);

/// A function that computes a table field from sales data.
pub(crate) type TableFn = fn(&SalesPlus) -> TableField;

/// A report skeleton, made out of field functions.
pub(crate) struct ReportTemplate {
  /// All string field functions.
  sfields: Vec<FieldFn>,
  /// All table field functions.
  tfields: Vec<TableFn>
}

impl Default for ReportTemplate {
  fn default() -> Self {
    return Self {
      sfields: SFIELDS.to_vec(),
      tfields: Vec::new()
    };
  }
}

impl ReportTemplate {
  pub(crate) fn compute(&self, data: &SalesPlus) -> Report {
    return Report {
      sfields: self.sfields.iter().map(|f| f(data)).collect(),
      tfields: self.tfields.iter().map(|f| f(data)).collect(),
    }
  }
}

/// A corresponding report, calculated from the skeleton and sales data.
#[derive(Clone, Properties, PartialEq, Eq)]
pub(crate) struct Report {
  /// All string fields.
  sfields: Vec<StringField>,
  /// All table fields.
  tfields: Vec<TableField>
}

/// A component that displays a report.
pub(crate) struct ReportDisplay;

impl Component for ReportDisplay {
  type Message = ();
  type Properties = Report;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    return Self;
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    return html! {
      <div class="report">
        <ul class="sfields">
          {
            for ctx.props().sfields.iter().map(|sf| {
              html_nested! {
                <li>
                  <b>{ &sf.0 }</b>{ ":" } { &sf.1 }
                </li>
              }
            })
          }
        </ul>
        <ul class="tfields">
          {
            for ctx.props().tfields.iter().map(|tf| {
              html_nested! {
                <li>
                  <b>{ &tf.0 }</b>{ ":" }
                  <ul class="tfield-vals">
                    {
                      for tf.1.iter().map(|(k, v)| {
                        html_nested! {
                          <li>
                            <b>{ &k }</b>{ ":" } { &v }
                          </li>
                        }
                      })
                    }
                  </ul>
                </li>
              }
            })
          }
        </ul>
      </div>
    }
  }
}
