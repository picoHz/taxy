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
    #[at("/ports/:id")]
    PortView { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <home::Home /> },
        Route::Login => html! { <login::Login /> },
        Route::Logout => html! { <logout::Logout /> },
        Route::Ports => html! { <port_list::PortList /> },
        Route::PortView { id } => html! { <port_view::PortView {id} /> },
        Route::NotFound => html! { <Redirect<Route> to={Route::Home}/> },
    }
}
