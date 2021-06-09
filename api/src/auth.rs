use crate::config::Config;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::{Cookie, CookieJar};
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

fn create_token<'a>(
    username: &str,
    expiration: Duration,
    secret: &str,
) -> Result<String, Custom<String>> {
    let expiration = Utc::now()
        .checked_add_signed(expiration)
        .expect("clock went awry")
        .timestamp();

    let claims = Claims {
        exp: expiration as usize,
        sub: username.to_owned(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?;

    Ok(token)
}

#[rocket::post("/login", data = "<request>")]
async fn login(
    request: Json<LoginRequest<'_>>,
    config: &State<Config>,
    cookies: &CookieJar<'_>,
) -> Result<String, Custom<String>> {
    let token = create_token(request.username, Duration::minutes(1), &config.jwt_secret)?;
    let refresh_token = create_token(request.username, Duration::weeks(1), &config.jwt_secret)?;

    cookies.add_private(Cookie::new("refresh_token", refresh_token));
    Ok(token)
}
