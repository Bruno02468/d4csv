//! Useful information to report after looking at the sales list.

pub(crate) mod sfields;
pub(crate) mod tfields;

use std::collections::HashMap;
use yew::{Component, Properties, html, html_nested};
use crate::report::sfields::SFIELDS;
use crate::report::tfields::TFIELDS;
use crate::sale::plus::{SalesPlus, SalePlus};

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
      tfields: TFIELDS.to_vec()
    };
  }
}

impl ReportTemplate {
  /// Computes the report from sales data.
  pub(crate) fn compute(&self, data: &SalesPlus) -> Report {
    return Report {
      sfields: self.sfields.iter().map(|f| f(data)).collect(),
      tfields: self.tfields.iter().map(|f| f(data)).collect(),
      better_csv: data.gen_csv()
    }
  }
}

/// A corresponding report, calculated from the skeleton and sales data.
#[derive(Clone, Properties, PartialEq, Eq)]
pub(crate) struct Report {
  /// All string fields.
  sfields: Vec<StringField>,
  /// All table fields.
  tfields: Vec<TableField>,
  /// The "better" CSV.
  better_csv: Vec<Vec<String>>
}

/// A component that displays a report.
pub(crate) struct ReportDisplay;

impl ReportDisplay {
  fn make_csv_txt(report: &Report) -> String {
    let mut wr = csv::WriterBuilder::new()
      .double_quote(true)
      .delimiter(b',')
      .has_headers(true)
      .from_writer(vec![]);
    wr.write_record(SalePlus::better_csv_header()).ok();
    for l in &report.better_csv {
      wr.write_record(l).ok();
    }
    return String::from_utf8(
      wr.into_inner().unwrap_or(vec![])
    ).unwrap_or("ERRO".to_owned());
  }
}

impl Component for ReportDisplay {
  type Message = ();
  type Properties = Report;

  fn create(_ctx: &yew::Context<Self>) -> Self {
    return Self;
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    return html! {
      <div class="report">
        <hr />
        <small>
          <table class="sfields">
            {
              for ctx.props().sfields.iter().map(|sf| {
                html_nested! {
                  <tr>
                    <td><b>{ &sf.0 }</b></td>
                    <td>{ &sf.1 }</td>
                  </tr>
                }
              })
            }
          </table>
        </small>
        <hr />
        <div class="tfields">
          {
            for ctx.props().tfields.iter().map(|tf| {
              html_nested! {
                <div>
                  <b>{ &tf.0 }</b>{ ": " }
                  <br />
                  <small>
                    <table class="tfield-vals">
                      {
                        for tf.1.iter().map(|(k, v)| {
                          html_nested! {
                            <tr>
                              <td><b>{ &k }</b></td>
                              <td>{ &v }</td>
                            </tr>
                          }
                        })
                      }
                    </table>
                  </small>
                </div>
              }
            })
          }
        </div>
        <hr />
        <div class="better-csv">
          <b>{ "CSV melhorado:" }</b>
          <br />
          <textarea
            class="csv-in" 
            readonly=true
            value={ ReportDisplay::make_csv_txt(ctx.props()) }
          >
          </textarea>
        </div>
      </div>
    }
  }
}
