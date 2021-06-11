use crate::error::UiError;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, Request, RequestCredentials, RequestInit, RequestMode, Response};

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    jwt_token: String,
}

async fn make_request(request: Request) -> Result<JsValue, UiError> {
    let window = web_sys::window().unwrap();

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    let json = JsFuture::from(resp.json()?).await?;
    Ok(json)
}

pub async fn fetch_jwt() -> Result<String, UiError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.credentials(RequestCredentials::Include);

    opts.body(Some(&JsValue::from_str(
        r#"
          { "username": "@projectmoon:agnos.is", "password": "lolol" }
        "#,
    )));

    let url = format!("http://localhost:10000/login");

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Content-Type", "application/json")?;
    request.headers().set("Accept", "application/json")?;

    //TODO don't unwrap the response. OR... change it so we have a standard response.
    let response = make_request(request).await?;
    let response: LoginResponse = response.into_serde().unwrap();

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

    //TODO don't unwrap the response. OR... change it so we have a standard response.
    let response = make_request(request).await?;
    console::log_1(&response);
    let response: LoginResponse = response.into_serde().unwrap();

    Ok(response.jwt_token)
}
