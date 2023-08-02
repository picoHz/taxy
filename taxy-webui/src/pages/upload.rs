use crate::{
    auth::use_ensure_auth,
    pages::{
        cert_list::{CertsQuery, CertsTab},
        Route,
    },
    API_ENDPOINT,
};
use gloo_net::http::Request;
use taxy_api::cert::{CertKind, UploadQuery};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Upload)]
pub fn upload() -> Html {
    use_ensure_auth();

    let location = use_location().unwrap();
    let query: UploadQuery = location.query().unwrap_or_default();

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

    let chain = use_state(|| Option::<web_sys::File>::None);
    let chain_onchange = Callback::from({
        let chain = chain.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            chain.set(target.files().and_then(|file| file.get(0)));
        }
    });

    let key = use_state(|| Option::<web_sys::File>::None);
    let key_onchange = Callback::from({
        let key = key.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            key.set(target.files().and_then(|file| file.get(0)));
        }
    });

    let is_loading = use_state(|| false);

    let chain_cloned = (*chain).clone();
    let key_cloned = (*key).clone();
    let is_loading_cloned = is_loading.clone();
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let chain_cloned = chain_cloned.clone();
        let key_cloned = key_cloned.clone();
        let is_loading_cloned = is_loading_cloned.clone();
        if let Some(chain) = chain_cloned {
            is_loading_cloned.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if upload_cert(&chain, key_cloned.as_ref(), query.kind)
                    .await
                    .is_ok()
                {
                    let _ = navigator.push_with_query(
                        &Route::Certs,
                        &CertsQuery {
                            tab: if query.kind == CertKind::Server {
                                CertsTab::Server
                            } else {
                                CertsTab::Root
                            },
                        },
                    );
                }
                is_loading_cloned.set(false);
            });
        }
    });

    let uploadable = chain.is_some() && (query.kind == CertKind::Root || key.is_some());

    html! {
        <>
            <form {onsubmit} class="bg-white shadow-sm p-5 border border-neutral-300 md:rounded-md">
                <label class="block mb-2 text-sm font-medium text-neutral-900">{"Certificate Chain"}</label>
                <input onchange={chain_onchange} class="block w-full text-sm text-neutral-900 border border-neutral-300 rounded-lg cursor-pointer bg-neutral-50 focus:outline-none file:bg-transparent file:border-0 file:bg-neutral-100 file:mr-4 file:py-3 file:px-4" type="file" />

                <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Private Key"}</label>
                <input onchange={key_onchange} class="block w-full text-sm text-neutral-900 border border-neutral-300 rounded-lg cursor-pointer bg-neutral-50 focus:outline-none file:bg-transparent file:border-0 file:bg-neutral-100 file:mr-4 file:py-3 file:px-4" type="file" />

                <div class="flex mt-4 items-center justify-end">
                    <button type="button" onclick={cancel_onclick} class="mr-2 inline-flex items-center text-neutral-500 bg-neutral-50 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2">
                        {"Cancel"}
                    </button>
                    <button type="submit" disabled={!uploadable} class="inline-flex items-center text-neutral-500 bg-neutral-50 border border-neutral-300 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2">
                        {"Upload"}
                    </button>
                </div>
            </form>
        </>
    }
}

async fn upload_cert(
    chain: &web_sys::File,
    key: Option<&web_sys::File>,
    kind: CertKind,
) -> Result<(), gloo_net::Error> {
    let form_data = web_sys::FormData::new().unwrap();
    form_data.append_with_blob("chain", chain).unwrap();
    if let Some(key) = key {
        form_data.append_with_blob("key", key).unwrap();
    }

    Request::post(&format!("{API_ENDPOINT}/certs/upload"))
        .query([("kind", kind.to_string())])
        .body(form_data)?
        .send()
        .await?;

    Ok(())
}
