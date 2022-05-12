//! Set up the yew app.
//! Not much to see here.

#![allow(dead_code)]

pub(crate) mod ticket;
pub(crate) mod sale;
pub(crate) mod context;
pub(crate) mod report;
pub(crate) mod app;
mod wrapper;

fn main() {
  wasm_logger::init(wasm_logger::Config::default());
  yew::start_app::<wrapper::Wrapper>();
}
