use crate::auth::use_ensure_auth;
use crate::components::breadcrumb::Breadcrumb;
use crate::pages::Route;
use crate::store::{PortStore, SiteStore};
use crate::utils::format_addr;
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use taxy_api::port::PortEntry;
use taxy_api::site::SiteEntry;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(SiteList)]
pub fn site_list() -> Html {
    use_ensure_auth();

    let (ports, ports_dispatcher) = use_store::<PortStore>();
    let (sites, sites_dispatcher) = use_store::<SiteStore>();

    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_list().await {
                    sites_dispatcher.set(SiteStore { entries: res });
                }
            });
        },
        (),
    );

    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_ports().await {
                    ports_dispatcher.set(PortStore { entries: res });
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();
    let list = sites.entries.clone();
    let active_index = use_state(|| -1);

    let navigator_cloned = navigator.clone();
    let new_site_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::NewSite);
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
                let config_onclick = Callback::from(move |_|  {
                    let id = id.clone();
                    navigator.push(&Route::SiteView {id});
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

                let ports = entry.site.ports.iter().filter_map(|port| {
                    ports.entries.iter().find(|p| p.id == *port)
                }).map(|entry| format_addr(&entry.port)).collect::<Vec<_>>().join(", ");
                html! {
                    <div class="list-item">
                        <div class="list-item-content">
                            <div class="list-item-title">{&entry.id}</div>
                            <div class="list-item-description">{ports}</div>
                        </div>

                        <div class="list-item-controls">
                            <div class="buttons is-right">
                            <button class="button" onclick={config_onclick}>
                                <span class="icon is-small">
                                    <ion-icon name="settings"></ion-icon>
                                </span>
                                <span>{"Config"}</span>
                            </button>

                                <div class={classes!("dropdown", "is-right", is_active.then_some("is-active"))}>
                                    <div class="dropdown-trigger" onfocusout={dropdown_onfocusout}>
                                        <button class="button" onclick={dropdown_onclick}>
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
                <a class="card-footer-item" onclick={new_site_onclick}>
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="add"></ion-icon>
                    </span>
                    <span>{"New Site"}</span>
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

async fn get_list() -> Result<Vec<SiteEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/sites"))
        .send()
        .await?
        .json()
        .await
}

async fn delete_site(id: &str) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/sites/{id}"))
        .send()
        .await?;
    Ok(())
}
