use crate::{pages::Route, store::SessionStore, API_ENDPOINT};
use gloo_net::http::Request;
use serde_derive::{Deserialize, Serialize};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct LoginQuery {
    #[serde(default)]
    pub redirect: Option<Route>,
}

#[hook]
pub fn use_ensure_auth() {
    let navigator = use_navigator().unwrap();

    let query = LoginQuery {
        redirect: use_route::<Route>().filter(|route| route != &Route::Login),
    };

    let (session, dispatcher) = use_store::<SessionStore>();
    if let Some(token) = session.token.clone() {
        wasm_bindgen_futures::spawn_local(async move {
            if !test_token(&token).await {
                dispatcher.set(Default::default());
                let _ = navigator.replace_with_query(&Route::Login, &query);
            }
        });
    } else {
        let _ = navigator.replace_with_query(&Route::Login, &query);
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
