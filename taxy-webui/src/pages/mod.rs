use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use yew::prelude::*;
use yew_router::prelude::*;

mod cert_list;
mod home;
mod login;
mod logout;
mod new_acme;
mod new_port;
mod new_site;
mod port_list;
mod port_view;
mod self_sign;
mod site_list;
mod site_view;
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
    PortView { id: String },
    #[at("/sites")]
    Sites,
    #[at("/certs")]
    Certs,
    #[at("/certs/self_sign")]
    SelfSign,
    #[at("/certs/upload")]
    Upload,
    #[at("/certs/new_acme")]
    NewAcme,
    #[at("/sites/new")]
    NewSite,
    #[at("/sites/:id")]
    SiteView { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

impl Route {
    pub fn root(&self) -> Option<Route> {
        match self {
            Route::Ports | Route::NewPort | Route::PortView { .. } => Some(Route::Ports),
            Route::Certs | Route::SelfSign | Route::Upload | Route::NewAcme => Some(Route::Certs),
            Route::Sites | Route::NewSite | Route::SiteView { .. } => Some(Route::Sites),
            _ => None,
        }
    }

    pub fn breadcrumb(&self) -> Vec<BreadcrumbItem> {
        match self {
            Route::Home => vec![BreadcrumbItem {
                name: "Home".into(),
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
            Route::Sites => vec![BreadcrumbItem {
                name: "Sites".into(),
                route: Route::Sites,
            }],
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
            Route::PortView { id } => vec![
                BreadcrumbItem {
                    name: "Ports".into(),
                    route: Route::Ports,
                },
                BreadcrumbItem {
                    name: id.clone().into(),
                    route: Route::PortView { id: id.clone() },
                },
            ],
            Route::NewSite => vec![
                BreadcrumbItem {
                    name: "Sites".into(),
                    route: Route::Sites,
                },
                BreadcrumbItem {
                    name: "New Site".into(),
                    route: Route::NewSite,
                },
            ],
            Route::SiteView { id } => vec![
                BreadcrumbItem {
                    name: "Sites".into(),
                    route: Route::Sites,
                },
                BreadcrumbItem {
                    name: id.clone().into(),
                    route: Route::SiteView { id: id.clone() },
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
        Route::Home => html! { <home::Home /> },
        Route::Login => html! { <login::Login /> },
        Route::Logout => html! { <logout::Logout /> },
        Route::Ports => html! { <port_list::PortList /> },
        Route::NewPort => html! { <new_port::NewPort /> },
        Route::PortView { id } => html! { <port_view::PortView {id} /> },
        Route::Sites => html! { <site_list::SiteList /> },
        Route::SiteView { id } => html! { <site_view::SiteView {id} /> },
        Route::NewSite => html! { <new_site::NewSite /> },
        Route::Certs => html! { <cert_list::CertList /> },
        Route::SelfSign => html! { <self_sign::SelfSign /> },
        Route::NewAcme => html! { <new_acme::NewAcme /> },
        Route::Upload => html! { <upload::Upload /> },
        Route::NotFound => html! { <Redirect<Route> to={Route::Home}/> },
    }
}
