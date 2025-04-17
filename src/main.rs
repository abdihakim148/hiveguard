#![allow(unused)]
/// Main module for the application.
mod adaptors;
/// Module for domain logic.
mod domain;
/// Module for input and output ports.
mod ports;


use adaptors::inputs::api::actix::{Actix, Result};

const NAME: &'static str = env!("CARGO_PKG_NAME");

/// Entry point of the application.
#[tokio::main]
async fn main() -> Result<()> {
    Actix::start().await
}
