use crate::{
    auth::use_ensure_auth, components::breadcrumb::Breadcrumb, pages::Route, store::SessionStore,
    API_ENDPOINT,
};
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::{cert::SelfSignedCertRequest, port::Port, subject_name::SubjectName};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(SelfSign)]
pub fn self_sign() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();
    let (session, _) = use_store::<SessionStore>();
    let token = session.token.clone();

    let entry = use_state::<Result<Port, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry_cloned = entry.clone();
    let on_changed: Callback<Result<Port, HashMap<String, String>>> =
        Callback::from(move |updated| {
            gloo_console::log!(&format!("updated: {:?}", updated));
            entry_cloned.set(updated);
        });

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Ports);
    });

    html! {
        <>
            <ybc::Card classes="py-5">
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>


            <div class="field is-grouped is-grouped-right mx-5">
                <p class="control">
                    <button class="button is-light" onclick={cancel_onclick}>
                    {"Cancel"}
                    </button>
                </p>
                <p class="control">
                    <button class="button is-primary" disabled={entry.is_err()}>
                    {"Sign"}
                    </button>
                </p>
            </div>
            </ybc::Card>
        </>
    }
}

async fn self_sign(token: &str, san: &[SubjectName]) -> Result<(), gloo_net::Error> {
    Request::post(&format!("{API_ENDPOINT}/server_certs/self_sign"))
        .header("Authorization", &format!("Bearer {token}"))
        .json(&SelfSignedCertRequest { san: san.to_vec() })?
        .send()
        .await?
        .json()
        .await
}
