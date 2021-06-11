use graphql_client_web::ClientError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("api client error: {0}")]
    ApiClientError(#[from] ClientError),

    /// General API error, collecting errors from graphql server.
    #[error("error: {0}")]
    ApiError(String),

    #[error("login token invalid or expired")]
    NotLoggedIn,

    #[error("error: {0}")]
    JsError(String),
}

impl From<wasm_bindgen::JsValue> for UiError {
    fn from(js_error: wasm_bindgen::JsValue) -> UiError {
        UiError::JsError(
            js_error
                .as_string()
                .unwrap_or("unknown JS error".to_string()),
        )
    }
}
