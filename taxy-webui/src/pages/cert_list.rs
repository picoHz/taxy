use crate::components::breadcrumb::Breadcrumb;
use crate::pages::Route;
use crate::store::{AcmeStore, CertStore};
use crate::API_ENDPOINT;
use crate::{auth::use_ensure_auth, store::SessionStore};
use gloo_net::http::Request;
use serde_derive::{Deserialize, Serialize};
use taxy_api::acme::AcmeInfo;
use taxy_api::cert::CertInfo;
use yew::prelude::*;

use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CertsTab {
    #[default]
    ServerCerts,
    Acme,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CertsQuery {
    #[serde(default)]
    pub tab: CertsTab,
}

impl ToString for CertsTab {
    fn to_string(&self) -> String {
        match self {
            CertsTab::ServerCerts => "Server Certificates",
            CertsTab::Acme => "ACME",
        }
        .into()
    }
}

const TABS: [CertsTab; 2] = [CertsTab::ServerCerts, CertsTab::Acme];

#[function_component(CertList)]
pub fn cert_list() -> Html {
    use_ensure_auth();

    let location = use_location().unwrap();
    let query: CertsQuery = location.query().unwrap_or_default();
    let tab = use_state(|| query.tab);

    let (session, _) = use_store::<SessionStore>();
    let (certs, certs_dispatcher) = use_store::<CertStore>();
    let (acme, acme_dispatcher) = use_store::<AcmeStore>();
    let token = session.token.clone();
    let token_cloned = token.clone();
    use_effect_with_deps(
        move |_| {
            if let Some(token) = token_cloned {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(res) = get_cert_list(&token).await {
                        certs_dispatcher.set(CertStore { entries: res });
                    }
                    if let Ok(res) = get_acme_list(&token).await {
                        acme_dispatcher.set(AcmeStore { entries: res });
                    }
                });
            }
        },
        session,
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let self_sign_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::SelfSign);
    });

    let navigator_cloned = navigator.clone();
    let upload_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Upload);
    });

    let navigator_cloned = navigator.clone();
    let new_acme_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::NewAcme);
    });

    let cert_list = certs.entries.clone();
    let acme_list = acme.entries.clone();
    let active_index = use_state(|| -1);
    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>
            <div class="tabs is-centered mb-0">
                <ul>
                    { TABS.into_iter().map(|item| {
                        let navigator = navigator.clone();
                        let is_active = item == *tab;
                        let tab = tab.clone();
                        let onclick = Callback::from(move |_|  {
                            tab.set(item);
                            let _ = navigator.push_with_query(&Route::Certs, &CertsQuery { tab: item });
                        });
                        html! {
                            <li class={classes!(is_active.then_some("is-active"))}>
                                <a {onclick}>{item}</a>
                            </li>
                        }
                    }).collect::<Html>() }

                </ul>
            </div>
            if *tab == CertsTab::ServerCerts {
            <div class="list has-visible-pointer-controls">
            { cert_list.into_iter().enumerate().map(|(i, entry)| {
                let subject_names = entry
                    .san
                    .iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let issuer = entry.issuer.to_string();
                let title = format!("{} ({})", subject_names, issuer);

                let delete_onmousedown = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                });
                let token_cloned = token.clone();
                let id = entry.id.clone();
                let delete_onclick = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                    if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                        let id = id.clone();
                        if let Some(token) = token_cloned.clone() {
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = delete_server_cert(&token, &id).await;
                            });
                        }
                    }
                });

                let active_index_cloned = active_index.clone();
                let dropdown_onclick = Callback::from(move |_|  {
                    active_index_cloned.set(i as i32);
                });
                let active_index_cloned = active_index.clone();
                let dropdown_onfocusout = Callback::from(move |_|  {
                    active_index_cloned.set(-1);
                });
                let is_active = *active_index == i as i32;

                html! {
                    <div class="list-item">
                        <div class="list-item-content">
                            <div class="list-item-title">{title}</div>
                            <div class="list-item-description">{&entry.id}</div>
                        </div>

                        <div class="list-item-controls">
                            <div class="buttons is-right">

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
                <a class="card-footer-item" onclick={self_sign_onclick}>
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="create"></ion-icon>
                    </span>
                    <span>{"Self-sign"}</span>
                    </span>
                </a>
                <a class="card-footer-item" onclick={upload_onclick}>
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="cloud-upload"></ion-icon>
                    </span>
                    <span>{"Upload"}</span>
                    </span>
                </a>
            </ybc::CardFooter>
            } else {
            <div class="list has-visible-pointer-controls">
            { acme_list.into_iter().enumerate().map(|(i, entry)| {
                let subject_names = entry.identifiers.join(", ");
                let provider = entry.provider.to_string();
                let title = format!("{} ({})", subject_names, provider);

                let delete_onmousedown = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                });
                let token_cloned = token.clone();
                let id = entry.id.clone();
                let delete_onclick = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                    if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                        let id = id.clone();
                        if let Some(token) = token_cloned.clone() {
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = delete_acme(&token, &id).await;
                            });
                        }
                    }
                });

                let active_index_cloned = active_index.clone();
                let dropdown_onclick = Callback::from(move |_|  {
                    active_index_cloned.set(i as i32);
                });
                let active_index_cloned = active_index.clone();
                let dropdown_onfocusout = Callback::from(move |_|  {
                    active_index_cloned.set(-1);
                });
                let is_active = *active_index == i as i32;

                html! {
                    <div class="list-item">
                        <div class="list-item-content">
                            <div class="list-item-title">{title}</div>
                            <div class="list-item-description">{&entry.id}</div>
                        </div>

                        <div class="list-item-controls">
                            <div class="buttons is-right">

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
                <a class="card-footer-item" onclick={new_acme_onclick}>
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="add"></ion-icon>
                    </span>
                    <span>{"New request"}</span>
                    </span>
                </a>
            </ybc::CardFooter>
            }
            </ybc::Card>
        </>
    }
}

async fn get_cert_list(token: &str) -> Result<Vec<CertInfo>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/server_certs"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?
        .json()
        .await
}

async fn get_acme_list(token: &str) -> Result<Vec<AcmeInfo>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/acme"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?
        .json()
        .await
}

async fn delete_server_cert(token: &str, id: &str) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/server_certs/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?;
    Ok(())
}

async fn delete_acme(token: &str, id: &str) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/acme/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?;
    Ok(())
}
