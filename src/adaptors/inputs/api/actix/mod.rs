use crate::adaptors::outputs::database::memory::Memory;
use crate::adaptors::outputs::mailer::smtp::SmtpMailer;
use crate::ports::inputs::config::Config as Conf;
use actix_web::{web::Data, App, HttpServer};
use crate::domain::services::Authentication;
use std::error::Error as StdError;
use crate::domain::types::Config;
use crate::ports::Error;
use std::sync::Arc;


mod user;


type Response<T> = std::result::Result<T, Error>;
#[cfg(feature = "memory")]
type DB = Memory;
#[cfg(feature = "smtp")]
type Mailer = SmtpMailer;
pub type Result<T> = std::result::Result<T, Box<dyn StdError + 'static>>;

#[derive(Default)]
pub struct Actix;


impl Actix {
    pub async fn start() -> Result<()> {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
        let state = Arc::new(<Config<Memory, Mailer> as Conf>::load(None, ()).await?);
        let data = Data::new(state);
        HttpServer::new(move|| {
            App::new()
            .app_data(data.clone())
            .service(user::signup)
            .service(user::login)
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
        Ok(())
    }
}