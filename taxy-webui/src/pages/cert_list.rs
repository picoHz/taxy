use crate::auth::use_ensure_auth;
use crate::format::format_duration;
use crate::pages::Route;
use crate::store::{AcmeStore, CertStore};
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use serde_derive::{Deserialize, Serialize};
use taxy_api::acme::AcmeInfo;
use taxy_api::cert::{CertInfo, CertKind, UploadQuery};
use taxy_api::id::ShortId;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CertsTab {
    #[default]
    Server,
    Root,
    Acme,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CertsQuery {
    #[serde(default)]
    pub tab: CertsTab,
}

impl ToString for CertsTab {
    fn to_string(&self) -> String {
        match self {
            CertsTab::Server => "Server Certs",
            CertsTab::Root => "Root Certs",
            CertsTab::Acme => "ACME",
        }
        .into()
    }
}

const TABS: [CertsTab; 3] = [CertsTab::Server, CertsTab::Root, CertsTab::Acme];

#[function_component(CertList)]
pub fn cert_list() -> Html {
    use_ensure_auth();

    let location = use_location().unwrap();
    let query: CertsQuery = location.query().unwrap_or_default();
    let tab = use_state(|| query.tab);

    let (certs, certs_dispatcher) = use_store::<CertStore>();
    let (acme, acme_dispatcher) = use_store::<AcmeStore>();

    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_cert_list().await {
                    certs_dispatcher.set(CertStore {
                        entries: res,
                        loaded: true,
                    });
                }
                if let Ok(res) = get_acme_list().await {
                    acme_dispatcher.set(AcmeStore {
                        entries: res,
                        loaded: true,
                    });
                }
            });
        },
        (),
    );

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let self_sign_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::SelfSign);
    });

    let navigator_cloned = navigator.clone();
    let tab_cloned = tab.clone();
    let upload_onclick = Callback::from(move |_| {
        let _ = navigator_cloned.push_with_query(
            &Route::Upload,
            &UploadQuery {
                kind: if *tab_cloned == CertsTab::Server {
                    CertKind::Server
                } else {
                    CertKind::Root
                },
            },
        );
    });

    let navigator_cloned = navigator.clone();
    let new_acme_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::NewAcme);
    });

    let cert_list = certs
        .entries
        .iter()
        .filter(|cert| {
            cert.kind
                == if *tab == CertsTab::Server {
                    CertKind::Server
                } else {
                    CertKind::Root
                }
        })
        .collect::<Vec<_>>();
    let acme_list = acme.entries.clone();
    let active_index = use_state(|| -1);
    html! {
        <>
        <div class="flex flex-col mb-4 px-4 lg:px-0">
            <div class="text-sm font-medium text-center text-neutral-500 dark:text-neutral-300">
                <ul class="flex justify-center sm:justify-start flex-wrap -mb-px">
                    { TABS.into_iter().map(|item| {
                        let navigator = navigator.clone();
                        let active_index = active_index.clone();
                        let is_active = item == *tab;
                        let tab = tab.clone();
                        let onclick = Callback::from(move |_|  {
                            tab.set(item);
                            active_index.set(-1);
                            let _ = navigator.push_with_query(&Route::Certs, &CertsQuery { tab: item });
                        });
                        let class = if is_active {
                            vec!["text-blue-600", "dark:text-blue-400", "border-blue-600", "dark:border-blue-400", "active"]
                        } else {
                            vec!["border-transparent"]
                        };
                        html! {
                            <li class="mr-2">
                                <a {onclick} class={classes!("inline-block", "cursor-pointer", "px-4", "py-2", "border-b-2", "rounded-t-lg", "hover:border-neutral-300", class)}>{item}</a>
                            </li>
                        }
                    }).collect::<Html>() }

                </ul>
            </div>
        </div>
            if *tab == CertsTab::Server {
                <div class="relative overflow-x-auto bg-white dark:bg-neutral-800 shadow-sm border border-neutral-300 dark:border-neutral-700 lg:rounded-md">
                if !certs.loaded {
                    <svg aria-hidden="true" role="status" class="w-8 h-8 mx-auto my-7 text-neutral-200 animate-spin" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="#ccc"/>
                    <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="#888"/>
                    </svg>
                } else if cert_list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold text-neutral-500 dark:text-neutral-300 px-16 text-center">{"List is empty."}</p>
                } else {
                    <table class="w-full text-sm text-left text-neutral-600 dark:text-neutral-200 rounded-md">
                        <thead class="text-xs text-neutral-800 dark:text-neutral-200 uppercase border-b border-neutral-300 dark:border-neutral-700">
                            <tr>
                                <th scope="col" class="px-4 py-3">
                                    {"Subject Names"}
                                </th>
                                <th scope="col" class="px-4 py-3">
                                    {"Issuer"}
                                </th>
                                <th scope="col" class="px-4 py-3">
                                    {"Digest"}
                                </th>
                                <th scope="col" class="px-4 py-3">
                                    {"Expires on"}
                                </th>
                                <th scope="col" class="px-4 py-3">
                                    <span class="sr-only">{"Edit"}</span>
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                        { cert_list.into_iter().map(|entry| {
                            let subject_names = entry
                            .san
                            .iter()
                            .map(|name| name.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");

                        let id = entry.id;
                        let delete_onclick = Callback::from(move |e: MouseEvent|  {
                            e.prevent_default();
                            if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                                wasm_bindgen_futures::spawn_local(async move {
                                    let _ = delete_server_cert(id).await;
                                });
                            }
                        });

                        let download_onclick = Callback::from(move |e: MouseEvent|  {
                            e.prevent_default();
                            if gloo_dialogs::confirm(&format!("Are you sure to download {id}.tar.gz?\nThis file contains the unencrypted private key.")) {
                                location::assign(&format!("{API_ENDPOINT}/certs/{id}/download"));
                            }
                        });

                            html! {
                                <tr class="border-b dark:border-neutral-700">
                                    <th scope="row" class="px-4 py-4 font-medium text-neutral-900 dark:text-neutral-200 whitespace-nowrap">
                                        {subject_names}
                                    </th>
                                    <td class="px-4 py-4">
                                        {entry.issuer.clone()}
                                    </td>
                                    <td class="px-4 py-4">
                                        {entry.id.to_string()}
                                    </td>
                                    <td class="px-4 py-4">
                                        {format_duration(entry.not_after)}
                                    </td>
                                    <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
                                        <a class="cursor-pointer font-medium text-blue-600 dark:text-blue-400 hover:underline mr-5" onclick={download_onclick}>{"Download"}</a>
                                        <a class="cursor-pointer font-medium text-red-600 hover:underline" onclick={delete_onclick}>{"Delete"}</a>
                                    </td>
                                </tr>
                            }
                        }).collect::<Html>() }
                        </tbody>
                    </table>
                }
                </div>
        } else if *tab == CertsTab::Root {
            <div class="relative overflow-x-auto bg-white dark:bg-neutral-800 shadow-sm border border-neutral-300 dark:border-neutral-700 lg:rounded-md">
                if !certs.loaded {
                    <svg aria-hidden="true" role="status" class="w-8 h-8 mx-auto my-7 text-neutral-200 animate-spin" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="#ccc"/>
                    <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="#888"/>
                    </svg>
                } else if cert_list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold text-neutral-500 dark:text-neutral-300 px-16 text-center">{"List is empty."}</p>
                } else {
                <table class="w-full text-sm text-left text-neutral-600 dark:text-neutral-200 rounded-md">
                    <thead class="text-xs text-neutral-800 dark:text-neutral-200 uppercase border-b border-neutral-300 dark:border-neutral-700">
                        <tr>
                            <th scope="col" class="px-4 py-3">
                                {"Issuer"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Digest"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Private Key"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Expires on"}
                            </th>
                            <th scope="col" class="px-4 py-3" align="right">
                                <span class="sr-only">{"Edit"}</span>
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                    { cert_list.into_iter().map(|entry| {
                    let id = entry.id;
                    let delete_onclick = Callback::from(move |e: MouseEvent|  {
                        e.prevent_default();
                        if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = delete_server_cert(id).await;
                            });
                        }
                    });

                    let no_key = !entry.has_private_key;
                    let download_onclick = Callback::from(move |e: MouseEvent|  {
                        e.prevent_default();
                        if no_key || gloo_dialogs::confirm(&format!("Are you sure to download {id}.tar.gz?\nThis file contains the unencrypted private key.")) {
                            location::assign(&format!("{API_ENDPOINT}/certs/{id}/download"));
                        }
                    });

                        html! {
                            <tr class="border-b dark:border-neutral-700">
                                <td class="px-4 py-4">
                                    {entry.issuer.clone()}
                                </td>
                                <td class="px-4 py-4">
                                    {entry.id.to_string()}
                                </td>
                                <td class="px-4 py-4">
                                    if no_key {
                                        {"No"}
                                    } else {
                                        {"Yes"}
                                    }
                                </td>
                                <td class="px-4 py-4">
                                    {format_duration(entry.not_after)}
                                </td>
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
                                    <a class="cursor-pointer font-medium text-blue-600 dark:text-blue-400 hover:underline mr-5" onclick={download_onclick}>{"Download"}</a>
                                    <a class="cursor-pointer font-medium text-red-600 hover:underline" onclick={delete_onclick}>{"Delete"}</a>
                                </td>
                            </tr>
                        }
                    }).collect::<Html>() }
                    </tbody>
                </table>
                }
            </div>
            } else if *tab == CertsTab::Acme {
                <div class="relative overflow-x-auto bg-white dark:bg-neutral-800 shadow-sm border border-neutral-300 dark:border-neutral-700 lg:rounded-md">
                if !acme.loaded {
                    <svg aria-hidden="true" role="status" class="w-8 h-8 mx-auto my-7 text-neutral-200 animate-spin" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="#ccc"/>
                    <path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="#888"/>
                    </svg>
                } else if acme_list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold text-neutral-500 dark:text-neutral-300 px-16 text-center">{"List is empty."}</p>
                } else {
                <table class="w-full text-sm text-left text-neutral-600 dark:text-neutral-200 rounded-md">
                    <thead class="text-xs dark:text-neutral-200 uppercase border-b border-neutral-300 dark:border-neutral-700">
                        <tr>
                            <th scope="col" class="px-4 py-3">
                                {"Subject Names"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Provider"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Renews on"}
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
                    { acme_list.into_iter().map(|entry| {
                        let subject_names = entry.identifiers.join(", ");
                        let provider = entry.config.provider.to_string();

                        let id = entry.id;
                        let delete_onclick = Callback::from(move |e: MouseEvent|  {
                            e.prevent_default();
                            if gloo_dialogs::confirm(&format!("Are you sure to delete {id}?")) {
                                wasm_bindgen_futures::spawn_local(async move {
                                    let _ = delete_acme(id).await;
                                });
                            }
                        });

                        let id = entry.id;
                        let navigator_cloned = navigator.clone();
                        let log_onclick = Callback::from(move |_|  {
                            let id = id.to_string();
                            navigator_cloned.push(&Route::CertLogView {id});
                        });

                        let active = entry.config.active;
                        let onchange = Callback::from(move |_: Event| {
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = toggle_acme(id).await;
                            });
                        });

                        html! {
                            <tr class="border-b dark:border-neutral-700">
                                <td class="px-4 py-4">
                                    {subject_names}
                                </td>
                                <td class="px-4 py-4">
                                    {provider}
                                </td>
                                <td class="px-4 py-4">
                                    if let Some(time) = entry.next_renewal {
                                        { format_duration(time) }
                                    }
                                </td>
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="center">
                                    <label class="relative inline-flex items-center cursor-pointer mt-1">
                                        <input {onchange} type="checkbox" checked={active} class="sr-only peer" />
                                        <div class="w-9 h-4 bg-neutral-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-neutral-300 after:border after:rounded-full after:h-3 after:w-4 after:transition-all peer-checked:bg-blue-600"></div>
                                    </label>
                                </td>
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
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
            }
            <div class="flex justify-end rounded-md mt-4 sm:ml-auto px-4 lg:px-0" role="group">
                if *tab == CertsTab::Server {
                    <button onclick={self_sign_onclick} class="inline-flex items-center px-4 py-2 text-sm font-medium text-neutral-500 dark:text-neutral-200 bg-white dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-l-lg hover:bg-neutral-100 hover:dark:bg-neutral-900 focus:z-10 focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-600">
                        <img src="/assets/icons/create.svg" class="w-4 h-4 mr-1 text-neutral-500" />
                        {"Self-sign"}
                    </button>
                    <button onclick={upload_onclick} class="inline-flex items-center px-4 py-2 text-sm font-medium text-neutral-500 dark:text-neutral-200 bg-white dark:bg-neutral-800 border border-l-0 border-neutral-300 dark:border-neutral-700 rounded-r-lg hover:bg-neutral-100 hover:dark:bg-neutral-900 focus:z-10 focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-600">
                        <img src="/assets/icons/cloud-upload.svg" class="w-4 h-4 mr-1" />
                        {"Upload"}
                    </button>
                } else if *tab == CertsTab::Root {
                    <button onclick={upload_onclick} class="inline-flex items-center px-4 py-2 text-sm font-medium text-neutral-500 dark:text-neutral-200 bg-white dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg hover:bg-neutral-100 hover:dark:bg-neutral-900 focus:z-10 focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-600">
                        <img src="/assets/icons/cloud-upload.svg" class="w-4 h-4 mr-1" />
                        {"Upload"}
                    </button>
                } else {
                    <button onclick={new_acme_onclick} class="inline-flex items-center px-4 py-2 text-sm font-medium text-neutral-500 dark:text-neutral-200 bg-white dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 rounded-lg hover:bg-neutral-100 hover:dark:bg-neutral-900 focus:z-10 focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-600">
                        <img src="/assets/icons/add.svg" class="w-4 h-4 mr-1" />
                        {"Add"}
                    </button>
                }
            </div>
        </>
    }
}

async fn get_cert_list() -> Result<Vec<CertInfo>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/certs"))
        .send()
        .await?
        .json()
        .await
}

async fn get_acme_list() -> Result<Vec<AcmeInfo>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/acme"))
        .send()
        .await?
        .json()
        .await
}

async fn delete_server_cert(id: ShortId) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/certs/{id}"))
        .send()
        .await?;
    Ok(())
}

async fn delete_acme(id: ShortId) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("{API_ENDPOINT}/acme/{id}"))
        .send()
        .await?;
    Ok(())
}

async fn toggle_acme(id: ShortId) -> Result<(), gloo_net::Error> {
    let mut acme: AcmeInfo = Request::get(&format!("{API_ENDPOINT}/acme/{id}"))
        .send()
        .await?
        .json()
        .await?;
    acme.config.active = !acme.config.active;
    Request::put(&format!("{API_ENDPOINT}/acme/{id}"))
        .json(&acme.config)?
        .send()
        .await?;
    Ok(())
}

mod location {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = location)]
        pub fn assign(url: &str);
    }
}
