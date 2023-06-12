use crate::components::breadcrumb::Breadcrumb;
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
pub fn post_list() -> Html {
    use_ensure_auth();

    let (session, _) = use_store::<SessionStore>();
    let (ports, dispatcher) = use_store::<PortStore>();
    let token = session.token.clone();
    let token_cloned = token.clone();
    use_effect_with_deps(
        move |_| {
            if let Some(token) = token_cloned {
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

    let navigator_cloned = navigator.clone();
    let new_port_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::NewPort);
    });

    let list = ports.entries.clone();
    let active_index = use_state(|| -1);
    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>
            <div class="list has-visible-pointer-controls">
            { list.into_iter().enumerate().map(|(i, entry)| {
                let navigator = navigator.clone();
                let active_index = active_index.clone();
                let id = entry.id.clone();
                let config_onclick = Callback::from(move |_|  {
                    let id = id.clone();
                    navigator.push(&Route::PortView {id});
                });

                let delete_onmousedown = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                });
                let token_cloned = token.clone();
                let id = entry.id.clone();
                let delete_onclick = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                    if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                        let id = id.clone();
                        if let Some(token) = token_cloned.clone() {
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = delete_port(&token, &id).await;
                            });
                        }
                    }
                });

                let reset_onmousedown = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                });
                let token_cloned = token.clone();
                let id = entry.id.clone();
                let reset_onclick = Callback::from(move |e: MouseEvent|  {
                    e.prevent_default();
                    if gloo_dialogs::confirm(&format!("Are you sure to reset {id}?\nThis operation closes all existing connections. ")) {
                        let id = id.clone();
                        if let Some(token) = token_cloned.clone() {
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = reset_port(&token, &id).await;
                            });
                        }
                    }
                });

                let active_index_cloned = active_index.clone();
                let dropdown_onclick = Callback::from(move |_|  {
                    active_index_cloned.set(i as i32);
                });
                let active_index_cloned = active_index.clone();
                let dropdown_onfocusout = Callback::from(move |_|  {
                    active_index_cloned.set(-1);
                });
                let is_active = *active_index == i as i32;
                html! {
                    <div class="list-item">
                        <div class="list-item-content">
                            <div class="list-item-title">{&entry.port.listen}</div>
                            <div class="list-item-description">{&entry.id}</div>
                        </div>

                        <div class="list-item-controls">
                            <div class="buttons is-right">
                                <button class="button" onclick={config_onclick}>
                                    <span class="icon is-small">
                                        <ion-icon name="settings"></ion-icon>
                                    </span>
                                    <span>{"Config"}</span>
                                </button>

                                <div class={classes!("dropdown", "is-right", is_active.then_some("is-active"))}>
                                    <div class="dropdown-trigger" onfocusout={dropdown_onfocusout}>
                                        <button class="button" onclick={dropdown_onclick}>
                                            <span class="icon is-small">
                                                <ion-icon name="ellipsis-horizontal"></ion-icon>
                                            </span>
                                        </button>
                                    </div>
                                    <div class="dropdown-menu" id="dropdown-menu" role="menu">
                                        <div class="dropdown-content">
                                            <a class="dropdown-item" onmousedown={reset_onmousedown} onclick={reset_onclick}>
                                                <span class="icon-text">
                                                    <span class="icon">
                                                        <ion-icon name="refresh"></ion-icon>
                                                    </span>
                                                    <span>{"Reset"}</span>
                                                </span>
                                            </a>
                                            <a class="dropdown-item has-text-danger	" onmousedown={delete_onmousedown} onclick={delete_onclick}>
                                                <span class="icon-text">
                                                    <span class="icon">
                                                        <ion-icon name="trash"></ion-icon>
                                                    </span>
                                                    <span>{"Delete"}</span>
                                                </span>
                                            </a>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                }
            }).collect::<Html>() }
            </div>
            <ybc::CardFooter>
                <a class="card-footer-item" onclick={new_port_onclick}>
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="add"></ion-icon>
                    </span>
                    <span>{"New Port"}</span>
                    </span>
                </a>
            </ybc::CardFooter>
            </ybc::Card>
        </>
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

async fn delete_port(token: &str, id: &str) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/ports/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?;
    Ok(())
}

async fn reset_port(token: &str, id: &str) -> Result<(), gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports/{id}/reset"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await?;
    Ok(())
}
