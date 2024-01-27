use crate::components::acme_provider::AcmeProvider;
use crate::components::custom_acme::CustomAcme;
use crate::pages::cert_list::{CertsQuery, CertsTab};
use crate::{auth::use_ensure_auth, pages::Route, API_ENDPOINT};
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
    GoogleTrustServices,
    ZeroSSL,
    Custom,
}

impl Provider {
    fn html(&self, onchanged: Callback<Result<AcmeRequest, HashMap<String, String>>>) -> Html {
        match self {
            Provider::LetsEncrypt => {
                html! { <AcmeProvider name={self.to_string()} url={"https://acme-v02.api.letsencrypt.org/directory"} {onchanged} /> }
            }
            Provider::GoogleTrustServices => {
                html! { <AcmeProvider name={self.to_string()} eab={true} url={"https://dv.acme-v02.api.pki.goog/directory"} {onchanged} /> }
            }
            Provider::ZeroSSL => {
                html! { <AcmeProvider name={self.to_string()} eab={true} url={"https://acme.zerossl.com/v2/DV90"} {onchanged} /> }
            }
            Provider::Custom => html! { <CustomAcme {onchanged} /> },
        }
    }
}

impl ToString for Provider {
    fn to_string(&self) -> String {
        match self {
            Provider::LetsEncrypt => "Let's Encrypt".to_string(),
            Provider::GoogleTrustServices => "Google Trust Services".to_string(),
            Provider::ZeroSSL => "ZeroSSL".to_string(),
            Provider::Custom => "Custom".to_string(),
        }
    }
}

const PROVIDERS: &[Provider] = &[
    Provider::LetsEncrypt,
    Provider::GoogleTrustServices,
    Provider::ZeroSSL,
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
    let onchanged: Callback<Result<AcmeRequest, HashMap<String, String>>> =
        Callback::from(move |updated| {
            entry_cloned.set(updated);
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
            <form {onsubmit} class="bg-white dark:bg-neutral-800 shadow-sm p-5 border border-neutral-300 dark:border-neutral-700 lg:rounded-md">
                <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900 dark:text-neutral-200">{"Provider"}</label>
                <select onchange={provider_onchange} class="bg-neutral-50 dark:text-neutral-200 dark:bg-neutral-800 dark:border-neutral-600 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5">
                    { PROVIDERS.iter().enumerate().map(|(i, item)| {
                        html! {
                            <option selected={&*provider == item} value={i.to_string()}>{item.to_string()}</option>
                        }
                    }).collect::<Html>() }
                </select>

                { provider.html(onchanged) }

                <div class="flex mt-4 items-center justify-end">
                    <button type="button" onclick={cancel_onclick} class="mr-2 inline-flex items-center text-neutral-500 bg-neutral-50 dark:text-neutral-200 dark:bg-neutral-800 focus:outline-none hover:bg-neutral-100 hover:dark:bg-neutral-900 focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-600 font-medium rounded-lg text-sm px-4 py-2">
                        {"Cancel"}
                    </button>
                    <button disabled={entry.is_err()} type="submit" class="inline-flex items-center text-neutral-500 bg-neutral-50 dark:text-neutral-200 dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-600 focus:outline-none hover:bg-neutral-100 hover:dark:bg-neutral-900 focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-600 font-medium rounded-lg text-sm px-4 py-2">
                        if *is_loading {
                            <svg aria-hidden="true" role="status" class="inline w-4 h-4 mr-3 text-neutral-200 animate-spin dark:text-neutral-600" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                            <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="#ccc"/>
                            <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="#1C64F2"/>
                            </svg>
                        }
                        {"Request"}
                    </button>
                </div>
            </form>
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
