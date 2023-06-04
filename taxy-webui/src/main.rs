#![recursion_limit = "1024"]

use console_error_panic_hook::set_once as set_panic_hook;
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

mod auth;
mod navbar;
mod pages;

#[cfg(debug_assertions)]
const API_ENDPOINT: &str = "http://127.0.0.1:46492/";
#[cfg(not(debug_assertions))]
const API_ENDPOINT: &str = "/";

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
#[store(storage = "local")]
struct UserSession {
    token: Option<String>,
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <>
        <BrowserRouter>
            <navbar::Navbar />
            <Switch<pages::Route> render={pages::switch} />
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
