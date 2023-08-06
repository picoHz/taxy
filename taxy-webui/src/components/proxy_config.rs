use crate::components::http_proxy_config::HttpProxyConfig;
use crate::components::tcp_proxy_config::TcpProxyConfig;
use crate::store::PortStore;
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use std::collections::HashMap;
use taxy_api::id::ShortId;
use taxy_api::site::{HttpProxy, ProxyKind, TcpProxy};
use taxy_api::{port::PortEntry, site::Proxy};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProxyProtocol {
    Http,
    Tcp,
}

impl ToString for ProxyProtocol {
    fn to_string(&self) -> String {
        match self {
            ProxyProtocol::Http => "HTTP / HTTPS".to_string(),
            ProxyProtocol::Tcp => "TCP / TCP over TLS".to_string(),
        }
    }
}

const PROTOCOLS: &[ProxyProtocol] = &[ProxyProtocol::Http, ProxyProtocol::Tcp];

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub proxy: Proxy,
    pub onchanged: Callback<Result<Proxy, HashMap<String, String>>>,
}

#[function_component(ProxyConfig)]
pub fn proxy_config(props: &Props) -> Html {
    let (ports, dispatcher) = use_store::<PortStore>();

    let ports_cloned = ports.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_ports().await {
                    dispatcher.set(PortStore {
                        entries: res,
                        ..(*ports_cloned).clone()
                    });
                }
            });
        },
        (),
    );

    let name = use_state(|| props.proxy.name.clone());
    let name_onchange = Callback::from({
        let name = name.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            name.set(target.value());
        }
    });

    let protocol = use_state(|| {
        if matches!(props.proxy.kind, ProxyKind::Http(_)) {
            ProxyProtocol::Http
        } else {
            ProxyProtocol::Tcp
        }
    });
    let protocol_onchange = Callback::from({
        let protocol = protocol.clone();
        move |event: Event| {
            let target: HtmlSelectElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            if let Ok(index) = target.value().parse::<usize>() {
                protocol.set(PROTOCOLS[index]);
            }
        }
    });

    let bound_ports = use_state(|| props.proxy.ports.clone());

    let http_proxy = use_state::<Result<ProxyKind, HashMap<String, String>>, _>(|| {
        Ok(ProxyKind::Http(Default::default()))
    });
    let http_proxy_cloned = http_proxy.clone();
    let http_proxy_onchanged: Callback<Result<HttpProxy, HashMap<String, String>>> =
        Callback::from(move |updated: Result<HttpProxy, HashMap<String, String>>| {
            http_proxy_cloned.set(updated.map(ProxyKind::Http));
        });

    let tcp_proxy = use_state::<Result<ProxyKind, HashMap<String, String>>, _>(|| {
        Ok(ProxyKind::Http(Default::default()))
    });
    let tcp_proxy_cloned = tcp_proxy.clone();
    let tcp_proxy_onchanged: Callback<Result<TcpProxy, HashMap<String, String>>> =
        Callback::from(move |updated: Result<TcpProxy, HashMap<String, String>>| {
            tcp_proxy_cloned.set(updated.map(ProxyKind::Tcp));
        });

    let compatible_ports = ports
        .entries
        .clone()
        .into_iter()
        .filter(|entry| entry.port.listen.is_http() ^ (*protocol == ProxyProtocol::Tcp))
        .collect::<Vec<_>>();

    let prev_entry =
        use_state::<Result<Proxy, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry = get_site(
        &name,
        &bound_ports,
        if *protocol == ProxyProtocol::Http {
            &http_proxy
        } else {
            &tcp_proxy
        },
        &compatible_ports,
    );

    if entry != *prev_entry {
        prev_entry.set(entry.clone());
        props.onchanged.emit(entry);
    }

    let http_proxy = if let ProxyKind::Http(http_proxy) = &props.proxy.kind {
        http_proxy.clone()
    } else {
        Default::default()
    };

    let tcp_proxy = if let ProxyKind::Tcp(tcp_proxy) = &props.proxy.kind {
        tcp_proxy.clone()
    } else {
        Default::default()
    };

    html! {
        <>
            <label class="block mb-2 text-sm font-medium text-neutral-900">{"Name"}</label>
            <input type="text" value={name.to_string()} onchange={name_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" placeholder="My Website" />

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Protocol"}</label>
            <select onchange={protocol_onchange} class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5">
                { PROTOCOLS.iter().enumerate().map(|(i, item)| {
                    html! {
                        <option selected={&*protocol == item} value={i.to_string()}>{item.to_string()}</option>
                    }
                }).collect::<Html>() }
            </select>

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Ports"}</label>
            <ul class="h-32 pb-3 overflow-y-auto text-sm text-gray-700 bg-gray-50 border border-gray-300 rounded-lg">
                { compatible_ports.into_iter().map(|entry| {
                    let bound_ports_cloned = bound_ports.clone();
                    let onchange = Callback::from(move |event: Event| {
                        let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                        let mut ports = (*bound_ports_cloned).clone();
                        if target.checked() {
                            if !ports.contains(&entry.id) {
                                ports.push(entry.id);
                            }
                        } else {
                            ports.retain(|&id| id != entry.id);
                        }
                        bound_ports_cloned.set(ports);
                    });
                    let bound_ports_cloned = bound_ports.clone();
                    html! {
                        <li>
                            <div class="flex items-center pl-2 rounded hover:bg-gray-100">
                                <input {onchange} id={entry.id.to_string()} type="checkbox" checked={bound_ports_cloned.contains(&entry.id)} class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2" />
                                <label for={entry.id.to_string()} class="w-full py-2 ml-2 text-sm font-medium text-gray-900 rounded">{entry.port.listen.to_string()}</label>
                            </div>
                        </li>
                    }
                }).collect::<Html>() }
            </ul>

            if *protocol == ProxyProtocol::Http {
                <HttpProxyConfig onchanged={http_proxy_onchanged} proxy={http_proxy} />
            } else {
                <TcpProxyConfig onchanged={tcp_proxy_onchanged} proxy={tcp_proxy} />
            }
        </>
    }
}

fn get_site(
    name: &str,
    ports: &[ShortId],
    kind: &Result<ProxyKind, HashMap<String, String>>,
    compatible_ports: &[PortEntry],
) -> Result<Proxy, HashMap<String, String>> {
    let mut errors = HashMap::new();
    let mut ports = ports.to_vec();
    ports.retain(|&id| compatible_ports.iter().any(|entry| entry.id == id));
    ports.sort();
    ports.dedup();

    let kind = match kind {
        Ok(kind) => kind,
        Err(err) => {
            errors.extend(err.clone());
            return Err(errors);
        }
    };

    if errors.is_empty() {
        Ok(Proxy {
            name: name.trim().to_string(),
            ports,
            kind: kind.clone(),
        })
    } else {
        Err(errors)
    }
}

async fn get_ports() -> Result<Vec<PortEntry>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports"))
        .send()
        .await?
        .json()
        .await
}
