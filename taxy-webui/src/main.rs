#![recursion_limit = "1024"]

use console_error_panic_hook::set_once as set_panic_hook;
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

mod auth;
mod home;
mod login;
mod logout;

#[cfg(debug_assertions)]
const API_ENDPOINT: &str = "http://127.0.0.1:46492/";
#[cfg(not(debug_assertions))]
const API_ENDPOINT: &str = "/";

#[derive(Clone, Debug, Routable, PartialEq)]
enum Route {
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

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
struct UserSession {
    token: Option<String>,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <home::Home /> },
        Route::Login => html! { <login::Login /> },
        Route::Logout => html! { <logout::Logout /> },
        Route::Ports => html! { <login::Login /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    let (counter, _) = use_store::<UserSession>();
    html! {
        <>
        <BrowserRouter>
            <ybc::Navbar
                classes={classes!("is-success")}
                padded=true
                navbrand={html!{
                    <ybc::NavbarItem>
                        <ybc::Title classes={classes!("has-text-white")} size={ybc::HeaderSize::Is4}>{"Trunk | Yew | YBC"}</ybc::Title>
                    </ybc::NavbarItem>
                }}
                navstart={html!{}}
                navend={html!{
                    <>
                    <ybc::NavbarItem>
                        <ybc::ButtonAnchor classes={classes!("is-inverted")} rel={String::from("noopener noreferrer")} target={String::from("_blank")}>
                            {"Trunk"}
                        </ybc::ButtonAnchor>
                    </ybc::NavbarItem>
                    <ybc::NavbarItem>
                        <ybc::ButtonAnchor classes={classes!("is-inverted")} rel={String::from("noopener noreferrer")} target={String::from("_blank")} href="https://yew.rs">
                            {"Yew"}
                        </ybc::ButtonAnchor>
                    </ybc::NavbarItem>
                    <ybc::NavbarItem>
                        <ybc::ButtonAnchor classes={classes!("is-inverted")} rel={String::from("noopener noreferrer")} target={String::from("_blank")} href="https://github.com/thedodd/ybc">
                            {counter.token.clone()}
                        </ybc::ButtonAnchor>
                    </ybc::NavbarItem>
                    </>
                }}
            />
            <Switch<Route> render={switch} />
        </BrowserRouter>
        </>
    }
}

#[wasm_bindgen(inline_js = "export function snippetTest() { console.log('Hello from JS FFI!'); }")]
extern "C" {
    fn snippetTest();
}

fn main() {
    set_panic_hook();
    snippetTest();

    yew::Renderer::<App>::new().render();
}
