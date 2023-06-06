use crate::pages::Route;
use crate::store::PortStore;
use crate::API_ENDPOINT;
use crate::{auth::use_ensure_auth, store::SessionStore};
use gloo_net::http::Request;
use taxy_api::port::PortEntry;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(PortList)]
pub fn port_view() -> Html {
    use_ensure_auth();

    let (session, _) = use_store::<SessionStore>();
    let (ports, dispatcher) = use_store::<PortStore>();
    let token = session.token.clone();
    use_effect_with_deps(
        move |_| {
            if let Some(token) = token {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(res) = get_list(&token).await {
                        dispatcher.set(PortStore { entries: res });
                    }
                });
            }
        },
        session,
    );

    let navigator = use_navigator().unwrap();
    let list = ports.entries.clone();
    html! {
        <ybc::Columns classes={classes!("is-centered")}>
            <ybc::Column classes={classes!("is-three-fifths-desktop")}>
                <div class="list has-visible-pointer-controls">
                { list.into_iter().map(|entry| {
                    let navigator = navigator.clone();
                    let id = entry.id.clone();
                    let onclick = Callback::from(move |_|  {
                        let id = id.clone();
                        navigator.push(&Route::PortView {id});
                    });
                    html! {
                        <div class="list-item">
                            <div class="list-item-content">
                                <div class="list-item-title">{&entry.port.listen}</div>
                                <div class="list-item-description">{&entry.id}</div>
                            </div>
                    
                            <div class="list-item-controls">
                                <div class="buttons is-right">
                                    <button class="button" {onclick}>
                                        <span class="icon is-small">
                                            <ion-icon name="settings"></ion-icon>
                                        </span>
                                        <span>{"Config"}</span>
                                    </button>
                            
                                    <button class="button">
                                        <span class="icon is-small">
                                            <ion-icon name="ellipsis-horizontal"></ion-icon>
                                        </span>
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                }).collect::<Html>() }
                </div>
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
