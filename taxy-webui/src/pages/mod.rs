use std::borrow::Cow;

use yew::prelude::*;
use yew_router::prelude::*;

mod cert_list;
mod home;
mod login;
mod logout;
mod new_port;
mod port_list;
mod port_view;
mod site_list;

#[derive(Clone, Debug, Routable, PartialEq)]
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
    #[at("/sites/:id")]
    SiteView { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

impl Route {
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
        Route::SiteView { id } => html! { <port_view::PortView {id} /> },
        Route::Certs => html! { <cert_list::CertList /> },
        Route::NotFound => html! { <Redirect<Route> to={Route::Home}/> },
    }
}
