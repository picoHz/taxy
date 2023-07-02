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
            <ybc::Card classes="pb-5">
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>

            <form {onsubmit}>
                <div class="field is-horizontal m-5">
                    <div class="field-label is-normal">
                    <label class="label">{"Certificate Chain"}</label>
                    </div>
                    <div class="field-body">
                        <div class={classes!("file", chain.as_ref().map(|_| "has-name"))}>
                        <label class="file-label">
                        <input class="file-input" type="file" name="chain" onchange={chain_onchange} />
                        <span class="file-cta">
                            <span class="file-icon">
                                <ion-icon name="folder-open"></ion-icon>
                            </span>
                            <span class="file-label">
                            {"Choose a PEM file…"}
                            </span>
                        </span>
                        if let Some(file) = chain.as_ref() {
                            <span class="file-name">
                                {file.name()}
                            </span>
                        }
                        </label>
                    </div>
                    </div>
                </div>

                <div class="field is-horizontal m-5">
                    <div class="field-label is-normal">
                    <label class="label">{"Private Key"}</label>
                    </div>
                    <div class="field-body">
                        <div class={classes!("file", key.as_ref().map(|_| "has-name"))}>
                        <label class="file-label">
                        <input class="file-input" type="file" name="key" onchange={key_onchange} />
                        <span class="file-cta">
                            <span class="file-icon">
                                <ion-icon name="folder-open"></ion-icon>
                            </span>
                            <span class="file-label">
                            {"Choose a PEM file…"}
                            </span>
                        </span>
                        if let Some(file) = key.as_ref() {
                            <span class="file-name">
                                {file.name()}
                            </span>
                        }
                        </label>
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
                        <button type="submit" class={classes!("button", "is-primary", is_loading.then_some("is-loading"))} disabled={!uploadable}>
                        {"Upload"}
                        </button>
                    </p>
                </div>
            </form>
            </ybc::Card>
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
        .body(form_data)
        .send()
        .await?;

    Ok(())
}
