use crate::components::letsencrypt::LetsEncrypt;
use crate::{
    auth::use_ensure_auth, components::breadcrumb::Breadcrumb, pages::Route, store::SessionStore,
    API_ENDPOINT,
};
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::acme::{Acme, AcmeRequest};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    LetsEncrypt,
    LetsEncryptStaging,
}

impl Provider {
    fn html(&self) -> Html {
        match self {
            Provider::LetsEncrypt => html! { <LetsEncrypt staging={false} /> },
            Provider::LetsEncryptStaging => html! { <LetsEncrypt staging={true} /> },
        }
    }
}

impl ToString for Provider {
    fn to_string(&self) -> String {
        match self {
            Provider::LetsEncrypt => "Let's Encrypt".to_string(),
            Provider::LetsEncryptStaging => "Let's Encrypt (Staging)".to_string(),
        }
    }
}

const PROVIDERS: &[Provider] = &[Provider::LetsEncrypt, Provider::LetsEncryptStaging];

#[function_component(NewAcme)]
pub fn new_acme() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();
    let (session, _) = use_store::<SessionStore>();
    let token = session.token.clone();

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Certs);
    });

    let san = use_state(String::new);
    let san_onchange = Callback::from({
        let san = san.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            san.set(target.value());
        }
    });

    let entry = get_request(&san);
    let san_err = entry
        .as_ref()
        .err()
        .and_then(|errors| errors.get("san").map(|s| s.to_string()));

    let entry_cloned = entry.clone();
    let self_sign_onclick = Callback::from(move |_| {
        let navigator = navigator.clone();
        if let Some(token) = token.clone() {
            if let Ok(entry) = entry_cloned.clone() {
                wasm_bindgen_futures::spawn_local(async move {
                    if add_acme(&token, &entry).await.is_ok() {
                        navigator.push(&Route::Certs);
                    }
                });
            }
        }
    });

    let provider = use_state(|| PROVIDERS[0]);

    html! {
        <>
            <ybc::Card classes="py-5">
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
                        <select>
                            { PROVIDERS.iter().map(|item| {
                                html! {
                                    <option selected={&*provider == item} value={item.to_string()}>{item.to_string()}</option>
                                }
                            }).collect::<Html>() }
                        </select>
                        </div>
                    </div>
                    </div>
                </div>
            </div>

            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                <label class="label">{"Provider"}</label>
                </div>
                <div class="field-body">
                    <div class="field">
                        <p class="control is-expanded">
                        <input class={classes!("input", san_err.as_ref().map(|_| "is-danger"))} type="text" placeholder="Server Names" value={san.to_string()} onchange={san_onchange} />
                        </p>
                        if let Some(err) = san_err {
                            <p class="help is-danger">{err}</p>
                        } else {
                            <p class="help">
                            {"You can use commas to list multiple names, e.g, example.com, *.test.examle.com."}
                            </p>
                        }
                    </div>
                </div>
            </div>

            <div class="field is-grouped is-grouped-right mx-5">
                <p class="control">
                    <button class="button is-light" onclick={cancel_onclick}>
                    {"Cancel"}
                    </button>
                </p>
                <p class="control">
                    <button class="button is-primary" onclick={self_sign_onclick} disabled={entry.is_err()}>
                    {"Request"}
                    </button>
                </p>
            </div>
            </ybc::Card>
        </>
    }
}

fn get_request(_san: &str) -> Result<AcmeRequest, HashMap<String, String>> {
    Ok(AcmeRequest {
        server_url: String::new(),
        contacts: vec![],
        eab: None,
        acme: Acme {
            provider: String::new(),
            identifiers: vec![],
            challenge_type: String::new(),
            renewal_days: 0,
            is_trusted: true,
        },
    })
}

async fn add_acme(token: &str, req: &AcmeRequest) -> Result<(), gloo_net::Error> {
    Request::post(&format!("{API_ENDPOINT}/acme"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&req)?
        .send()
        .await?
        .json()
        .await
}
