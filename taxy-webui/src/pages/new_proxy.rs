use crate::{
    auth::use_ensure_auth, components::proxy_config::ProxyConfig, pages::Route, API_ENDPOINT,
};
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::site::Proxy;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(NewProxy)]
pub fn new_proxy() -> Html {
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
            <form {onsubmit} class="bg-white shadow-sm p-5 border border-neutral-300 lg:rounded-md">
                <ProxyConfig {onchanged} />

                <div class="flex mt-4 items-center justify-end">
                    <button type="button" onclick={cancel_onclick} class="mr-2 inline-flex items-center text-neutral-500 bg-neutral-50 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2">
                        {"Cancel"}
                    </button>
                    <button type="submit" disabled={entry.is_err()} class="inline-flex items-center text-neutral-500 bg-neutral-50 border border-neutral-300 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2">
                        {"Create"}
                    </button>
                </div>
            </form>
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
