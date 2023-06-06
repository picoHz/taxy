use crate::breadcrumb::Breadcrumb;
use crate::pages::Route;
use crate::store::{PortStore, SiteStore};
use crate::API_ENDPOINT;
use crate::{auth::use_ensure_auth, store::SessionStore};
use gloo_net::http::Request;
use taxy_api::site::SiteEntry;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(SiteList)]
pub fn site_list() -> Html {
    use_ensure_auth();

    let (session, _) = use_store::<SessionStore>();
    let (ports, _) = use_store::<PortStore>();
    let (sites, dispatcher) = use_store::<SiteStore>();
    let token = session.token.clone();
    use_effect_with_deps(
        move |_| {
            if let Some(token) = token {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(res) = get_list(&token).await {
                        dispatcher.set(SiteStore { entries: res });
                    }
                });
            }
        },
        session,
    );

    let navigator = use_navigator().unwrap();
    let list = sites.entries.clone();
    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>
            <div class="list has-visible-pointer-controls">
            { list.into_iter().map(|entry| {
                let navigator = navigator.clone();
                let id = entry.id.clone();
                let onclick = Callback::from(move |_|  {
                    let id = id.clone();
                    navigator.push(&Route::SiteView {id});
                });
                let ports = entry.site.ports.iter().filter_map(|port| {
                    ports.entries.iter().find(|p| p.id == *port)
                }).map(|entry| entry.port.listen.to_string()).collect::<Vec<_>>().join(",");
                html! {
                    <div class="list-item">
                        <div class="list-item-content">
                            <div class="list-item-title">{&entry.id}</div>
                            <div class="list-item-description">{ports}</div>
                        </div>

                        <div class="list-item-controls">
                            <div class="buttons is-right">
                                <button class="button" {onclick}>
                                    <span class="icon is-small">
                                        <ion-icon name="settings"></ion-icon>
                                    </span>
                                    <span>{"Config"}</span>
                                </button>

                                <div class="dropdown is-right is-hoverable">
                                    <div class="dropdown-trigger">
                                        <button class="button">
                                            <span class="icon is-small">
                                                <ion-icon name="ellipsis-horizontal"></ion-icon>
                                            </span>
                                        </button>
                                    </div>
                                    <div class="dropdown-menu" id="dropdown-menu" role="menu">
                                        <div class="dropdown-content">
                                            <a href="#" class="dropdown-item">
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
                <a href="#" class="card-footer-item">
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

async fn get_list(token: &str) -> Result<Vec<SiteEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/sites"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?
        .json()
        .await
}
