use std::collections::HashMap;

use crate::auth::use_ensure_auth;
use crate::pages::Route;
use crate::store::{PortStore, ProxyStore};
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use taxy_api::id::ShortId;
use taxy_api::port::PortEntry;
use taxy_api::proxy::{ProxyEntry, ProxyState, ProxyStatus};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(ProxyList)]
pub fn proxy_list() -> Html {
    use_ensure_auth();

    let (ports, ports_dispatcher) = use_store::<PortStore>();
    let (proxies, proxies_dispatcher) = use_store::<ProxyStore>();

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
                    proxies_dispatcher.set(ProxyStore {
                        entries: res,
                        statuses,
                        loaded: true,
                    });
                }
            });
        },
        (),
    );

    let ports_cloned = ports.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_ports().await {
                    ports_dispatcher.set(PortStore {
                        entries: res,
                        loaded: true,
                        ..(*ports_cloned).clone()
                    });
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();
    let list = proxies.entries.clone();

    let navigator_cloned = navigator.clone();
    let new_proxy_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::NewProxy);
    });

    html! {
        <>
            <div class="relative overflow-x-auto bg-white dark:bg-neutral-800 shadow-sm border border-neutral-300 dark:border-neutral-700 lg:rounded-md">
                if !proxies.loaded {
                    <svg aria-hidden="true" role="status" class="w-8 h-8 mx-auto my-7 text-neutral-200 animate-spin" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="#ccc"/>
                    <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="#888"/>
                    </svg>
                } else if list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold dark:text-neutral-300 text-neutral-500 px-16 text-center">{"List is empty. Click 'Add' to configure a new proxy."}</p>
                } else {
                <table class="w-full text-sm text-left text-neutral-600 dark:text-neutral-200 rounded-md">
                    <thead class="text-xs text-neutral-800 dark:text-neutral-200 uppercase border-b border-neutral-300 dark:border-neutral-700">
                        <tr>
                            <th scope="col" class="px-4 py-3">
                                {"Name"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Ports"}
                            </th>
                            <th scope="col" class="px-4 py-3 w-48">
                                {"Status"}
                            </th>
                            <th scope="col" class="px-4 py-3" align="center">
                                {"Active"}
                            </th>
                            <th scope="col" class="px-4 py-3" align="right">
                                <span class="sr-only">{"Edit"}</span>
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                    { list.into_iter().map(|entry| {
                        let navigator = navigator.clone();

                        let id = entry.id;
                        let navigator_cloned = navigator.clone();
                        let log_onclick = Callback::from(move |_|  {
                            navigator_cloned.push(&Route::ProxyLogView {id});
                        });

                        let config_onclick = Callback::from(move |_|  {
                            navigator.push(&Route::ProxyView {id});
                        });

                        let delete_onclick = Callback::from(move |e: MouseEvent|  {
                            e.prevent_default();
                            if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                                wasm_bindgen_futures::spawn_local(async move {
                                    let _ = delete_site(id).await;
                                });
                            }
                        });

                        let active = entry.proxy.active;
                        let onchange = Callback::from(move |_: Event| {
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = toggle_proxy(id).await;
                            });
                        });

                        let ports = entry.proxy.ports.iter().filter_map(|port| {
                            ports.entries.iter().find(|p| p.id == *port)
                        }).map(|entry| {
                            format!("{}/{}", entry.port.listen.protocol_name(), entry.port.listen.socket_addr().unwrap())
                        }).collect::<Vec<_>>();
                        let ports = ports.join(", ");

                        let title = if entry.proxy.name.is_empty() {
                            entry.id.to_string()
                        } else {
                            entry.proxy.name.clone()
                        };

                        let status = proxies.statuses.get(&entry.id).cloned().unwrap_or_default();
                        let (status_text, tag) = match status.state {
                            ProxyState::Active => ("Active", "bg-green-500"),
                            ProxyState::Inactive => ("Inactive", "bg-neutral-500"),
                            ProxyState::Unknown => ("Unknown", "bg-neutral-500"),
                        };

                        html! {
                            <tr class="border-b dark:border-neutral-700">
                                <th scope="row" class="px-4 py-4 font-medium text-neutral-900 dark:text-neutral-200 whitespace-nowrap">
                                    {title}
                                </th>
                                <td class="px-4 py-4">
                                    {ports}
                                </td>
                                <td class="px-4 py-4">
                                    <div class="flex items-center">
                                        <div class={classes!("h-2.5", "w-2.5", "shrink-0", "rounded-full", "bg-green-500", "mr-2", tag)}></div> {status_text}
                                    </div>
                                </td>
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="center">
                                    <label class="relative inline-flex items-center cursor-pointer mt-1">
                                        <input {onchange} type="checkbox" checked={active} class="sr-only peer" />
                                        <div class="w-9 h-4 bg-neutral-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-neutral-300 after:border after:rounded-full after:h-3 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </td>
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
                                    <a class="cursor-pointer font-medium text-blue-600 dark:text-blue-400 hover:underline mr-5" onclick={config_onclick}>{"Edit"}</a>
                                    <a class="cursor-pointer font-medium text-blue-600 dark:text-blue-400 hover:underline mr-5" onclick={log_onclick}>{"Log"}</a>
                                    <a class="cursor-pointer font-medium text-red-600 hover:underline" onclick={delete_onclick}>{"Delete"}</a>
                                </td>
                            </tr>
                        }
                    }).collect::<Html>() }
                    </tbody>
                </table>
            }
            </div>
            <div class="flex items-center justify-end my-4 px-4 lg:px-0">
                <div>
                    <button onclick={new_proxy_onclick} class="inline-flex items-center text-neutral-500 dark:text-neutral-200 bg-white dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 focus:outline-none hover:bg-neutral-100 hover:dark:bg-neutral-900 focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-600 font-medium rounded-lg text-sm px-4 py-2" type="button">
                        <img src="/assets/icons/add.svg" class="w-4 h-4 mr-1" />
                        {"Add"}
                    </button>
                </div>
            </div>
        </>
    }
}

async fn get_ports() -> Result<Vec<PortEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports"))
        .send()
        .await?
        .json()
        .await
}

async fn get_list() -> Result<Vec<ProxyEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/proxies"))
        .send()
        .await?
        .json()
        .await
}

async fn get_status(id: ShortId) -> Result<ProxyStatus, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/proxies/{id}/status"))
        .send()
        .await?
        .json()
        .await
}

async fn delete_site(id: ShortId) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/proxies/{id}"))
        .send()
        .await?;
    Ok(())
}

async fn toggle_proxy(id: ShortId) -> Result<(), gloo_net::Error> {
    let mut entry: ProxyEntry = Request::get(&format!("{API_ENDPOINT}/proxies/{id}"))
        .send()
        .await?
        .json()
        .await?;
    entry.proxy.active = !entry.proxy.active;
    Request::put(&format!("{API_ENDPOINT}/proxies/{id}"))
        .json(&entry)?
        .send()
        .await?;
    Ok(())
}
