use yew::prelude::*;
use yew_router::prelude::*;

mod home;
mod login;
mod logout;

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
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <home::Home /> },
        Route::Login => html! { <login::Login /> },
        Route::Logout => html! { <logout::Logout /> },
        Route::Ports => html! { <login::Login /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}
