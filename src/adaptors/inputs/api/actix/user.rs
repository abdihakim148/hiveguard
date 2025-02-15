use actix_web::{post, web::{Json, Data, Either, Form, Header}, Responder, HttpResponse};
use crate::domain::types::{Config, User, Contact, Audience};
use crate::domain::services::Authentication;
use super::{Response, DB, Mailer};
use serde::Deserialize;
use std::sync::Arc;


#[derive(Deserialize)]
struct Credentials {
    #[serde(alias = "email", alias = "phone", flatten)]
    pub contact: Contact,
    pub password: String
}

#[post("/signup")]
async fn signup(json: Json<User>, config: Data<Arc<Config<DB, Mailer>>>) -> Response<impl Responder> {
    let db = config.db();
    let hasher = config.argon();
    let issuer = config.name.clone();
    let paseto = config.paseto();
    let audience = Audience::None;
    let user = json.0;
    let token = user.register(db, hasher, paseto, issuer, audience).await?;
    Ok(token)
}


#[post("/login")]
async fn login(creds: Either<Json<Credentials>, Form<Credentials>>, config: Data<Arc<Config<DB, Mailer>>>) -> Response<impl Responder> {
    let credentials = creds.into_inner();
    let issuer = config.name.clone();
    let paseto = config.paseto();
    let audience = Audience::None;
    let verifier = config.argon();
    let db = config.db();
    let contact = &credentials.contact;
    let password = credentials.password.as_str();
    let token = User::authenticate(contact, password, db, verifier, paseto, issuer, audience).await?;
    Ok(token)
}