#![recursion_limit = "1024"]

use components::navbar::Navbar;
use console_error_panic_hook::set_once as set_panic_hook;
use yew::prelude::*;
use yew_router::prelude::*;

mod auth;
mod components;
mod event;
mod pages;
mod store;
pub mod utils;

const API_ENDPOINT: &str = "/api";

#[function_component(App)]
pub fn app() -> Html {
    event::use_event_subscriber();
    html! {
        <>
        <BrowserRouter>
            <Navbar />
            <div class="container px-5 py-10 mx-auto">
                <Switch<pages::Route> render={pages::switch} />
            </div>
        </BrowserRouter>
        </>
    }
}

fn main() {
    set_panic_hook();
    yew::Renderer::<App>::new().render();
}
