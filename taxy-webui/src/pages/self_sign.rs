use crate::{
    auth::use_ensure_auth,
    pages::{
        cert_list::{CertsQuery, CertsTab},
        Route,
    },
    API_ENDPOINT,
};
use gloo_net::http::Request;
use std::{collections::HashMap, str::FromStr};
use taxy_api::{
    cert::{CertInfo, CertKind, SelfSignedCertRequest},
    id::ShortId,
    subject_name::SubjectName,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlSelectElement};
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

    let ca_cert = use_state(|| ShortId::from_str("generate").unwrap_throw());
    let ca_cert_onchange = Callback::from({
        let ca_cert = ca_cert.clone();
        move |event: Event| {
            let target: HtmlSelectElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            ca_cert.set(target.value().parse().unwrap_throw());
        }
    });

    let ca_cert_list = use_state(Vec::<CertInfo>::new);
    let ca_cert_list_cloned = ca_cert_list.clone();
    let ca_cert_cloned = ca_cert.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_cert_list().await {
                    let list = res
                        .into_iter()
                        .filter(|cert| cert.has_private_key && cert.kind == CertKind::Root)
                        .collect::<Vec<_>>();
                    if let Some(cert) = list.first() {
                        ca_cert_cloned.set(cert.id);
                    }
                    ca_cert_list_cloned.set(list);
                }
            });
        },
        (),
    );

    let validation = use_state(|| false);

    let entry = get_request(&san, *ca_cert);
    let is_loading = use_state(|| false);

    let entry_cloned = entry;
    let is_loading_cloned = is_loading;
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        validation.set(true);
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
            <form {onsubmit} class="bg-white shadow-sm p-5 border border-neutral-300 lg:rounded-md">
                <label class="block mb-2 text-sm font-medium text-neutral-900">{"Subject Alternative Names"}</label>
                <input type="text" value={san.to_string()} onchange={san_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" placeholder="example.com" />
                <p class="mt-2 text-sm text-neutral-500">{"You can use commas to list multiple names, e.g, example.com, *.test.examle.com."}</p>

                <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"CA Certificate"}</label>
                <select onchange={ca_cert_onchange} class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5">
                    { ca_cert_list.iter().map(|cert| {
                        html! {
                            <option selected={*ca_cert == cert.id} value={cert.id.to_string()}>{format!("{} ({})", cert.issuer, cert.id)}</option>
                        }
                    }).collect::<Html>() }
                    <option selected={ca_cert.to_string() == "generate"} value={"generate"}>{"Generate New CA Certificate"}</option>
                </select>

                <div class="flex mt-4 items-center justify-end">
                    <button type="button" onclick={cancel_onclick} class="mr-2 inline-flex items-center text-neutral-500 bg-neutral-50 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2">
                        {"Cancel"}
                    </button>
                    <button type="submit" class="inline-flex items-center text-neutral-500 bg-neutral-50 border border-neutral-300 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2">
                        {"Sign"}
                    </button>
                </div>
            </form>
        </>
    }
}

fn get_request(
    san: &str,
    ca_cert: ShortId,
) -> Result<SelfSignedCertRequest, HashMap<String, String>> {
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
            ca_cert: if ca_cert.to_string() == "generate" {
                None
            } else {
                Some(ca_cert)
            },
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

async fn get_cert_list() -> Result<Vec<CertInfo>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/certs"))
        .send()
        .await?
        .json()
        .await
}
