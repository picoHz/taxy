use crate::components::breadcrumb::Breadcrumb;
use crate::store::CertStore;
use crate::API_ENDPOINT;
use crate::{auth::use_ensure_auth, store::SessionStore};
use gloo_net::http::Request;
use taxy_api::cert::CertInfo;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    ServerCerts,
    Acme,
}

impl ToString for Tab {
    fn to_string(&self) -> String {
        match self {
            Tab::ServerCerts => "Server Certificates",
            Tab::Acme => "ACME",
        }
        .into()
    }
}

const TABS: [Tab; 2] = [Tab::ServerCerts, Tab::Acme];

#[function_component(CertList)]
pub fn cert_list() -> Html {
    use_ensure_auth();

    let tab = use_state(|| Tab::ServerCerts);

    let (session, _) = use_store::<SessionStore>();
    let (certs, dispatcher) = use_store::<CertStore>();
    let token = session.token.clone();
    use_effect_with_deps(
        move |_| {
            if let Some(token) = token {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(res) = get_cert_list(&token).await {
                        dispatcher.set(CertStore { entries: res });
                    }
                });
            }
        },
        session,
    );

    let list = certs.entries.clone();
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
                        let is_active = item == *tab;
                        let tab = tab.clone();
                        let onclick = Callback::from(move |_|  {
                            tab.set(item);
                        });
                        html! {
                            <li class={classes!(is_active.then_some("is-active"))}>
                                <a {onclick}>{item}</a>
                            </li>
                        }
                    }).collect::<Html>() }

                </ul>
            </div>
            if *tab == Tab::ServerCerts {
            <div class="list has-visible-pointer-controls">
            { list.into_iter().enumerate().map(|(i, entry)| {
                let subject_names = entry
                    .san
                    .iter()
                    .map(|name| name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let issuer = entry.issuer.to_string();
                let title = format!("{} ({})", subject_names, issuer);

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
                        <ion-icon name="create"></ion-icon>
                    </span>
                    <span>{"Self-sign"}</span>
                    </span>
                </a>
                <a href="#" class="card-footer-item">
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="cloud-upload"></ion-icon>
                    </span>
                    <span>{"Upload"}</span>
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