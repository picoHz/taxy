use crate::auth::use_ensure_auth;
use crate::event::use_event_subscriber;
use crate::pages::Route;
use crate::store::{CertStore, PortStore, ProxyStore};
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use taxy_api::cert::CertInfo;
use taxy_api::port::PortEntry;
use taxy_api::site::ProxyEntry;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    use_ensure_auth();
    use_event_subscriber();

    let (ports, ports_dispatcher) = use_store::<PortStore>();
    let (proxies, proxies_dispatcher) = use_store::<ProxyStore>();
    let (certs, certs_dispatcher) = use_store::<CertStore>();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_port_list().await {
                    ports_dispatcher.set(PortStore { entries: res });
                }
                if let Ok(res) = get_proxy_list().await {
                    proxies_dispatcher.set(ProxyStore { entries: res });
                }
                if let Ok(res) = get_cert_list().await {
                    certs_dispatcher.set(CertStore { entries: res });
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let ports_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Ports);
    });

    let navigator_cloned = navigator.clone();
    let proxies_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Proxies);
    });

    let certs_onclick = Callback::from(move |_| {
        navigator.push(&Route::Certs);
    });

    html! {
        <div class="columns is-tablet">
            <ybc::Card classes="column p-0 m-5">
                <div class="p-5">
                <p class="heading">{"Ports"}</p>
                <p class="title">{ports.entries.len().to_string()}</p>
                </div>
                <ybc::CardFooter>
                    <a class="card-footer-item" onclick={ports_onclick}>
                        <span class="icon-text">
                        <span class="icon">
                            <ion-icon name="settings"></ion-icon>
                        </span>
                        <span>{"Edit"}</span>
                        </span>
                    </a>
                </ybc::CardFooter>
            </ybc::Card>
            <ybc::Card classes="column p-0 m-5">
                <div class="p-5">
                <p class="heading">{"Proxies"}</p>
                <p class="title">{proxies.entries.len().to_string()}</p>
                </div>
                <ybc::CardFooter>
                    <a class="card-footer-item" onclick={proxies_onclick}>
                        <span class="icon-text">
                        <span class="icon">
                            <ion-icon name="settings"></ion-icon>
                        </span>
                        <span>{"Edit"}</span>
                        </span>
                    </a>
                </ybc::CardFooter>
            </ybc::Card>
            <ybc::Card classes="column p-0 m-5">
                <div class="p-5">
                <p class="heading">{"Certificates"}</p>
                <p class="title">{certs.entries.len().to_string()}</p>
                </div>
                <ybc::CardFooter>
                    <a class="card-footer-item" onclick={certs_onclick}>
                        <span class="icon-text">
                        <span class="icon">
                            <ion-icon name="settings"></ion-icon>
                        </span>
                        <span>{"Edit"}</span>
                        </span>
                    </a>
                </ybc::CardFooter>
            </ybc::Card>
        </div>
    }
}

async fn get_port_list() -> Result<Vec<PortEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports"))
        .send()
        .await?
        .json()
        .await
}

async fn get_proxy_list() -> Result<Vec<ProxyEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/proxies"))
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
