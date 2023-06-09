use std::collections::HashMap;

use crate::{
    auth::use_ensure_auth,
    components::{breadcrumb::Breadcrumb, port_config::PortConfig},
    pages::Route,
};
use taxy_api::port::Port;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(NewPort)]
pub fn new_port() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator;
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Ports);
    });

    let entry = use_state::<Result<Port, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let on_changed: Callback<Result<Port, HashMap<String, String>>> =
        Callback::from(move |updated| {
            gloo_console::log!(&format!("updated: {:?}", updated));
            entry_cloned.set(updated);
        });

    html! {
        <>
            <ybc::Card classes="py-5">
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>

            <PortConfig {on_changed} />

            <div class="field is-grouped is-grouped-right mx-5">
                <p class="control">
                    <button class="button is-light" onclick={cancel_onclick}>
                    {"Cancel"}
                    </button>
                </p>
                <p class="control">
                    <button class="button is-primary" disabled={entry.is_err()}>
                    {"Create"}
                    </button>
                </p>
            </div>
            </ybc::Card>
        </>
    }
}
