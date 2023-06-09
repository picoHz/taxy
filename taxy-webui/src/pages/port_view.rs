use std::collections::HashMap;

use crate::{
    auth::use_ensure_auth,
    components::{breadcrumb::Breadcrumb, port_config::PortConfig},
    pages::Route,
    store::{PortStore, SessionStore},
    API_ENDPOINT,
};
use gloo_net::http::Request;
use taxy_api::port::{Port, PortEntry};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(PortView)]
pub fn port_view(props: &Props) -> Html {
    use_ensure_auth();

    let (ports, _) = use_store::<PortStore>();
    let port = use_state(|| ports.entries.iter().find(|e| e.id == props.id).cloned());
    let (session, _) = use_store::<SessionStore>();
    let token = session.token.clone();
    let id = props.id.clone();
    let port_cloned = port.clone();
    use_effect_with_deps(
        move |_| {
            if let Some(token) = token {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(entry) = get_port(&token, &id).await {
                        port_cloned.set(Some(entry));
                    }
                });
            }
        },
        session.clone(),
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Ports);
    });

    let entry = use_state::<Result<Port, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let on_changed: Callback<Result<Port, HashMap<String, String>>> =
        Callback::from(move |updated| {
            entry_cloned.set(updated);
        });

    let id = props.id.clone();
    let token = session.token.clone();
    let create_onclick = Callback::from(move |_| {
        let navigator = navigator.clone();
        let id = id.clone();
        if let Some(token) = token.clone() {
            if let Ok(entry) = (*entry).clone() {
                wasm_bindgen_futures::spawn_local(async move {
                    if update_port(&token, &id, &entry).await.is_ok() {
                        navigator.push(&Route::Ports);
                    }
                });
            }
        }
    });

    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>

            if let Some(entry) = &*port {
                <PortConfig port={entry.port.clone()} {on_changed} />

                <ybc::CardFooter>
                    <a class="card-footer-item" onclick={cancel_onclick}>
                        <span class="icon-text">
                        <span class="icon">
                            <ion-icon name="close"></ion-icon>
                        </span>
                        <span>{"Cancel"}</span>
                        </span>
                    </a>
                    <a class="card-footer-item" onclick={create_onclick}>
                        <span class="icon-text">
                        <span class="icon">
                            <ion-icon name="checkmark"></ion-icon>
                        </span>
                        <span>{"Update"}</span>
                        </span>
                    </a>
                </ybc::CardFooter>
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

async fn get_port(token: &str, id: &str) -> Result<PortEntry, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?
        .json()
        .await
}

async fn update_port(token: &str, id: &str, entry: &Port) -> Result<(), gloo_net::Error> {
    Request::put(&format!("{API_ENDPOINT}/ports/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(entry)?
        .send()
        .await?
        .json()
        .await
}
