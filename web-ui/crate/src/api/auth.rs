use crate::error::UiError;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, Request, RequestCredentials, RequestInit, RequestMode, Response};

/// A struct representing an error coming back from the REST API
/// endpoint. The API server encodes any errors as JSON objects with a
/// "message" property containing the error, and a bad status code.
#[derive(Debug, Serialize, Deserialize)]
struct ApiError {
    message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    jwt_token: String,
}

async fn make_request<T>(request: Request) -> Result<T, UiError>
where
    T: for<'a> Deserialize<'a>,
{
    let window = web_sys::window().unwrap();

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    let ok = resp.ok();

    let json = JsFuture::from(resp.json()?).await?;
    console::log_1(&json);
    //if ok, attempt to deserialize into T.
    //if not ok, attempt to deserialize into struct with message, and fall back
    //if that fails.
    if ok {
        let data: T = json.into_serde()?;
        Ok(data)
    } else {
        let data: ApiError = json.into_serde()?;
        Err(UiError::ApiError(data.message.unwrap_or_else(|| {
            let status_text = resp.status_text();
            let status = resp.status();
            format!("[{}] - {} - unknown api error", status, status_text)
        })))
    }
}

pub async fn login(username: &str, password: &str) -> Result<String, UiError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.credentials(RequestCredentials::Include);

    let body = JsValue::from_str(
        &serde_json::json!({
            "username": username,
            "password": password
        })
        .to_string(),
    );

    opts.body(Some(&body));
    let url = format!("http://localhost:10000/login");

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Content-Type", "application/json")?;
    request.headers().set("Accept", "application/json")?;

    let response: LoginResponse = make_request(request).await?;
    Ok(response.jwt_token)
}

pub async fn refresh_jwt() -> Result<String, UiError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.credentials(RequestCredentials::Include);

    let url = format!("http://localhost:10000/refresh");

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Content-Type", "application/json")?;
    request.headers().set("Accept", "application/json")?;

    let response: LoginResponse = make_request(request).await?;
    Ok(response.jwt_token)
}
