use crate::{
    auth::use_ensure_auth,
    components::{breadcrumb::Breadcrumb, proxy_config::ProxyConfig},
    pages::Route,
    API_ENDPOINT,
};
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::site::Proxy;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(NewProxy)]
pub fn new_site() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();

    let entry = use_state::<Result<Proxy, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let onchanged: Callback<Result<Proxy, HashMap<String, String>>> =
        Callback::from(move |updated| {
            entry_cloned.set(updated);
        });

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Proxies);
    });

    let is_loading = use_state(|| false);

    let entry_cloned = entry.clone();
    let is_loading_cloned = is_loading.clone();
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let is_loading_cloned = is_loading_cloned.clone();
        if let Ok(entry) = (*entry_cloned).clone() {
            is_loading_cloned.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if create_port(&entry).await.is_ok() {
                    navigator.push(&Route::Proxies);
                }
                is_loading_cloned.set(false);
            });
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

            <form {onsubmit}>
                <ProxyConfig {onchanged} />

                <div class="field is-grouped is-grouped-right mx-5">
                    <p class="control">
                        <button type="button" class="button is-light" onclick={cancel_onclick}>
                        {"Cancel"}
                        </button>
                    </p>
                    <p class="control">
                        <button type="submit" class={classes!("button", "is-primary", is_loading.then_some("is-loading"))} disabled={entry.is_err()}>
                        {"Create"}
                        </button>
                    </p>
                </div>
            </form>
            </ybc::Card>
        </>
    }
}

async fn create_port(entry: &Proxy) -> Result<(), gloo_net::Error> {
    Request::post(&format!("{API_ENDPOINT}/proxies"))
        .json(entry)?
        .send()
        .await?
        .json()
        .await
}
