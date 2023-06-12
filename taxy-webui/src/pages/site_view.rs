use crate::{
    auth::use_ensure_auth,
    components::{breadcrumb::Breadcrumb, site_config::SiteConfig},
    pages::Route,
    store::{SessionStore, SiteStore},
    API_ENDPOINT,
};
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::site::{Site, SiteEntry};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(SiteView)]
pub fn site_view(props: &Props) -> Html {
    use_ensure_auth();

    let (sites, _) = use_store::<SiteStore>();
    let site = use_state(|| sites.entries.iter().find(|e| e.id == props.id).cloned());
    let (session, _) = use_store::<SessionStore>();
    let token = session.token.clone();
    let id = props.id.clone();
    let site_cloned = site.clone();
    use_effect_with_deps(
        move |_| {
            if let Some(token) = token {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(entry) = get_site(&token, &id).await {
                        site_cloned.set(Some(entry));
                    }
                });
            }
        },
        session.clone(),
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Sites);
    });

    let entry = use_state::<Result<Site, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let on_changed: Callback<Result<Site, HashMap<String, String>>> =
        Callback::from(move |updated| {
            entry_cloned.set(updated);
        });

    let is_loading = use_state(|| false);

    let id = props.id.clone();
    let token = session.token.clone();
    let entry_cloned = entry.clone();
    let is_loading_cloned = is_loading.clone();
    let update_onclick = Callback::from(move |_| {
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let id = id.clone();
        let is_loading_cloned = is_loading_cloned.clone();
        if let Some(token) = token.clone() {
            if let Ok(entry) = (*entry_cloned).clone() {
                is_loading_cloned.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    if update_site(&token, &id, &entry).await.is_ok() {
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

            if let Some(site_entry) = &*site {
                <SiteConfig site={site_entry.site.clone()} {on_changed} />

                <div class="field is-grouped is-grouped-right mx-5">
                    <p class="control">
                        <button class="button is-light" onclick={cancel_onclick}>
                        {"Cancel"}
                        </button>
                    </p>
                    <p class="control">
                        <button class={classes!("button", "is-primary", is_loading.then_some("is-loading"))} onclick={update_onclick} disabled={entry.is_err()}>
                        {"Update"}
                        </button>
                    </p>
                </div>
            } else {
                <ybc::Hero body={
                    html! {
                    <p class="title">
                        {"Not Found"}
                    </p>
                    }
                } />
            }

            </ybc::Card>
        </>
    }
}

async fn get_site(token: &str, id: &str) -> Result<SiteEntry, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/sites/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?
        .json()
        .await
}

async fn update_site(token: &str, id: &str, entry: &Site) -> Result<(), gloo_net::Error> {
    Request::put(&format!("{API_ENDPOINT}/sites/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(entry)?
        .send()
        .await?
        .json()
        .await
}
