use crate::auth::use_ensure_auth;
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
                    certs_dispatcher.set(CertStore { entries: res });
                }
                if let Ok(res) = get_acme_list().await {
                    acme_dispatcher.set(AcmeStore { entries: res });
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
        <div class="flex flex-col mb-4 sm:flex-row px-4 lg:px-0">
            <div class="text-sm font-medium text-center text-neutral-500">
                <ul class="flex flex-wrap -mb-px">

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
                            vec!["text-blue-600", "border-blue-600", "active"]
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
                <div class="relative overflow-x-auto bg-white shadow-sm border border-neutral-300 lg:rounded-md">
                if cert_list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold text-neutral-500 px-16 text-center">{"List is empty."}</p>
                } else {
                    <table class="w-full text-sm text-left text-neutral-600 rounded-md">
                        <thead class="text-xs text-neutral-800 uppercase border-b border-neutral-300">
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
                                <tr class="border-b">
                                    <th scope="row" class="px-4 py-4 font-medium text-neutral-900 whitespace-nowrap">
                                        {subject_names}
                                    </th>
                                    <td class="px-4 py-4">
                                        {entry.issuer.clone()}
                                    </td>
                                    <td class="px-4 py-4">
                                        {entry.id.to_string()}
                                    </td>
                                    <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
                                        <a class="cursor-pointer font-medium text-blue-600 hover:underline mr-5" onclick={download_onclick}>{"Download"}</a>
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
            <div class="relative overflow-x-auto bg-white shadow-sm border border-neutral-300 lg:rounded-md">
                if cert_list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold text-neutral-500 px-16 text-center">{"List is empty."}</p>
                } else {
                <table class="w-full text-sm text-left text-neutral-600 rounded-md">
                    <thead class="text-xs text-neutral-800 uppercase border-b border-neutral-300">
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
                            <tr class="border-b">
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
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
                                    <a class="cursor-pointer font-medium text-blue-600 hover:underline mr-5" onclick={download_onclick}>{"Download"}</a>
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
                <div class="relative overflow-x-auto bg-white shadow-sm border border-neutral-300 lg:rounded-md">
                if acme_list.is_empty() {
                    <p class="mb-8 mt-8 text-xl font-bold text-neutral-500 px-16 text-center">{"List is empty."}</p>
                } else {
                <table class="w-full text-sm text-left text-neutral-600 rounded-md">
                    <thead class="text-xs text-neutral-800 uppercase border-b border-neutral-300">
                        <tr>
                            <th scope="col" class="px-4 py-3">
                                {"Subject Names"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                {"Provider"}
                            </th>
                            <th scope="col" class="px-4 py-3">
                                <span class="sr-only">{"Edit"}</span>
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                    { acme_list.into_iter().map(|entry| {
                        let subject_names = entry.identifiers.join(", ");
                        let provider = entry.provider.to_string();

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


                        html! {
                            <tr class="border-b">
                                <td class="px-4 py-4">
                                    {subject_names}
                                </td>
                                <td class="px-4 py-4">
                                    {provider}
                                </td>
                                <td class="px-4 py-4 w-0 whitespace-nowrap" align="right">
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
            }
            <div class="flex justify-end rounded-md mt-4 sm:ml-auto px-4 lg:px-0" role="group">
                if *tab == CertsTab::Server {
                    <button onclick={self_sign_onclick} class="inline-flex items-center px-4 py-2 text-sm font-medium text-neutral-500 bg-white border border-neutral-300 rounded-l-lg hover:bg-neutral-100 focus:z-10 focus:ring-4 focus:ring-neutral-200">
                        <img src="/assets/icons/create.svg" class="w-4 h-4 mr-1 text-neutral-500" />
                        {"Self-sign"}
                    </button>
                    <button onclick={upload_onclick} class="inline-flex items-center px-4 py-2 text-sm font-medium text-neutral-500 bg-white border border-l-0 border-neutral-300 rounded-r-lg hover:bg-neutral-100 focus:z-10 focus:ring-4 focus:ring-neutral-200">
                        <img src="/assets/icons/cloud-upload.svg" class="w-4 h-4 mr-1" />
                        {"Upload"}
                    </button>
                } else if *tab == CertsTab::Root {
                    <button onclick={upload_onclick} class="inline-flex items-center px-4 py-2 text-sm font-medium text-neutral-500 bg-white border border-neutral-300 rounded-lg hover:bg-neutral-100 focus:z-10 focus:ring-4 focus:ring-neutral-200">
                        <img src="/assets/icons/cloud-upload.svg" class="w-4 h-4 mr-1" />
                        {"Upload"}
                    </button>
                } else {
                    <button onclick={new_acme_onclick} class="inline-flex items-center px-4 py-2 text-sm font-medium text-neutral-500 bg-white border border-neutral-300 rounded-lg hover:bg-neutral-100 focus:z-10 focus:ring-4 focus:ring-neutral-200">
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

mod location {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = location)]
        pub fn assign(url: &str);
    }
}
