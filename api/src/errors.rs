use rocket::http::ContentType;
use rocket::response::{self, Responder, Response};
use rocket::{http::Status, request::Request};
use std::io::Cursor;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("user account does not exist: {0}")]
    UserDoesNotExist(String),

    #[error("invalid password for user account: {0}")]
    AuthenticationDenied(String),

    #[error("authentication token missing from request")]
    AuthenticationRequired,

    #[error("refresh token missing from request")]
    RefreshTokenMissing,

    #[error("error decoding token: {0}")]
    TokenDecodingError(#[from] jsonwebtoken::errors::Error),
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = match self {
            Self::UserDoesNotExist(_) => Status::Forbidden,
            Self::AuthenticationRequired => Status::Forbidden,
            Self::RefreshTokenMissing => Status::Forbidden,
            Self::AuthenticationDenied(_) => Status::Forbidden,
            Self::TokenDecodingError(_) => Status::InternalServerError,
        };

        //TODO certain errors might be too sensitive; need to filter them here.
        let body = serde_json::json!({
            "message": self.to_string()
        })
        .to_string();

        Response::build()
            .header(ContentType::JsonApi)
            .status(status)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}
