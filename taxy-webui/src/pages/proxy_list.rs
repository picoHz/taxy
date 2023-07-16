use crate::auth::use_ensure_auth;
use crate::components::breadcrumb::Breadcrumb;
use crate::pages::Route;
use crate::store::{PortStore, ProxyStore};
use crate::utils::format_multiaddr;
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use taxy_api::port::PortEntry;
use taxy_api::site::ProxyEntry;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(ProxyList)]
pub fn proxy_list() -> Html {
    use_ensure_auth();

    let (ports, ports_dispatcher) = use_store::<PortStore>();
    let (proxies, proxies_dispatcher) = use_store::<ProxyStore>();

    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_list().await {
                    proxies_dispatcher.set(ProxyStore { entries: res });
                }
            });
        },
        (),
    );

    let ports_cloned = ports.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_ports().await {
                    ports_dispatcher.set(PortStore {
                        entries: res,
                        ..(*ports_cloned).clone()
                    });
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();
    let list = proxies.entries.clone();
    let active_index = use_state(|| -1);

    let navigator_cloned = navigator.clone();
    let new_proxy_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::NewProxy);
    });

    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>
            if list.is_empty() {
                <ybc::Hero body_classes="has-text-centered" body={
                    html! {
                    <p class="title has-text-grey-lighter">
                        {"No Items"}
                    </p>
                    }
                } />
            }
            <div class="list has-visible-pointer-controls">
            { list.into_iter().enumerate().map(|(i, entry)| {
                let navigator = navigator.clone();

                let id = entry.id.clone();
                let navigator_cloned = navigator.clone();
                let log_onclick = Callback::from(move |_|  {
                    let id = id.clone();
                    navigator_cloned.push(&Route::ProxyLogView {id});
                });

                let id = entry.id.clone();
                let config_onclick = Callback::from(move |_|  {
                    let id = id.clone();
                    navigator.push(&Route::ProxyView {id});
                });

                let delete_onmousedown = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                });
                let id = entry.id.clone();
                let delete_onclick = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                    if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                        let id = id.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let _ = delete_site(&id).await;
                        });
                    }
                });

                let active_index_cloned = active_index.clone();
                let dropdown_onclick = Callback::from(move |_|  {
                    active_index_cloned.set(if *active_index_cloned == i as i32 {
                        -1
                    } else {
                        i as i32
                    });
                });
                let active_index_cloned = active_index.clone();
                let dropdown_onfocusout = Callback::from(move |_|  {
                    active_index_cloned.set(-1);
                });
                let is_active = *active_index == i as i32;

                let ports = if entry.proxy.ports.is_empty() {
                    "-".to_string()
                } else {
                    entry.proxy.ports.iter().filter_map(|port| {
                        ports.entries.iter().find(|p| p.id == *port)
                    }).map(|entry| format_multiaddr(&entry.port.listen)).collect::<Vec<_>>().join(", ")
                };

                let title = if entry.proxy.name.is_empty() {
                    entry.id.clone()
                } else {
                    entry.proxy.name.clone()
                };
                html! {
                    <div class="list-item">
                        <div class="list-item-content">
                            <div class="list-item-title">{title}</div>
                            <div class="list-item-description">{ports}</div>
                        </div>

                        <div class="list-item-controls">
                            <div class="buttons is-right">

                            <button type="button" class="button" data-tooltip="Logs" onclick={log_onclick}>
                                <span class="icon is-small">
                                    <ion-icon name="receipt"></ion-icon>
                                </span>
                            </button>

                            <button type="button" class="button" data-tooltip="Configs" onclick={config_onclick}>
                                <span class="icon is-small">
                                    <ion-icon name="settings"></ion-icon>
                                </span>
                            </button>

                                <div class={classes!("dropdown", "is-right", is_active.then_some("is-active"))}>
                                    <div class="dropdown-trigger" onfocusout={dropdown_onfocusout}>
                                        <button type="button" class="button" onclick={dropdown_onclick}>
                                            <span class="icon is-small">
                                                <ion-icon name="ellipsis-horizontal"></ion-icon>
                                            </span>
                                        </button>
                                    </div>
                                    <div class="dropdown-menu" id="dropdown-menu" role="menu">
                                        <div class="dropdown-content">
                                            <a class="dropdown-item has-text-danger" onmousedown={delete_onmousedown} onclick={delete_onclick}>
                                                <span class="icon-text">
                                                    <span class="icon">
                                                        <ion-icon name="trash"></ion-icon>
                                                    </span>
                                                    <span>{"Delete"}</span>
                                                </span>
                                            </a>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }).collect::<Html>() }
            </div>
            <ybc::CardFooter>
                <a class="card-footer-item" onclick={new_proxy_onclick}>
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="add"></ion-icon>
                    </span>
                    <span>{"New Proxy"}</span>
                    </span>
                </a>
            </ybc::CardFooter>
            </ybc::Card>
        </>
    }
}

async fn get_ports() -> Result<Vec<PortEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports"))
        .send()
        .await?
        .json()
        .await
}

async fn get_list() -> Result<Vec<ProxyEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/proxies"))
        .send()
        .await?
        .json()
        .await
}

async fn delete_site(id: &str) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/proxies/{id}"))
        .send()
        .await?;
    Ok(())
}
