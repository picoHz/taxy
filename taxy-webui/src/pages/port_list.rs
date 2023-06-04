use crate::auth::{use_ensure_auth, UserSession};
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use taxy_api::port::PortEntry;
use yew::prelude::*;
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

    html! {
        <ul class="item-list">
            { list.iter().map(|entry| {
                html! {
                    <li>
                        <span>{ &entry.id }</span>
                    </li>
                }
            }).collect::<Html>() }
        </ul>
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
