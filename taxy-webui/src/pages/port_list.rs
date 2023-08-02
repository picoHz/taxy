use crate::auth::use_ensure_auth;
use crate::pages::Route;
use crate::store::PortStore;
use crate::utils::convert_multiaddr;
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::{
    id::ShortId,
    port::{PortEntry, PortStatus, SocketState},
};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(PortList)]
pub fn post_list() -> Html {
    use_ensure_auth();

    let (ports, dispatcher) = use_store::<PortStore>();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_list().await {
                    let mut statuses = HashMap::new();
                    for entry in &res {
                        if let Ok(status) = get_status(entry.id).await {
                            statuses.insert(entry.id, status);
                        }
                    }
                    dispatcher.set(PortStore {
                        entries: res,
                        statuses,
                    });
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let new_port_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::NewPort);
    });

    let list = ports.entries.clone();
    html! {
        <>
            <div class="relative overflow-x-auto bg-white shadow-sm border border-neutral-300 md:rounded-md">
                if list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold text-neutral-500 px-16 text-center">{"List is empty. Click 'Add' to configure a new port."}</p>
                } else {
                <table class="w-full text-sm text-left text-neutral-600 rounded-md">
                    <thead class="text-xs text-neutral-800 uppercase border-b border-neutral-300">
                        <tr>
                            <th scope="col" class="px-4 py-3">
                                {"Name"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Protocol"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Address"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Status"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                <span class="sr-only">{"Edit"}</span>
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        { list.into_iter().map(|entry| {
                            let title = if entry.port.name.is_empty() {
                                entry.id.to_string()
                            } else {
                                entry.port.name.clone()
                            };
                            let (protocol, addr) = convert_multiaddr(&entry.port.listen);
                            let status = ports.statuses.get(&entry.id).cloned().unwrap_or_default();
                            let (status_text, tag) = match status.state.socket {
                                SocketState::Listening => ("Listening", "bg-green-500"),
                                SocketState::AddressAlreadyInUse => ("Address In Use", "bg-red-500"),
                                SocketState::PermissionDenied => ("Permission Denied", "bg-red-500"),
                                SocketState::AddressNotAvailable => ("Address Unavailable", "bg-red-500"),
                                SocketState::Error => ("Error", "bg-red-500"),
                                SocketState::Unknown => ("Unknown", "bg-neutral-500"),
                            };

                            let id = entry.id;
                            let navigator_cloned = navigator.clone();
                            let config_onclick = Callback::from(move |_|  {
                                navigator_cloned.push(&Route::PortView {id});
                            });

                            let navigator_cloned = navigator.clone();
                            let log_onclick = Callback::from(move |_|  {
                                navigator_cloned.push(&Route::PortLogView {id});
                            });

                            let reset_onclick = Callback::from(move |e: MouseEvent|  {
                                e.prevent_default();
                                if gloo_dialogs::confirm(&format!("Are you sure to reset {id}?\nThis operation closes all existing connections. ")) {
                                    wasm_bindgen_futures::spawn_local(async move {
                                        let _ = reset_port(id).await;
                                    });
                                }
                            });

                            let delete_onclick = Callback::from(move |e: MouseEvent|  {
                                e.prevent_default();
                                if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                                    wasm_bindgen_futures::spawn_local(async move {
                                        let _ = delete_port(id).await;
                                    });
                                }
                            });

                            html! {
                                <tr class="border-b">
                                    <th scope="row" class="px-4 py-4 font-medium text-neutral-900 whitespace-nowrap">
                                        {title}
                                    </th>
                                    <td class="px-4 py-4">
                                        {protocol}
                                    </td>
                                    <td class="px-4 py-4">
                                        {addr}
                                    </td>
                                    <td class="px-4 py-4">
                                        <div class="flex items-center">
                                            <div class={classes!("h-2.5", "w-2.5", "rounded-full", "bg-green-500", "mr-2", tag)}></div> {status_text}
                                        </div>
                                    </td>
                                    <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
                                        <a class="font-medium text-blue-600 hover:underline mr-5" onclick={config_onclick}>{"Edit"}</a>
                                        <a class="font-medium text-blue-600 hover:underline mr-5" onclick={log_onclick}>{"Log"}</a>
                                        <a class="font-medium text-orange-600 hover:underline mr-5" onclick={reset_onclick}>{"Reset"}</a>
                                        <a class="font-medium text-red-600 hover:underline" onclick={delete_onclick}>{"Delete"}</a>
                                    </td>
                                </tr>
                            }
                        }).collect::<Html>() }
                    </tbody>
                </table>
                }
            </div>
            <div class="flex items-center justify-end my-4 px-4 md:px-0">
                <div>
                    <button onclick={new_port_onclick} class="inline-flex items-center text-neutral-500 bg-white border border-neutral-300 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2" type="button">
                        <img src="/assets/icons/add.svg" class="w-4 h-4 mr-1" />
                        {"Add"}
                    </button>
                </div>
            </div>
        </>
    }
}

async fn get_list() -> Result<Vec<PortEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports"))
        .send()
        .await?
        .json()
        .await
}

async fn get_status(id: ShortId) -> Result<PortStatus, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports/{id}/status"))
        .send()
        .await?
        .json()
        .await
}

async fn delete_port(id: ShortId) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/ports/{id}"))
        .send()
        .await?;
    Ok(())
}

async fn reset_port(id: ShortId) -> Result<(), gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports/{id}/reset"))
        .send()
        .await?;
    Ok(())
}
