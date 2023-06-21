#![recursion_limit = "1024"]

use console_error_panic_hook::set_once as set_panic_hook;
use yew::prelude::*;
use yew_router::prelude::*;

mod auth;
mod components;
mod event;
mod pages;
mod store;

use components::navbar::Navbar;

const API_ENDPOINT: &str = "/api";

#[function_component(App)]
pub fn app() -> Html {
    event::use_event_subscriber();
    html! {
        <>
        <BrowserRouter>
            <Navbar />
            <ybc::Columns classes={classes!("is-centered")}>
                <ybc::Column classes={classes!("is-three-fifths-desktop", "m-5")}>
                    <Switch<pages::Route> render={pages::switch} />
                </ybc::Column>
            </ybc::Columns>
        </BrowserRouter>
        </>
    }
}

fn main() {
    set_panic_hook();
    yew::Renderer::<App>::new().render();
}
