use crate::config::Config;
use crate::errors::ApiError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::request::{self, FromRequest, Request};
use rocket::response::status::Custom;
use rocket::{
    http::Status,
    serde::{json::Json, Deserialize, Serialize},
};
use rocket::{
    http::{Cookie, CookieJar},
    outcome::Outcome,
};
use rocket::{routes, Route, State};
use substring::Substring;

#[derive(Clone, Debug)]
pub(crate) struct User {
    username: String, //TODO more state and such here.
}

fn decode_token(token: &str, config: &Config) -> Result<Claims, ApiError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config: Option<&Config> = req.rocket().state();
        let auth_header = req
            .headers()
            .get_one("Authorization")
            .map(|auth| auth.substring("Bearer ".len(), auth.len()));

        let token = auth_header
            .zip(config)
            .map(|(encoded_token, app_cfg)| decode_token(encoded_token, app_cfg))
            .unwrap_or(Err(ApiError::AuthenticationDenied("username".to_string())));

        match token {
            Err(e) => Outcome::Failure((Status::Forbidden, e)),
            Ok(token) => Outcome::Success(User {
                username: token.sub,
            }),
        }
    }
}

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
