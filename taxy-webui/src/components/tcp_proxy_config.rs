use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use taxy_api::port::UpstreamServer;
use taxy_api::proxy::TcpProxy;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub proxy: TcpProxy,
    pub onchanged: Callback<Result<TcpProxy, HashMap<String, String>>>,
}

#[function_component(TcpProxyConfig)]
pub fn tls_proxy_config(props: &Props) -> Html {
    let upstream_servers = use_state(|| {
        props
            .proxy
            .upstream_servers
            .iter()
            .map(|server| {
                (
                    server.addr.host().unwrap_or_default(),
                    server.addr.port().unwrap_or(0),
                )
            })
            .collect::<Vec<_>>()
    });
    if upstream_servers.is_empty() {
        upstream_servers.set(vec![("example.com".into(), 8080)]);
    }

    let prev_entry =
        use_state::<Result<TcpProxy, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry = get_proxy(&upstream_servers);

    if entry != *prev_entry {
        prev_entry.set(entry.clone());
        props.onchanged.emit(entry);
    }

    html! {
        <>
            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900 dark:text-neutral-200">{"Upstream Server"}</label>

            { upstream_servers.iter().enumerate().map(|(i, (host, port))| {
                let upstream_servers_cloned = upstream_servers.clone();
                let host_onchange = Callback::from(move |event: Event| {
                    let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                    let mut servers = (*upstream_servers_cloned).clone();
                    servers[i].0 = target.value();
                    upstream_servers_cloned.set(servers);
                });

                let upstream_servers_cloned = upstream_servers.clone();
                let port_onchange = Callback::from(move |event: Event| {
                    let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                    let mut servers = (*upstream_servers_cloned).clone();
                    servers[i].1 = target.value().parse().unwrap();
                    upstream_servers_cloned.set(servers);
                });

                html! {
                    <div class="mt-2 bg-white shadow-sm p-5 border border-neutral-300 dark:border-neutral-600 dark:bg-neutral-800 rounded-md">
                        <label class="block mb-2 text-sm font-medium text-neutral-900 dark:text-neutral-200">{"Host"}</label>
                        <input type="text" autocapitalize="off" placeholder="example.com" onchange={host_onchange} value={host.clone()} class="bg-neutral-50 dark:text-neutral-200 dark:bg-neutral-800 dark:border-neutral-600 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />

                        <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900 dark:text-neutral-200">{"Port"}</label>
                        <input type="number" placeholder="8080" onchange={port_onchange} value={port.to_string()} max="65535" min="1" class="bg-neutral-50 dark:text-neutral-200 dark:bg-neutral-800 dark:border-neutral-600 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />
                    </div>
                }
            }).collect::<Html>() }
        </>
    }
}

fn get_proxy(servers: &[(String, u16)]) -> Result<TcpProxy, HashMap<String, String>> {
    let mut errors = HashMap::new();

    let mut upstream_servers = Vec::new();
    for (i, (host, port)) in servers.iter().enumerate() {
        if host.is_empty() {
            errors.insert(format!("upstream_servers_{i}"), "Host is required".into());
        } else {
            let addr = if let Ok(addr) = host.parse::<Ipv4Addr>() {
                format!("/ip4/{addr}/tcp/{port}")
            } else if let Ok(addr) = host.parse::<Ipv6Addr>() {
                format!("/ip6/{addr}/tcp/{port}")
            } else {
                format!("/dns/{host}/tcp/{port}")
            };
            let addr = addr.parse().unwrap();
            upstream_servers.push(UpstreamServer { addr });
        }
    }

    if errors.is_empty() {
        Ok(TcpProxy { upstream_servers })
    } else {
        Err(errors)
    }
}
