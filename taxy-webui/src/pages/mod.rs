use serde_derive::{Deserialize, Serialize};
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
