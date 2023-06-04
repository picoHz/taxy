use crate::{pages::Route, API_ENDPOINT};
use gloo_net::http::Request;
use serde_derive::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
pub struct UserSession {
    pub token: Option<String>,
}

#[hook]
pub fn use_ensure_auth() {
    let navigator = use_navigator().unwrap();
    let (session, dispatcher) = use_store::<UserSession>();
    if let Some(token) = session.token.clone() {
        wasm_bindgen_futures::spawn_local(async move {
            if !test_token(&token).await {
                dispatcher.set(Default::default());
                navigator.replace(&Route::Login);
            }
        });
    } else {
        navigator.replace(&Route::Login);
    }
}

pub async fn test_token(token: &str) -> bool {
    if let Ok(res) = Request::get(&format!("{API_ENDPOINT}/app_info"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
    {
        res.status() == 200
    } else {
        false
    }
}
