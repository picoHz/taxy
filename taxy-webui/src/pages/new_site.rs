use crate::{
    auth::use_ensure_auth,
    components::{breadcrumb::Breadcrumb, site_config::SiteConfig},
    pages::Route,
    store::SessionStore,
    API_ENDPOINT,
};
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::site::Site;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(NewSite)]
pub fn new_site() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();
    let (session, _) = use_store::<SessionStore>();
    let token = session.token.clone();

    let entry = use_state::<Result<Site, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let on_changed: Callback<Result<Site, HashMap<String, String>>> =
        Callback::from(move |updated| {
            entry_cloned.set(updated);
        });

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Sites);
    });

    let is_loading = use_state(|| false);

    let entry_cloned = entry.clone();
    let is_loading_cloned = is_loading.clone();
    let create_onclick = Callback::from(move |_| {
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let is_loading_cloned = is_loading_cloned.clone();
        if let Some(token) = token.clone() {
            if let Ok(entry) = (*entry_cloned).clone() {
                is_loading_cloned.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    if create_port(&token, &entry).await.is_ok() {
                        navigator.push(&Route::Sites);
                    }
                    is_loading_cloned.set(false);
                });
            }
        }
    });

    html! {
        <>
            <ybc::Card classes="pb-5">
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>

            <SiteConfig {on_changed} />

            <div class="field is-grouped is-grouped-right mx-5">
                <p class="control">
                    <button class="button is-light" onclick={cancel_onclick}>
                    {"Cancel"}
                    </button>
                </p>
                <p class="control">
                    <button class={classes!("button", "is-primary", is_loading.then_some("is-loading"))} onclick={create_onclick} disabled={entry.is_err()}>
                    {"Create"}
                    </button>
                </p>
            </div>
            </ybc::Card>
        </>
    }
}

async fn create_port(token: &str, entry: &Site) -> Result<(), gloo_net::Error> {
    Request::post(&format!("{API_ENDPOINT}/sites"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(entry)?
        .send()
        .await?
        .json()
        .await
}
