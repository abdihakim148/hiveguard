use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponseBuilder as ResponseBuilder, http::StatusCode, HttpResponse};
use crate::adaptors::outputs::database::memory::{MEMORY, Memory};
use crate::ports::{Error, outputs::database::Database};
use serde_json::to_string;
use crate::domain::services::Registration;
use crate::domain::types::User;


type Response = Result<HttpResponse, Box<dyn std::error::Error + 'static>>;


pub struct Actix;


impl Actix {
    pub async fn start() -> std::io::Result<()> {
        HttpServer::new(|| {
            App::new()
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
async fn register(user: web::Json<User>) -> Response {
    let table = MEMORY.get_or_try_init(||async {Memory::new(()).await}).await?.users().await?;
    let user = user.register(table).await?;
    let mut builder = ResponseBuilder::new(StatusCode::CREATED);
    builder.content_type("application/json");
    Ok(builder.body(to_string(&user)?))
}