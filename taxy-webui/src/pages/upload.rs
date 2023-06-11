use crate::{
    auth::use_ensure_auth, components::breadcrumb::Breadcrumb, pages::Route, store::SessionStore,
    API_ENDPOINT,
};
use gloo_net::http::Request;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(Upload)]
pub fn upload() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();
    let (session, _) = use_store::<SessionStore>();
    let token = session.token.clone();

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Certs);
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
    let upload_onclick = Callback::from(move |_| {
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let chain_cloned = chain_cloned.clone();
        let key_cloned = key_cloned.clone();
        let is_loading_cloned = is_loading_cloned.clone();
        if let Some(token) = token.clone() {
            if let Some(chain) = chain_cloned {
                if let Some(key) = key_cloned {
                    is_loading_cloned.set(true);
                    wasm_bindgen_futures::spawn_local(async move {
                        if upload_cert(&token, &chain, &key).await.is_ok() {
                            navigator.push(&Route::Certs);
                        }
                        is_loading_cloned.set(false);
                    });
                }
            }
        }
    });

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
                    <button class={classes!("button", "is-primary", is_loading.then_some("is-loading"))} onclick={upload_onclick} disabled={chain.is_none() || key.is_none()}>
                    {"Upload"}
                    </button>
                </p>
            </div>
            </ybc::Card>
        </>
    }
}

async fn upload_cert(
    token: &str,
    chain: &web_sys::File,
    key: &web_sys::File,
) -> Result<(), gloo_net::Error> {
    let form_data = web_sys::FormData::new().unwrap();
    form_data.append_with_blob("chain", chain).unwrap();
    form_data.append_with_blob("key", key).unwrap();

    Request::post(&format!("{API_ENDPOINT}/server_certs/upload"))
        .header("Authorization", &format!("Bearer {token}"))
        .body(form_data)
        .send()
        .await?;

    Ok(())
}
