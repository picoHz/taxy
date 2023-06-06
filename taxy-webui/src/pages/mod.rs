use std::borrow::Cow;

use yew::prelude::*;
use yew_router::prelude::*;

mod home;
mod login;
mod logout;
mod port_list;
mod port_view;

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
    #[at("/sites")]
    Sites,
    #[at("/certs")]
    Certs,
    #[at("/ports/:id")]
    PortView { id: String },
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
            Route::Sites => vec![BreadcrumbItem {
                name: "Sites".into(),
                route: Route::Sites,
            }],
            Route::Certs => vec![BreadcrumbItem {
                name: "Certs".into(),
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
        Route::PortView { id } => html! { <port_view::PortView {id} /> },
        Route::Sites => html! { <port_list::PortList /> },
        Route::Certs => html! { <port_list::PortList /> },
        Route::NotFound => html! { <Redirect<Route> to={Route::Home}/> },
    }
}
