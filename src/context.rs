//! Sale context that comes from outside the CSV.

use std::error::Error;
use itertools::Itertools;
use yew::{Component, Properties, html};
use yew::html::{TargetCast, Scope};
use web_sys::{Event, HtmlInputElement};
use crate::app::{App, AppMsg};
use crate::ticket::batch::{BatchPrices, iter2bp, bp2iter, Batch};

static WEBFEE_PRECISION: usize = 1000;
static PRICES_SEPARATOR: &str = ";";

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

impl Default for SalesContext {
  /// Data from the 2022 D4.
  fn default() -> Self {
    Self {
      online_fee: (11, 10),
      batches: iter2bp(vec![5500, 6500, 7500, 8500].into_iter()),
      promo_limit: Some(1)
    }
  }
}

/// Context input as it comes from the document.
#[derive(Clone, Debug, PartialEq, Properties)]
pub(crate) struct ContextInputData {
  webfee: f64,
  prices: String,
  promos: f64
}

impl TryFrom<ContextInputData> for SalesContext {
  type Error = Box<dyn Error>;

  fn try_from(data: ContextInputData) -> Result<Self, Self::Error> {
    let mut cents: Vec<usize> = Vec::new();
    let ic = data.prices
      .split(PRICES_SEPARATOR)
      .map(|s| str::parse::<f64>(s));
    for c in ic {
      match c {
        Ok(f) => cents.push((f * 100.0) as usize),
        Err(_) => return Err(
          format!(
            "preços inválidos! faça tipo: 55;65;77.5;100.0;101"
          ).into()
        )
      }
    }
    return Ok(Self {
      online_fee: (
        ((data.webfee + 1.0) * (WEBFEE_PRECISION as f64)) as usize,
        WEBFEE_PRECISION
      ),
      batches: iter2bp(cents.into_iter()),
      promo_limit: {
        if data.promos != 0.0 {
          Some(data.promos as usize)
        } else {
          None
        }
      }
    });
  }
}

impl From<&SalesContext> for ContextInputData {
  fn from(ctx: &SalesContext) -> Self {
    let mut bps: Vec<Batch> = bp2iter(&ctx.batches).collect();
    bps.sort_by_key(|b| b.num);
    return Self {
      webfee: (ctx.online_fee.0 as f64) / (ctx.online_fee.1 as f64) - 1.0,
      prices: bps.into_iter()
        .map(|b| (b.price as f64)/100.0)
        .join(PRICES_SEPARATOR),
      promos: match ctx.promo_limit {
        Some(n) => n as f64,
        None => 0.0,
      },
    }
  }
}

impl Default for ContextInputData {
  fn default() -> Self {
    return (&SalesContext::default()).into();
  }
}

/// A component for the user to input context info.
pub(crate) struct ContextInput {
  data: ContextInputData
}

/// The events the context input reacts to.
#[derive(Clone, Debug)]
pub(crate) enum ContextInputMsg {
  /// A change to the web fee number.
  WebfeeChanged(f64),
  /// A change to the batch prices list.
  PricesChanged(String),
  /// A change to the promo limits.
  PromosChanged(f64)
}

impl ContextInput {
  /// Try and send the context upward.
  fn send_up(&self, ctx: &yew::Context<Self>) {
    if let Ok(sc) = self.try_get_context() {
      if let Some(scope) = ctx.link().get_parent() {
        let app: Scope<App> = scope.clone().downcast::<App>();
        app.send_message(AppMsg::GotContext(sc));
      }
    }
  }
}

impl Component for ContextInput {
  type Message = ContextInputMsg;
  type Properties = ();

  fn create(ctx: &yew::Context<Self>) -> Self {
    let s = Self {
      data: ContextInputData::default()
    };
    s.send_up(ctx);
    return s;
  }

  fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
    let b = false;
    match msg {
      ContextInputMsg::WebfeeChanged(x) => {
        if self.data.webfee != x {
          self.data.webfee = x;
          // b = true;
        }
      },
      ContextInputMsg::PricesChanged(s) => {
        if self.data.prices != s {
          self.data.prices = s;
          // b = true;
        }
      },
      ContextInputMsg::PromosChanged(x) => {
        if self.data.promos != x {
          self.data.promos = x;
          // b = true;
        }
      },
    }
    self.send_up(ctx);
    return b;
  }

  fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
    let webfee_change = ctx.link().callback(|e: Event| {
      let input: HtmlInputElement = e.target_unchecked_into();
      let v = input.value_as_number();
      return Self::Message::WebfeeChanged(v);
    });
    let prices_change = ctx.link().callback(|e: Event| {
      let input: HtmlInputElement = e.target_unchecked_into();
      let v = input.value();
      return Self::Message::PricesChanged(v);
    });
    let promos_change = ctx.link().callback(|e: Event| {
      let input: HtmlInputElement = e.target_unchecked_into();
      let v = input.value_as_number();
      return Self::Message::PromosChanged(v);
    });
    return html! {
      <div id="context-form">
        { "taxa web:" }
        <input
          type="number"
          min=0 step=0.1
          onchange={webfee_change}
          value={Some(self.data.webfee.to_string())}
        />
        <br />
        { "preços dos lotes: " }
        <input
          type="text"
          onchange={prices_change}
          value={Some(self.data.prices.clone())}
        />
        <br />
        { "promo/pessoa:" }
        <input
          type="number"
          min=1
          step=1
          onchange={promos_change}
          value={Some(self.data.promos.to_string())}
        />
        <br />
      </div>
    }
  }
}

impl ContextInput {
  /// Tries to convert the input data into a proper SalesContext.
  pub(crate) fn try_get_context(
    &self
  ) -> Result<SalesContext, Box<dyn Error>> {
    return self.data.clone().try_into();
  }
}
