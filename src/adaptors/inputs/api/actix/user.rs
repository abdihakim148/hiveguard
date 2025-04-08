use crate::domain::{services::{Get, Paseto, Update, Verification, Authentication}, types::{Audience, Config, Contact, Either as DomainEither, EmailAddress, Id, Key, User, Value, VerificationMedia, Phone}};
use actix_web::{post, get, patch, web::{self, Json, Data, Either, Form}, Responder, HttpResponse, HttpRequest, http::header};
use crate::ports::outputs::database::GetItem;
use crate::ports::outputs::oauth::OAuth; // Updated path for OAuth trait
use super::{Response, DB, Verifyer};
use std::collections::HashMap;
use super::error::Error;
use serde::Deserialize;
use std::str::FromStr;
use serde_json::json;
use std::sync::Arc;


#[derive(Deserialize)]
struct Credentials {
    #[serde(alias = "email", alias = "phone", flatten)]
    pub contact: Contact,
    pub password: String
}

#[derive(Deserialize)]
struct QueryParam {
    pub contact: DomainEither<Phone, EmailAddress>,
}


#[derive(Deserialize)]
struct Query {
    pub code: String,
}


#[post("/signup")]
async fn signup(req: HttpRequest, data: Either<Json<User>, Form<User>>, config: Data<Arc<Config<DB, Verifyer>>>) -> Response<impl Responder> {
    // Extract user data
    let user = match data {
        Either::Left(json) => json.0,
        Either::Right(form) => form.0,
    };
    
    // Construct base URL for verification
    let scheme_holder = req.connection_info();
    let scheme = scheme_holder.scheme();
    let base_url = format!("{}://{}/verify/confirm", scheme, config.domain);
    
    // Get required components
    let db = config.db();
    let hasher = config.argon();
    let verifyer = config.verifyer();
    
    // Determine channel based on contact type
    let channel = Default::default();
    
    // Register the user
    user.register(db, hasher, channel, &base_url, verifyer).await?;
    
    // Return 202 Accepted
    Ok(HttpResponse::Accepted().json(json!({
        "message": "User registered. Please verify your contact information."
    })))
}


#[post("/login")]
async fn login(creds: Either<Json<Credentials>, Form<Credentials>>, req: HttpRequest, config: Data<Arc<Config<DB, Verifyer>>>) -> Response<impl Responder> {
    // Extract credentials
    let is_json = matches!(creds, Either::Left(_));
    let creds = match creds {
        Either::Left(json) => json.0,
        Either::Right(form) => form.0,
    };
    
    // Construct base URL for verification
    let scheme_holder = req.connection_info();
    let scheme = scheme_holder.scheme();
    let base_url = format!("{}://{}/verify/confirm", scheme, config.domain);
    
    // Get required components
    let db = config.db();
    let hasher = config.argon();
    let paseto = config.paseto();
    let verifyer = config.verifyer();
    
    // Determine channel based on contact type
    let channel = Default::default();
    
    // Authenticate user
    let result = User::authenticate(
        &creds.contact, 
        &creds.password, 
        db, 
        hasher,
        paseto,
        config.name.clone(), // issuer
        Audience::None,      // audience
        channel,
        &base_url,
        verifyer
    ).await?;
    
    // Handle authentication result
    match result {
        None => {
            // User needs verification
            Ok(HttpResponse::Accepted().json(json!({
                "message": "Verification required. Please check your contact for verification code."
            })))
        },
        Some((user, token)) => {
            let token = token.try_sign(&paseto.keys)?;
            
            if is_json {
                // Return bearer token in JSON
                Ok(HttpResponse::Ok().insert_header((header::AUTHORIZATION, format!("Bearer {token}"))).json(user))
            } else {
                // Set cookie and return user
                Ok(HttpResponse::Ok().cookie(actix_web::cookie::Cookie::build("token", token).http_only(true).finish()).json(user))
            }
        }
    }
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
    let id = User::authorize(token, paseto).await?;
    
    let user = User::get(&id, db).await?;
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
    let id = User::authorize(token, paseto).await?;
    let item = item.0;
    let updated_user = User::update(&id, db, item).await?;
    Ok(updated_user)
}


#[post("/verify/request")]
async fn request_verification(
    req: HttpRequest,
    json: Json<QueryParam>, 
    config: Data<Arc<Config<DB, Verifyer>>>
) -> Response<impl Responder> {
    // Extract contact
    let contact = Into::<Contact>::into(json.0.contact).contact()?;
    
    // Construct base URL
    let scheme_holder = req.connection_info();
    let scheme = scheme_holder.scheme();
    let base_url = format!("{}://{}/verify/confirm", scheme, config.domain);
    
    // Get required components
    let db = config.db();
    let verifyer = config.verifyer();
    
    // Determine channel
    let channel = Default::default();
    
    // Initiate verification
    verifyer.initiate_verification(contact, channel, &base_url, db).await?;
    
    // Return success response
    Ok(HttpResponse::Accepted().json(json!({
        "message": "Verification code sent. Please check your contact."
    })))
}



#[get("/verify/confirm/{code}")]
async fn confirm_verification(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<QueryParam>, 
    config: Data<Arc<Config<DB, Verifyer>>>
) -> Response<impl Responder> {
    // Extract code and contact
    let code = path.into_inner();
    let email = Into::<Contact>::into(query.0.contact).contact()?;
    
    // Get required components
    let db = config.db();
    let verifyer = config.verifyer();
    let paseto = config.paseto();
    let mut id = Id::default();
    
    // Determine if code is ID or string code
    let code = if code.len() == 24 && code.chars().all(|c| c.is_ascii_hexdigit()) {
        // Looks like an ObjectId
        id = Id::from_str(&code)?;
        DomainEither::Right(&id)
    } else {
        // Regular verification code
        DomainEither::Left(code.as_str())
    };
    
    // Verify the contact
    let user = verifyer.confirm_verification(email, code, db).await?;
    
    // Generate token
    let token = user.token(
        config.name.clone(),
        Audience::None,
        paseto.ttl
    ).try_sign(&paseto.keys)?;
    
    // Return user and token (both as cookie and in JSON)
    Ok(
        HttpResponse::Ok()
        // return both a cookie and a bearer token
        .insert_header((header::AUTHORIZATION, format!("Bearer {token}")))
        .cookie(actix_web::cookie::Cookie::build("token", token).http_only(true).finish())
        .json(json!(user))
    )
}


#[get("/login/oauth/{provider}")]
async fn oauth_login(req: HttpRequest, path: web::Path<String>, config: Data<Arc<Config<DB, Verifyer>>>) -> Response<impl Responder> {
    let provider = path.as_str();
    let oauth = config.oauth();
    let scheme_holder = req.connection_info();
    let scheme = scheme_holder.scheme();
    let url = format!("{}://{}/login/oauth/{}/confirm", scheme, config.domain, provider);
    let redirect_url = url::Url::parse(&url).expect("INVALID REDIRECT URL");
    let url = oauth.authorization_url(provider, &redirect_url).await?;
    let url = url.as_str();
    let res = HttpResponse::TemporaryRedirect().insert_header((header::LOCATION, url)).finish();
    Ok(res)
}


#[get("/login/oauth/{provider}/confirm")]
async fn oauth_login_confirm(path: web::Path<String>, query: web::Query<Query>, config: Data<Arc<Config<DB, Verifyer>>>) -> Response<impl Responder> {
    let code = &query.code;
    let provider = path.as_str();
    let oauth = config.oauth();
    let user = oauth.authenticate(provider, code).await.unwrap();
    Ok(HttpResponse::Ok().json(user))
}