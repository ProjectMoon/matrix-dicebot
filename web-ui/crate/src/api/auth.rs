use crate::error::UiError;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, Request, RequestInit, RequestMode, Response};

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    jwt_token: String,
}

pub async fn fetch_jwt() -> Result<String, UiError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&JsValue::from_str(
        r#"
          { "username": "@projectmoon:agnos.is", "password": "lolol" }
        "#,
    )));

    let url = format!("http://localhost:10000/login");

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("Content-Type", "application/json")?;
    request.headers().set("Accept", "application/json")?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let json = JsFuture::from(resp.json()?).await?;

    console::log_1(&json);

    // Use serde to parse the JSON into a struct.
    let login_response: LoginResponse = json.into_serde().unwrap();

    // Send the `Branch` struct back to JS as an `Object`.
    Ok(login_response.jwt_token)
}
