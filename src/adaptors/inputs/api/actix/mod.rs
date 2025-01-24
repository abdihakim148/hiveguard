use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponseBuilder as ResponseBuilder, http::StatusCode, HttpResponse};
use crate::adaptors::outputs::database::memory::Memory;
use crate::adaptors::outputs::mailer::smtp::SmtpMailer;
use crate::ports::outputs::database::Database;
use crate::domain::services::Registration;
use crate::domain::types::{User, Config};
use serde_json::to_string;
use crate::ports::Error;
use std::sync::Arc;


type Response = Result<HttpResponse, Error>;
#[cfg(feature = "memory")]
type DB = Memory;
#[cfg(feature = "smtp")]
type Mailer = SmtpMailer;

#[derive(Default)]
pub struct Actix;


impl Actix {
    pub async fn start() -> std::io::Result<()> {
        let state = <Config<Memory, Mailer> as Default>::default();
        let data = web::Data::new(state);
        HttpServer::new(move|| {
            App::new()
            .app_data(data.clone())
            .service(greet)
            .service(register)
        })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
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
async fn register(user: web::Json<User>, config: web::Data<Arc<Config<DB, Mailer>>>) -> Response {
    let table = config.db().users().await?;
    let argon = config.argon();
    let user = user.register(table, argon).await?;
    let mut builder = ResponseBuilder::new(StatusCode::CREATED);
    builder.content_type("application/json");
    Ok(builder.body(to_string(&user)?))
}