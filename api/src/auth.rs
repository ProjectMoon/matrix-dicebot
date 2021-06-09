use crate::config::Config;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::response::status::Custom;
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
};
use rocket::{routes, Route, State};
use std::error::Error;

pub(crate) fn routes() -> Vec<Route> {
    routes![login]
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    exp: usize,
    sub: String,
}

#[derive(Deserialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[rocket::post("/login", data = "<request>")]
async fn login<'a>(
    request: Json<LoginRequest<'a>>,
    config: &State<Config>,
) -> Result<String, Custom<String>> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::seconds(60))
        .expect("clock went awry")
        .timestamp();

    let claims = Claims {
        exp: expiration as usize,
        sub: request.username.to_owned(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(token)
}
