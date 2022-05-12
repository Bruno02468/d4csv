//! The main app thing, minus the wrapper all-around.

use std::error::Error;
use csv::ReaderBuilder;
use web_sys::{Event, HtmlTextAreaElement, MouseEvent};
use yew::{Component, html, html_nested};
use yew::html::TargetCast;
use crate::context::{SalesContext, ContextInput};
use crate::report::{ReportDisplay, ReportTemplate};
use crate::sale::Sale;
use crate::sale::plus::SalesPlus;

#[derive(Debug)]
pub(crate) enum AppState {
  Input,
  Errors(Vec<Box<dyn Error>>),
  Loaded(SalesPlus)
}

#[derive(Debug)]
pub(crate) enum AppMsg {
  DoNothing,
  ShowErrors(Vec<Box<dyn Error>>),
  GotContext(SalesContext),
  GotCsv(String),
  TryReport
}

#[derive(Debug)]
pub(crate) struct App {
  context: Option<SalesContext>,
  csv_txt: Option<String>,
  state: AppState
}

impl App {
  /// Try and convert form data to SalesPlus.
  fn try_load(&self) -> Option<SalesPlus> {
    if let Some(ctx) = &self.context {
      if let Some(txt) = &self.csv_txt {
        let mut rdr = ReaderBuilder::new()
          .delimiter(b',')
          .quote(b'\"')
          .has_headers(true)
          .from_reader(txt.as_bytes());
        let sales = Sale::parse_csv(rdr.records(), &ctx);
        let mut sp = SalesPlus::from_sales(sales.0.into_iter(), ctx.clone());
        loop {
          if sp.comb_simple() == 0 { break; }
        }
        return Some(sp);
      }
    }
    return None;
  }
}

impl Component for App {
  type Message = AppMsg;
  type Properties = ();

  fn create(_ctx: &yew::Context<Self>) -> Self {
    return Self {
      context: None,
      csv_txt: None,
      state: AppState::Input
    };
  }

  fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    let mut b = false;
    // log::info!("{:#?}", &self);
    match msg {
      AppMsg::ShowErrors(ve) => {
        self.state = AppState::Errors(ve);
        b = true;
      },
      AppMsg::GotContext(ctx) => {
        self.context = Some(ctx);
      },
      AppMsg::GotCsv(s) => {
        self.csv_txt = Some(s);
      },
      AppMsg::TryReport => {
        if let Some(sp) = self.try_load() {
          self.state = AppState::Loaded(sp);
          b = true;
        }
      },
      _ => {}
    }
    return b;
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let csv_cb = ctx.link().callback(|e: Event| {
      let input: HtmlTextAreaElement = e.target_unchecked_into();
      let s = input.value();
      return Self::Message::GotCsv(s);
    });
    let btn_cb = ctx.link().callback(|_e: MouseEvent| {
      return Self::Message::TryReport;
    });
    return match &self.state {
      AppState::Input => {
        html! {
          <div class="app-input">
            <ContextInput />
            <br />
            <br />
            <textarea onchange={csv_cb} class="csv-in" />
            <br />
            <button onclick={btn_cb}>{ "bora" }</button>
          </div>
        }
      },
      AppState::Errors(ref v) => {
        html! {
          <div class="app-errors">
            { "deu ruim" }
            <br />
            <br />
            {
              for v.iter().map(|e| {
                html_nested! {
                  <>
                    <pre><code>{ e }</code></pre>
                    <br />
                    <br />
                  </>
                }
              })
            }
          </div>
        }
      },
      AppState::Loaded(sp) => {
        html! {
          <div class="app-report">
            <ReportDisplay ..ReportTemplate::default().compute(&sp) />
          </div>
        }
      },
    }
  }
}
