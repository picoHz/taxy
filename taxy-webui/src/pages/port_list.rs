use crate::pages::Route;
use crate::API_ENDPOINT;
use crate::{auth::use_ensure_auth, store::UserSession};
use gloo_net::http::Request;
use taxy_api::port::PortEntry;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(PortList)]
pub fn port_view() -> Html {
    let list = use_state::<Vec<PortEntry>, _>(Vec::new);
    use_ensure_auth();

    let (session, _) = use_store::<UserSession>();
    let token = session.token.clone();
    let list_cloned = list.clone();
    use_effect_with_deps(
        move |_| {
            if let Some(token) = token {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(res) = get_list(&token).await {
                        list_cloned.set(res);
                    }
                });
            }
        },
        session.clone(),
    );

    let navigator = use_navigator().unwrap();
    let list = list.to_vec();
    html! {
        <ybc::Columns classes={classes!("is-centered", "m-5")}>
            <ybc::Column classes={classes!("is-three-fifths-desktop")}>
                <ybc::Panel heading={html!("Ports")}>
                    { list.into_iter().map(|entry| {
                        let navigator = navigator.clone();
                        let onclick = Callback::from(move |_|  {
                            navigator.push(&Route::PortView {id: entry.id.clone()});
                        });
                        html! {
                            <a class="panel-block" {onclick}>{&entry.port.listen}</a>
                        }
                    }).collect::<Html>() }
                    <div class="panel-block">
                        <button class="button is-link is-outlined is-fullwidth">
                        {"Add Port"}
                        </button>
                    </div>
                </ybc::Panel>
            </ybc::Column>
        </ybc::Columns>
    }
}

async fn get_list(token: &str) -> Result<Vec<PortEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?
        .json()
        .await
}
