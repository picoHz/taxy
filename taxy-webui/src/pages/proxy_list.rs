use crate::auth::use_ensure_auth;
use crate::pages::Route;
use crate::store::{PortStore, ProxyStore};
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use taxy_api::id::ShortId;
use taxy_api::port::PortEntry;
use taxy_api::site::ProxyEntry;
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
                    proxies_dispatcher.set(ProxyStore { entries: res });
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
            <div class="relative overflow-x-auto bg-white shadow-sm border border-neutral-300 lg:rounded-md">
                if list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold text-neutral-500 px-16 text-center">{"List is empty. Click 'Add' to configure a new proxy."}</p>
                } else {
                <table class="w-full text-sm text-left text-neutral-600 rounded-md">
                    <thead class="text-xs text-neutral-800 uppercase border-b border-neutral-300">
                        <tr>
                            <th scope="col" class="px-4 py-3">
                                {"Name"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Ports"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Active"}
                            </th>
                            <th scope="col" class="px-4 py-3">
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

                        html! {
                            <tr class="border-b">
                                <th scope="row" class="px-4 py-4 font-medium text-neutral-900 whitespace-nowrap">
                                    {title}
                                </th>
                                <td class="px-4 py-4">
                                    {ports}
                                </td>
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
                                    <label class="relative inline-flex items-center cursor-pointer mt-1">
                                        <input {onchange} type="checkbox" checked={active} class="sr-only peer" />
                                        <div class="w-9 h-4 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-3 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </td>
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
                                    <a class="cursor-pointer font-medium text-blue-600 hover:underline mr-5" onclick={config_onclick}>{"Edit"}</a>
                                    <a class="cursor-pointer font-medium text-blue-600 hover:underline mr-5" onclick={log_onclick}>{"Log"}</a>
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
                    <button onclick={new_proxy_onclick} class="inline-flex items-center text-neutral-500 bg-white border border-neutral-300 focus:outline-none hover:bg-neutral-100 focus:ring-4 focus:ring-neutral-200 font-medium rounded-lg text-sm px-4 py-2" type="button">
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
