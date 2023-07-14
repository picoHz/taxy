use multiaddr::{Multiaddr, Protocol};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};
use taxy_api::port::UpstreamServer;
use taxy_api::site::TcpProxy;
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
            .map(|server| extract_host_port(&server.addr))
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

    let err = Default::default();
    let errors = prev_entry.as_ref().err().unwrap_or(&err);

    html! {
        <>
            <div class="field is-horizontal m-5">
            <div class="field-label is-normal">
            <label class="label">{"Upstream Servers"}</label>
            </div>

            <div class="is-flex-grow-5" style="flex-basis: 0">

            { upstream_servers.iter().enumerate().map(|(i, (host, port))| {
                let servers_len = upstream_servers.len();

                let upstream_servers_cloned = upstream_servers.clone();
                let add_onclick = Callback::from(move |_| {
                    let mut servers = (*upstream_servers_cloned).clone();
                    servers.insert(i + 1, ("example.com".into(), 8080));
                    upstream_servers_cloned.set(servers);
                });

                let upstream_servers_cloned = upstream_servers.clone();
                let remove_onclick = Callback::from(move |_| {
                    if servers_len > 1 {
                        let mut servers = (*upstream_servers_cloned).clone();
                        servers.remove(i);
                        upstream_servers_cloned.set(servers);
                    }
                });

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

                let not_first = i > 0;
                let err = errors.get(&format!("upstream_servers_{}", i)).map(|s| s.as_str());

                html! {
                    <div class={classes!(not_first.then_some("mt-3"))}>
                        <div class={classes!("field-body")}>
                        <div class="field has-addons">
                            <div class="control is-expanded">
                                <input class={classes!("input", err.map(|_| "is-danger"))} type="text" autocapitalize="off" placeholder="Host" onchange={host_onchange} value={host.clone()} />
                            </div>
                            <div class="control">
                                <input class={classes!("input", err.map(|_| "is-danger"))} type="number" placeholder="Port" max="65535" min="1" onchange={port_onchange} value={port.to_string()} />
                            </div>
                            <div class="control">
                                <button type="button" class={classes!("button", err.map(|_| "is-danger"))} onclick={add_onclick}>
                                    <span class="icon">
                                        <ion-icon name="add"></ion-icon>
                                    </span>
                                </button>
                            </div>
                            <div class="control">
                                <button type="button" class={classes!("button", err.map(|_| "is-danger"))} onclick={remove_onclick} disabled={servers_len <= 1}>
                                    <span class="icon">
                                        <ion-icon name="remove"></ion-icon>
                                    </span>
                                </button>
                            </div>
                        </div>
                    </div>
                    if let Some(err) = err {
                        <p class="help is-danger">{err}</p>
                    }
                </div>
                }
            }).collect::<Html>() }
            </div>
        </div>
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
            let addr: Multiaddr = "/dns/example.com/tcp/8080".parse().unwrap();
            let addr = set_host_port(&addr, host, *port);
            upstream_servers.push(UpstreamServer { addr });
        }
    }

    if errors.is_empty() {
        Ok(TcpProxy { upstream_servers })
    } else {
        Err(errors)
    }
}

fn extract_host_port(addr: &Multiaddr) -> (String, u16) {
    let host = addr
        .iter()
        .find_map(|p| match p {
            Protocol::Dns(host) | Protocol::Dns4(host) | Protocol::Dns6(host) => {
                Some(host.to_string())
            }
            Protocol::Ip4(host) => Some(host.to_string()),
            Protocol::Ip6(host) => Some(host.to_string()),
            _ => None,
        })
        .unwrap_or_else(|| "127.0.0.1".into());
    let port = addr
        .iter()
        .find_map(|p| match p {
            Protocol::Tcp(port) => Some(port),
            _ => None,
        })
        .unwrap_or(8080);
    (host, port)
}

fn set_host_port(addr: &Multiaddr, host: &str, port: u16) -> Multiaddr {
    addr.iter()
        .map(|p| match p {
            Protocol::Dns(_)
            | Protocol::Dns4(_)
            | Protocol::Dns6(_)
            | Protocol::Ip4(_)
            | Protocol::Ip6(_) => {
                if let Ok(addr) = host.parse::<Ipv4Addr>() {
                    Protocol::Ip4(addr)
                } else if let Ok(addr) = host.parse::<Ipv6Addr>() {
                    Protocol::Ip6(addr)
                } else {
                    Protocol::Dns(host.into())
                }
            }
            Protocol::Tcp(_) => Protocol::Tcp(port),
            _ => p,
        })
        .collect()
}
