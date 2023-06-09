use crate::{
    auth::use_ensure_auth,
    components::{breadcrumb::Breadcrumb, port_config::PortConfig},
    pages::Route,
    store::{PortStore, SessionStore},
    API_ENDPOINT,
};
use gloo_net::http::Request;
use taxy_api::port::PortEntry;
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
        session,
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator;
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Ports);
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
                <PortConfig port={entry.port.clone()} />

                <ybc::CardFooter>
                    <a class="card-footer-item" onclick={cancel_onclick}>
                        <span class="icon-text">
                        <span class="icon">
                            <ion-icon name="close"></ion-icon>
                        </span>
                        <span>{"Cancel"}</span>
                        </span>
                    </a>
                    <a class="card-footer-item">
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
