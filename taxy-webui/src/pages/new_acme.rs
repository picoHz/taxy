use crate::components::custom_acme::CustomAcme;
use crate::components::letsencrypt::LetsEncrypt;
use crate::pages::cert_list::{CertsQuery, CertsTab};
use crate::{
    auth::use_ensure_auth, components::breadcrumb::Breadcrumb, pages::Route, API_ENDPOINT,
};
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::acme::AcmeRequest;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlSelectElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    LetsEncrypt,
    LetsEncryptStaging,
    Custom,
}

impl Provider {
    fn html(&self, on_changed: Callback<Result<AcmeRequest, HashMap<String, String>>>) -> Html {
        match self {
            Provider::LetsEncrypt => html! { <LetsEncrypt staging={false} {on_changed} /> },
            Provider::LetsEncryptStaging => html! { <LetsEncrypt staging={true} {on_changed} /> },
            Provider::Custom => html! { <CustomAcme {on_changed} /> },
        }
    }
}

impl ToString for Provider {
    fn to_string(&self) -> String {
        match self {
            Provider::LetsEncrypt => "Let's Encrypt".to_string(),
            Provider::LetsEncryptStaging => "Let's Encrypt (Staging)".to_string(),
            Provider::Custom => "Custom".to_string(),
        }
    }
}

const PROVIDERS: &[Provider] = &[
    Provider::LetsEncrypt,
    Provider::LetsEncryptStaging,
    Provider::Custom,
];

#[function_component(NewAcme)]
pub fn new_acme() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        let _ = navigator_cloned.push_with_query(
            &Route::Certs,
            &CertsQuery {
                tab: CertsTab::Acme,
            },
        );
    });

    let entry =
        use_state::<Result<AcmeRequest, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let on_changed: Callback<Result<AcmeRequest, HashMap<String, String>>> =
        Callback::from(move |updated| {
            entry_cloned.set(updated);
        });

    let is_loading = use_state(|| false);

    let entry_cloned = entry.clone();
    let is_loading_cloned = is_loading.clone();
    let add_acme_onclick = Callback::from(move |_| {
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let is_loading_cloned = is_loading_cloned.clone();
        if let Ok(entry) = (*entry_cloned).clone() {
            is_loading_cloned.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if add_acme(&entry).await.is_ok() {
                    let _ = navigator.push_with_query(
                        &Route::Certs,
                        &CertsQuery {
                            tab: CertsTab::Acme,
                        },
                    );
                }
                is_loading_cloned.set(false);
            });
        }
    });

    let provider = use_state(|| PROVIDERS[0]);
    let provider_onchange = Callback::from({
        let provider = provider.clone();
        move |event: Event| {
            let target: HtmlSelectElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            if let Ok(index) = target.value().parse::<usize>() {
                provider.set(PROVIDERS[index]);
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

            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                    <label class="label">{"Proivder"}</label>
                </div>
                <div class="field-body">
                    <div class="field is-narrow">
                    <div class="control">
                        <div class="select is-fullwidth">
                        <select onchange={provider_onchange}>
                            { PROVIDERS.iter().enumerate().map(|(i, item)| {
                                html! {
                                    <option selected={&*provider == item} value={i.to_string()}>{item.to_string()}</option>
                                }
                            }).collect::<Html>() }
                        </select>
                        </div>
                    </div>
                    </div>
                </div>
            </div>

            { provider.html(on_changed) }

            <div class="field is-grouped is-grouped-right mx-5">
                <p class="control">
                    <button class="button is-light" onclick={cancel_onclick}>
                    {"Cancel"}
                    </button>
                </p>
                <p class="control">
                    <button class={classes!("button", "is-primary", is_loading.then_some("is-loading"))} onclick={add_acme_onclick} disabled={entry.is_err()}>
                    {"Request"}
                    </button>
                </p>
            </div>
            </ybc::Card>
        </>
    }
}

async fn add_acme(req: &AcmeRequest) -> Result<(), gloo_net::Error> {
    Request::post(&format!("{API_ENDPOINT}/acme"))
        .json(&req)?
        .send()
        .await?
        .json()
        .await
}
