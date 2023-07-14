use crate::{
    auth::use_ensure_auth,
    components::{breadcrumb::Breadcrumb, proxy_config::ProxyConfig},
    pages::Route,
    store::ProxyStore,
    API_ENDPOINT,
};
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::site::{Proxy, ProxyEntry};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(ProxyView)]
pub fn proxy_view(props: &Props) -> Html {
    use_ensure_auth();

    let (proxies, _) = use_store::<ProxyStore>();
    let site = use_state(|| proxies.entries.iter().find(|e| e.id == props.id).cloned());
    let id = props.id.clone();
    let proxy_cloned = site.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(entry) = get_site(&id).await {
                    proxy_cloned.set(Some(entry));
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Proxies);
    });

    let entry = use_state::<Result<Proxy, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let onchanged: Callback<Result<Proxy, HashMap<String, String>>> =
        Callback::from(move |updated| {
            entry_cloned.set(updated);
        });

    let is_loading = use_state(|| false);

    let id = props.id.clone();
    let entry_cloned = entry.clone();
    let is_loading_cloned = is_loading.clone();
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let id = id.clone();
        let is_loading_cloned = is_loading_cloned.clone();
        if let Ok(entry) = (*entry_cloned).clone() {
            is_loading_cloned.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if update_site(&id, &entry).await.is_ok() {
                    navigator.push(&Route::Proxies);
                }
                is_loading_cloned.set(false);
            });
        }
    });

    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>

            if let Some(proxy_entry) = &*site {
                <form {onsubmit}>
                    <ProxyConfig proxy={proxy_entry.proxy.clone()} {onchanged} />

                    <div class="field is-grouped is-grouped-right mx-5 pb-5">
                        <p class="control">
                            <button type="button" class="button is-light" onclick={cancel_onclick}>
                            {"Cancel"}
                            </button>
                        </p>
                        <p class="control">
                            <button type="submit" class={classes!("button", "is-primary", is_loading.then_some("is-loading"))} disabled={entry.is_err()}>
                            {"Update"}
                            </button>
                        </p>
                    </div>
                </form>
            } else {
                <ybc::Hero body_classes="has-text-centered" body={
                    html! {
                    <p class="title has-text-grey-lighter">
                        {"Not Found"}
                    </p>
                    }
                } />
            }

            </ybc::Card>
        </>
    }
}

async fn get_site(id: &str) -> Result<ProxyEntry, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/proxies/{id}"))
        .send()
        .await?
        .json()
        .await
}

async fn update_site(id: &str, entry: &Proxy) -> Result<(), gloo_net::Error> {
    Request::put(&format!("{API_ENDPOINT}/proxies/{id}"))
        .json(entry)?
        .send()
        .await?
        .json()
        .await
}
