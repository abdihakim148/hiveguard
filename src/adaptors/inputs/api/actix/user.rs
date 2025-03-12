use actix_web::{post, get, patch, web::{Json, Data, Either, Form}, Responder, HttpResponse, HttpRequest};
use crate::domain::{services::{Get, Update}, types::{Audience, Config, Contact, User, Value}};
use crate::domain::services::Authentication;
use super::{Response, DB, Verifyer};
use std::collections::HashMap;
use super::error::Error;
use serde::Deserialize;
use std::sync::Arc;


#[derive(Deserialize)]
struct Credentials {
    #[serde(alias = "email", alias = "phone", flatten)]
    pub contact: Contact,
    pub password: String
}

#[post("/signup")]
async fn signup(json: Json<User>, config: Data<Arc<Config<DB, Verifyer>>>) -> Response<impl Responder> {
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
async fn login(creds: Either<Json<Credentials>, Form<Credentials>>, config: Data<Arc<Config<DB, Verifyer>>>) -> Response<impl Responder> {
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


#[get("/users/")]
async fn user_info(req: HttpRequest, config: Data<Arc<Config<DB, Verifyer>>>) -> Response<impl Responder> {
    let token = match req.cookie("token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            match req.headers().get(actix_web::http::header::AUTHORIZATION) {
                Some(value) => match value.to_str(){
                    Ok(value) => value.to_string(),
                    _ => Err(Error::UnAuthorized)?
                },
                None => Err(Error::UnAuthorized)?
            }
        }
    };
    let token = &token.replace("Bearer ", "");
    let paseto = config.paseto();
    let db = config.db();
    let id = &User::authorize(token, paseto).await?;
    let user = User::get(id, db).await?;
    Ok(user)
}


#[patch("/users/")]
async fn patch_user(req: HttpRequest, item: Json<HashMap<String, Value>>, config: Data<Arc<Config<DB, Verifyer>>>) -> Response<impl Responder> {
    let token = match req.cookie("token") {
        Some(cookie) => cookie.value().to_string(),
        None => {
            match req.headers().get(actix_web::http::header::AUTHORIZATION) {
                Some(value) => match value.to_str(){
                    Ok(value) => value.to_string(),
                    _ => Err(Error::UnAuthorized)?
                },
                None => Err(Error::UnAuthorized)?
            }
        }
    };
    let token = &token.replace("Bearer ", "");
    let paseto = config.paseto();
    let db = config.db();
    let id = &User::authorize(token, paseto).await?;
    let item = item.0;
    let updated_user = User::update(id, db, item).await?;
    Ok(updated_user)
}