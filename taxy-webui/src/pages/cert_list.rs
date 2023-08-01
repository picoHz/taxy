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
        <div class="flex flex-col sm:flex-row px-4 md:px-0">
            <div class="text-sm font-medium text-center text-gray-500">
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
                                <a {onclick} class={classes!("inline-block", "px-4", "py-2", "border-b-2", "rounded-t-lg", "hover:text-gray-600", "hover:border-gray-300", class)}>{item}</a>
                            </li>
                        }
                    }).collect::<Html>() }

                </ul>
            </div>
            <div class="inline-flex rounded-md my-4 sm:my-0 sm:ml-auto" role="group">
                if *tab == CertsTab::Server {
                    <button type="button" onclick={self_sign_onclick} class="px-4 py-2 text-sm font-medium text-gray-500 bg-white border border-gray-300 rounded-l-lg hover:bg-gray-100 focus:z-10 focus:ring-4 focus:ring-gray-200">
                        {"Self-sign"}
                    </button>
                    <button type="button" onclick={upload_onclick} class="px-4 py-2 text-sm font-medium text-gray-500 bg-white border border-l-0 border-gray-300 rounded-r-lg hover:bg-gray-100 focus:z-10 focus:ring-4 focus:ring-gray-200">
                        {"Upload"}
                    </button>
                } else if *tab == CertsTab::Root {
                    <button type="button" onclick={upload_onclick} class="px-4 py-2 text-sm font-medium text-gray-500 bg-white border border-gray-300 rounded-lg hover:bg-gray-100 focus:z-10 focus:ring-4 focus:ring-gray-200">
                        {"Upload"}
                    </button>
                } else {
                    <button type="button" onclick={new_acme_onclick} class="px-4 py-2 text-sm font-medium text-gray-500 bg-white border border-gray-300 rounded-lg hover:bg-gray-100 focus:z-10 focus:ring-4 focus:ring-gray-200">
                        {"Add"}
                    </button>
                }
            </div>
        </div>

            if ((*tab == CertsTab::Server || *tab == CertsTab::Root) && cert_list.is_empty()) || (*tab == CertsTab::Acme && acme_list.is_empty()) {
                <p class="mb-8 mt-8 text-xl font-bold text-gray-500 px-16 text-center">{"List is empty."}</p>
            }

            if *tab == CertsTab::Server && !cert_list.is_empty() {
                <div class="relative overflow-x-auto">
                    <table class="w-full text-sm text-left text-neutral-600 rounded-md">
                        <thead class="text-xs text-neutral-800 uppercase">
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
                                        <a class="font-medium text-blue-600 hover:underline mr-5" onclick={download_onclick}>{"Download"}</a>
                                        <a class="font-medium text-red-600 hover:underline" onclick={delete_onclick}>{"Delete"}</a>
                                    </td>
                                </tr>
                            }
                        }).collect::<Html>() }
                        </tbody>
                    </table>
                </div>
        } else if *tab == CertsTab::Root && !cert_list.is_empty() {
            <div class="relative overflow-x-auto">
                <table class="w-full text-sm text-left text-neutral-600 rounded-md">
                    <thead class="text-xs text-neutral-800 uppercase">
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
                                    <a class="font-medium text-blue-600 hover:underline mr-5" onclick={download_onclick}>{"Download"}</a>
                                    <a class="font-medium text-red-600 hover:underline" onclick={delete_onclick}>{"Delete"}</a>
                                </td>
                            </tr>
                        }
                    }).collect::<Html>() }
                    </tbody>
                </table>
            </div>
            } else if *tab == CertsTab::Acme && !acme_list.is_empty() {
                <div class="relative overflow-x-auto">
                <table class="w-full text-sm text-left text-neutral-600 rounded-md">
                    <thead class="text-xs text-neutral-800 uppercase">
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
                                    <a class="font-medium text-blue-600 hover:underline mr-5" onclick={log_onclick}>{"Log"}</a>
                                    <a class="font-medium text-red-600 hover:underline" onclick={delete_onclick}>{"Delete"}</a>
                                </td>
                            </tr>
                        }
                    }).collect::<Html>() }
                    </tbody>
                </table>
            </div>
            }
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
