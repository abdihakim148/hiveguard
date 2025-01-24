#![allow(unused)]
/// Main module for the application.
mod adaptors;
/// Module for domain logic.
mod domain;
/// Module for input and output ports.
mod ports;


use adaptors::inputs::api::actix::Actix;

/// Entry point of the application.
#[tokio::main]
async fn main() -> std::io::Result<()> {
    Actix::start().await
}
