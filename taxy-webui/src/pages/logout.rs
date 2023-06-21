use crate::{pages::Route, API_ENDPOINT};
use gloo_net::http::Request;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Logout)]
pub fn logout() -> Html {
    wasm_bindgen_futures::spawn_local(async move {
        Request::get(&format!("{API_ENDPOINT}/logout"))
            .send()
            .await
            .unwrap();
    });

    html! {
        <Redirect<Route> to={Route::Login}/>
    }
}
