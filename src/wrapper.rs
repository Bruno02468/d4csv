//! A nice lil' wrapper for the app so I don't have to repeat HTML everywhere.

use yew::{Component, html};
use crate::app::App;

pub(crate) struct Wrapper;

impl Component for Wrapper {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
      return Self;
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
      return html! {
        <div class="wrapper">
          <center>
            <h1>{ "parser do csv da D4" }</h1>
            { "eu preciso dormir, cara. " }
            <br />
            <b>
              <a href="https://github.com/Bruno02468/d4csv" target="_blank">
                { "ah, e o código é aberto." }
              </a>
            </b>
          </center>
          <br />
          <br />
          <App />
        </div>
      };
    }
}
