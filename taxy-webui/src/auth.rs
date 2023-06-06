use crate::{pages::Route, store::UserSession, API_ENDPOINT};
use gloo_net::http::Request;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

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
