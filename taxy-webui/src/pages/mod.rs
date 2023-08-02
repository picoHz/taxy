use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use taxy_api::id::ShortId;
use yew::prelude::*;
use yew_router::prelude::*;

mod cert_list;
mod log_view;
mod login;
mod logout;
mod new_acme;
mod new_port;
mod new_site;
mod port_list;
mod port_view;
mod proxy_list;
mod proxy_view;
mod self_sign;
mod upload;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Routable)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/logout")]
    Logout,
    #[at("/ports")]
    Ports,
    #[at("/ports/new")]
    NewPort,
    #[at("/ports/:id")]
    PortView { id: ShortId },
    #[at("/ports/:id/log")]
    PortLogView { id: ShortId },
    #[at("/proxies")]
    Proxies,
    #[at("/proxies/:id/log")]
    ProxyLogView { id: ShortId },
    #[at("/certs")]
    Certs,
    #[at("/certs/self_sign")]
    SelfSign,
    #[at("/certs/upload")]
    Upload,
    #[at("/certs/new_acme")]
    NewAcme,
    #[at("/certs/:id/log")]
    CertLogView { id: String },
    #[at("/proxies/new")]
    NewProxy,
    #[at("/proxies/:id")]
    ProxyView { id: ShortId },
    #[not_found]
    #[at("/404")]
    NotFound,
}

impl Route {
    pub fn root(&self) -> Option<Route> {
        match self {
            Route::Home => Some(Route::Home),
            Route::Ports | Route::NewPort | Route::PortView { .. } | Route::PortLogView { .. } => {
                Some(Route::Ports)
            }
            Route::Certs
            | Route::SelfSign
            | Route::Upload
            | Route::NewAcme
            | Route::CertLogView { .. } => Some(Route::Certs),
            Route::Proxies
            | Route::NewProxy
            | Route::ProxyView { .. }
            | Route::ProxyLogView { .. } => Some(Route::Proxies),
            _ => None,
        }
    }

    pub fn breadcrumb(&self) -> Vec<BreadcrumbItem> {
        match self {
            Route::Home => vec![BreadcrumbItem {
                name: "Dashboard".into(),
                route: Route::Home,
            }],
            Route::Login => vec![BreadcrumbItem {
                name: "Login".into(),
                route: Route::Login,
            }],
            Route::Logout => vec![BreadcrumbItem {
                name: "Logout".into(),
                route: Route::Logout,
            }],
            Route::Ports => vec![BreadcrumbItem {
                name: "Ports".into(),
                route: Route::Ports,
            }],
            Route::NewPort => vec![
                BreadcrumbItem {
                    name: "Ports".into(),
                    route: Route::Ports,
                },
                BreadcrumbItem {
                    name: "New Port".into(),
                    route: Route::NewPort,
                },
            ],
            Route::Proxies => vec![BreadcrumbItem {
                name: "Proxies".into(),
                route: Route::Proxies,
            }],
            Route::ProxyLogView { id } => vec![
                BreadcrumbItem {
                    name: "Proxies".into(),
                    route: Route::Proxies,
                },
                BreadcrumbItem {
                    name: id.to_string().into(),
                    route: Route::ProxyView { id: *id },
                },
                BreadcrumbItem {
                    name: "Log".into(),
                    route: Route::ProxyLogView { id: *id },
                },
            ],
            Route::Certs => vec![BreadcrumbItem {
                name: "Certificates".into(),
                route: Route::Certs,
            }],
            Route::SelfSign => vec![
                BreadcrumbItem {
                    name: "Certificates".into(),
                    route: Route::Certs,
                },
                BreadcrumbItem {
                    name: "Self Sign".into(),
                    route: Route::SelfSign,
                },
            ],
            Route::Upload => vec![
                BreadcrumbItem {
                    name: "Certificates".into(),
                    route: Route::Certs,
                },
                BreadcrumbItem {
                    name: "Upload".into(),
                    route: Route::Upload,
                },
            ],
            Route::NewAcme => vec![
                BreadcrumbItem {
                    name: "Certificates".into(),
                    route: Route::Certs,
                },
                BreadcrumbItem {
                    name: "New Acme".into(),
                    route: Route::NewAcme,
                },
            ],
            Route::CertLogView { id } => vec![
                BreadcrumbItem {
                    name: "Certificates".into(),
                    route: Route::Certs,
                },
                BreadcrumbItem {
                    name: id.clone().into(),
                    route: Route::CertLogView { id: id.clone() },
                },
            ],
            Route::PortView { id } => vec![
                BreadcrumbItem {
                    name: "Ports".into(),
                    route: Route::Ports,
                },
                BreadcrumbItem {
                    name: id.to_string().into(),
                    route: Route::PortView { id: *id },
                },
            ],
            Route::PortLogView { id } => vec![
                BreadcrumbItem {
                    name: "Ports".into(),
                    route: Route::Ports,
                },
                BreadcrumbItem {
                    name: id.to_string().into(),
                    route: Route::PortView { id: *id },
                },
                BreadcrumbItem {
                    name: "Log".into(),
                    route: Route::PortLogView { id: *id },
                },
            ],
            Route::NewProxy => vec![
                BreadcrumbItem {
                    name: "Proxies".into(),
                    route: Route::Proxies,
                },
                BreadcrumbItem {
                    name: "New Proxy".into(),
                    route: Route::NewProxy,
                },
            ],
            Route::ProxyView { id } => vec![
                BreadcrumbItem {
                    name: "Proxies".into(),
                    route: Route::Proxies,
                },
                BreadcrumbItem {
                    name: id.to_string().into(),
                    route: Route::ProxyView { id: *id },
                },
            ],
            Route::NotFound => vec![],
        }
    }
}

pub struct BreadcrumbItem {
    pub name: Cow<'static, str>,
    pub route: Route,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Redirect<Route> to={Route::Ports}/> },
        Route::Login => html! { <login::Login /> },
        Route::Logout => html! { <logout::Logout /> },
        Route::Ports => html! { <port_list::PortList /> },
        Route::NewPort => html! { <new_port::NewPort /> },
        Route::PortView { id } => html! { <port_view::PortView {id} /> },
        Route::PortLogView { id } => html! { <log_view::LogView id={id.to_string()} /> },
        Route::Proxies => html! { <proxy_list::ProxyList /> },
        Route::ProxyLogView { id } => html! { <log_view::LogView id={id.to_string()} /> },
        Route::ProxyView { id } => html! { <proxy_view::ProxyView {id} /> },
        Route::NewProxy => html! { <new_site::NewProxy /> },
        Route::Certs => html! { <cert_list::CertList /> },
        Route::SelfSign => html! { <self_sign::SelfSign /> },
        Route::NewAcme => html! { <new_acme::NewAcme /> },
        Route::CertLogView { id } => html! { <log_view::LogView {id} /> },
        Route::Upload => html! { <upload::Upload /> },
        Route::NotFound => html! { <Redirect<Route> to={Route::Home}/> },
    }
}
