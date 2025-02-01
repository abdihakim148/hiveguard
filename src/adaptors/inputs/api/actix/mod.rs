use actix_web::{get, post, web::{Data, Json}, App, HttpServer, Responder, HttpResponseBuilder as ResponseBuilder, http::StatusCode};
use crate::adaptors::outputs::database::memory::Memory;
use crate::adaptors::outputs::mailer::smtp::SmtpMailer;
use crate::ports::inputs::config::Config as Conf;
use crate::ports::outputs::database::Database;
use crate::domain::services::Registration;
use crate::domain::types::{User, Config};
use std::error::Error as StdError;
use crate::domain::types::Error;
use std::sync::Arc;


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
            .service(greet)
            .service(register)
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
        Ok(())
    }
}



#[get("/")]
async fn greet() -> impl Responder {
    let mut builder = ResponseBuilder::new(StatusCode::OK);
    builder.content_type("html");
    let body = "<h1>Hello World!!!!</h1>";
    builder.body(body)
}


#[post("/register")]
async fn register(user: Json<User>, config: Data<Arc<Config<DB, Mailer>>>) -> Response<impl Responder> {
    let table = config.db().users().await?;
    let argon = config.argon();
    let user = user.register(table, argon).await?;
    Ok(user)
}