use std::collections::HashMap;

use crate::{
    auth::use_ensure_auth, components::port_config::PortConfig, pages::Route, store::PortStore,
    API_ENDPOINT,
};
use gloo_net::http::Request;
use taxy_api::{
    id::ShortId,
    port::{Port, PortEntry},
};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: ShortId,
}

#[function_component(PortView)]
pub fn port_view(props: &Props) -> Html {
    use_ensure_auth();

    let (ports, _) = use_store::<PortStore>();
    let port = use_state(|| ports.entries.iter().find(|e| e.id == props.id).cloned());
    let id = props.id;
    let port_cloned = port.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(entry) = get_port(id).await {
                    port_cloned.set(Some(entry));
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Ports);
    });

    let entry = use_state::<Result<Port, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let onchanged: Callback<Result<Port, HashMap<String, String>>> =
        Callback::from(move |updated| {
            entry_cloned.set(updated);
        });

    let is_loading = use_state(|| false);

    let id = props.id;
    let entry_cloned = entry.clone();
    let is_loading_cloned = is_loading.clone();
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        if *is_loading_cloned {
            return;
        }
        let navigator = navigator.clone();
        let id = id;
        let is_loading_cloned = is_loading_cloned.clone();
        if let Ok(entry) = (*entry_cloned).clone() {
            is_loading_cloned.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                if update_port(id, &entry).await.is_ok() {
                    navigator.push(&Route::Ports);
                }
                is_loading_cloned.set(false);
            });
        }
    });

    html! {
        <>
            if let Some(port_entry) = &*port {
                <form {onsubmit} class="bg-white shadow-sm p-5 border border-neutral-300 lg:rounded-md">
                    <PortConfig port={port_entry.port.clone()} {onchanged} />

                    <div class="flex mt-4 items-center justify-end">
                        <button type="button" onclick={cancel_onclick} class="mr-2 inline-flex items-center text-neutral-500 bg-neutral-50 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2">
                            {"Cancel"}
                        </button>
                        <button type="submit" disabled={entry.is_err()} class="inline-flex items-center text-neutral-500 bg-neutral-50 border border-neutral-300 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2">
                            {"Update"}
                        </button>
                    </div>
                </form>
            } else {
                <Redirect<Route> to={Route::Ports}/>
            }
        </>
    }
}

async fn get_port(id: ShortId) -> Result<PortEntry, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports/{id}"))
        .send()
        .await?
        .json()
        .await
}

async fn update_port(id: ShortId, entry: &Port) -> Result<(), gloo_net::Error> {
    Request::put(&format!("{API_ENDPOINT}/ports/{id}"))
        .json(entry)?
        .send()
        .await?
        .json()
        .await
}
