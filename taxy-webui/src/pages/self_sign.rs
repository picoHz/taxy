use crate::{
    auth::use_ensure_auth,
    components::breadcrumb::Breadcrumb,
    pages::{
        cert_list::{CertsQuery, CertsTab},
        Route,
    },
    API_ENDPOINT,
};
use gloo_net::http::Request;
use std::{collections::HashMap, str::FromStr};
use taxy_api::{cert::SelfSignedCertRequest, subject_name::SubjectName};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(SelfSign)]
pub fn self_sign() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();
    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        let _ = navigator_cloned.push_with_query(
            &Route::Certs,
            &CertsQuery {
                tab: CertsTab::Server,
            },
        );
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

    let is_loading = use_state(|| false);

    let entry_cloned = entry.clone();
    let is_loading_cloned = is_loading.clone();
    let self_sign_onclick = Callback::from(move |_| {
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let is_loading_cloned = is_loading_cloned.clone();
        if let Ok(entry) = entry_cloned.clone() {
            is_loading_cloned.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if request_self_sign(&entry).await.is_ok() {
                    let _ = navigator.push_with_query(
                        &Route::Certs,
                        &CertsQuery {
                            tab: CertsTab::Server,
                        },
                    );
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

            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                <label class="label">{"Subject Alternative Names"}</label>
                </div>
                <div class="field-body">
                    <div class="field">
                        <p class="control is-expanded">
                        <input class={classes!("input", san_err.as_ref().map(|_| "is-danger"))} type="text" autocapitalize="off" placeholder="Server Names" value={san.to_string()} onchange={san_onchange} />
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
                    <button class={classes!("button", "is-primary", is_loading.then_some("is-loading"))} onclick={self_sign_onclick} disabled={entry.is_err()}>
                    {"Sign"}
                    </button>
                </p>
            </div>
            </ybc::Card>
        </>
    }
}

fn get_request(san: &str) -> Result<SelfSignedCertRequest, HashMap<String, String>> {
    let mut errors = HashMap::new();
    let mut names = Vec::new();
    for name in san.split(',').filter(|s| !s.is_empty()) {
        if let Ok(name) = SubjectName::from_str(name) {
            names.push(name);
        } else {
            errors.insert("san".into(), "Invalid subject name.".into());
        }
    }
    if names.is_empty() {
        errors.insert(
            "san".into(),
            "At least one subject name is required.".into(),
        );
    }
    if errors.is_empty() {
        Ok(SelfSignedCertRequest {
            san: names,
            ca_cert: None,
        })
    } else {
        Err(errors)
    }
}

async fn request_self_sign(req: &SelfSignedCertRequest) -> Result<(), gloo_net::Error> {
    Request::post(&format!("{API_ENDPOINT}/certs/self_sign"))
        .json(&req)?
        .send()
        .await?
        .json()
        .await
}
