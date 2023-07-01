use crate::API_ENDPOINT;
use gloo_net::http::Request;
use multiaddr::{Multiaddr, Protocol};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
};
use taxy_api::{
    port::{NetworkInterface, Port, UpstreamServer},
    tls::TlsTermination,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_else(create_default_port)]
    pub port: Port,
    pub on_changed: Callback<Result<Port, HashMap<String, String>>>,
}

fn create_default_port() -> Port {
    Port {
        name: String::new(),
        listen: "/ip4/0.0.0.0/tcp/8080".parse().unwrap(),
        opts: Default::default(),
    }
}

const PROTOCOLS: &[(&str, &str)] = &[
    ("tcp", "TCP"),
    ("tls", "TLS"),
    ("http", "HTTP"),
    ("https", "HTTPS"),
];

#[function_component(PortConfig)]
pub fn port_config(props: &Props) -> Html {
    let stack = &props.port.listen;
    let tls = stack
        .iter()
        .any(|p| matches!(p, Protocol::Tls) || matches!(p, Protocol::Https));
    let http = stack
        .iter()
        .any(|p| matches!(p, Protocol::Http) || matches!(p, Protocol::Https));
    let (interface, port) = extract_host_port(&props.port.listen);

    let interfaces = use_state(|| vec![interface.clone()]);
    let interfaces_cloned = interfaces.clone();
    let interface_cloned = interface.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(entry) = get_interfaces().await {
                    let mut list = vec!["0.0.0.0".into(), "::".into()]
                        .into_iter()
                        .chain(
                            entry
                                .into_iter()
                                .flat_map(|ifs| ifs.addrs)
                                .map(|addr| addr.ip.to_string()),
                        )
                        .collect::<Vec<_>>();
                    if !list.contains(&interface_cloned) {
                        list.push(interface_cloned);
                    }
                    interfaces_cloned.set(list);
                }
            });
        },
        (),
    );

    let protocol = match (tls, http) {
        (true, true) => "https",
        (true, false) => "tls",
        (false, true) => "http",
        (false, false) => "tcp",
    };

    let name = use_state(|| props.port.name.clone());
    let name_onchange = Callback::from({
        let name = name.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            name.set(target.value());
        }
    });

    let protocol = use_state(|| protocol.to_string());
    let protocol_onchange = Callback::from({
        let protocol = protocol.clone();
        move |event: Event| {
            let target: HtmlSelectElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            protocol.set(target.value());
        }
    });

    let interface = use_state(|| interface);
    let interface_onchange = Callback::from({
        let interface = interface.clone();
        move |event: Event| {
            let target: HtmlSelectElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            interface.set(target.value());
        }
    });

    let port = use_state(|| port);
    let port_onchange = Callback::from({
        let port = port.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            port.set(target.value().parse().unwrap_or(1));
        }
    });

    let tls_server_names = use_state(|| {
        props
            .port
            .opts
            .tls_termination
            .as_ref()
            .map(|tls| tls.server_names.join(", "))
            .unwrap_or_default()
    });
    let tls_server_names_onchange = Callback::from({
        let tls_server_names = tls_server_names.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            tls_server_names.set(target.value());
        }
    });

    let upstream_servers = use_state(|| {
        props
            .port
            .opts
            .upstream_servers
            .iter()
            .map(|server| extract_host_port(&server.addr))
            .collect::<Vec<_>>()
    });
    if &*protocol == "tcp" || &*protocol == "tls" {
        if upstream_servers.is_empty() {
            upstream_servers.set(vec![("example.com".into(), 8080)]);
        }
    } else if !upstream_servers.is_empty() {
        upstream_servers.set(Vec::new());
    }

    let prev_entry =
        use_state::<Result<Port, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry = get_port(
        &name,
        &protocol,
        &interface,
        *port,
        &tls_server_names,
        &upstream_servers,
    );
    if entry != *prev_entry {
        prev_entry.set(entry.clone());
        props.on_changed.emit(entry);
    }

    let err = Default::default();
    let errors = prev_entry.as_ref().err().unwrap_or(&err);

    html! {
        <>
            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                <label class="label">{"Name"}</label>
                </div>
                <div class="field-body">
                    <div class="field">
                        <p class="control is-expanded">
                        <input class="input" type="text" value={name.to_string()} onchange={name_onchange} />
                        </p>
                    </div>
                </div>
            </div>

            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                <label class="label">{"Listener"}</label>
                </div>
                <div class="field-body">
                    <div class="field is-narrow">
                    <div class="control">
                        <div class="select is-fullwidth">
                        <select onchange={interface_onchange}>
                            { interfaces.iter().map(|value| {
                                html! {
                                    <option selected={&*interface == value} value={value.clone()}>{value}</option>
                                }
                            }).collect::<Html>() }
                        </select>
                        </div>
                    </div>
                    </div>

                    <div class="field">
                        <p class="control is-expanded">
                        <input class="input" type="number" placeholder="Port" onchange={port_onchange} value={port.to_string()} max="65535" min="1" />
                        </p>
                    </div>
                </div>
            </div>

            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                    <label class="label">{"Protocol"}</label>
                </div>
                <div class="field-body">
                    <div class="field is-narrow">
                    <div class="control">
                        <div class="select is-fullwidth">
                        <select onchange={protocol_onchange}>
                            { PROTOCOLS.iter().map(|(value, label)| {
                                html! {
                                    <option selected={&*protocol == value} value={*value}>{label}</option>
                                }
                            }).collect::<Html>() }
                        </select>
                        </div>
                    </div>
                    </div>
                </div>
            </div>

            if &*protocol == "tls" || &*protocol == "https" {
                <div class="field is-horizontal m-5">
                    <div class="field-label is-normal">
                    <label class="label">{"TLS Server Names"}</label>
                    </div>
                    <div class="field-body">
                        <div class="field">
                            <p class="control is-expanded">
                            <input class="input" type="text" autocapitalize="off" placeholder="Server Names" value={tls_server_names.to_string()} onchange={tls_server_names_onchange} />
                            </p>
                            <p class="help">
                            {"You can use commas to list multiple names, e.g, example.com, *.test.examle.com."}
                            </p>
                        </div>
                    </div>
                </div>
            }

            if &*protocol == "tcp" || &*protocol == "tls" {
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
                                        <button class={classes!("button", err.map(|_| "is-danger"))} onclick={add_onclick}>
                                            <span class="icon">
                                                <ion-icon name="add"></ion-icon>
                                            </span>
                                        </button>
                                    </div>
                                    <div class="control">
                                        <button class={classes!("button", err.map(|_| "is-danger"))} onclick={remove_onclick} disabled={servers_len <= 1}>
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
            }

        </>
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

fn get_port(
    name: &str,
    protocol: &str,
    interface: &str,
    port: u16,
    tls_server_names: &str,
    upstream_servers: &[(String, u16)],
) -> Result<Port, HashMap<String, String>> {
    let mut errors = HashMap::new();
    let mut addr = Multiaddr::empty();

    let interface = interface.trim();
    if interface.is_empty() {
        errors.insert("interface".into(), "Interface is required".into());
    } else if let Ok(ip) = interface.parse::<Ipv4Addr>() {
        addr.push(Protocol::Ip4(ip));
    } else if let Ok(ip) = interface.parse::<Ipv6Addr>() {
        addr.push(Protocol::Ip6(ip));
    } else {
        addr.push(Protocol::Dns(interface.into()));
    }

    match protocol {
        "tcp" => {
            addr.push(Protocol::Tcp(port));
        }
        "tls" => {
            addr.push(Protocol::Tcp(port));
            addr.push(Protocol::Tls);
        }
        "http" => {
            addr.push(Protocol::Tcp(port));
            addr.push(Protocol::Http);
        }
        "https" => {
            addr.push(Protocol::Tcp(port));
            addr.push(Protocol::Https);
        }
        _ => {
            errors.insert("protocol".into(), "Invalid protocol".into());
        }
    }

    let mut opts = Port {
        name: name.trim().to_string(),
        listen: addr,
        opts: Default::default(),
    };
    if protocol == "tls" || protocol == "https" {
        opts.opts.tls_termination = Some(TlsTermination {
            server_names: tls_server_names
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        });
    }
    if protocol == "tcp" || protocol == "tls" {
        for (i, (host, port)) in upstream_servers.iter().enumerate() {
            if host.is_empty() {
                errors.insert(format!("upstream_servers_{i}"), "Host is required".into());
            } else {
                let addr: Multiaddr = "/dns/example.com/tcp/8080".parse().unwrap();
                let addr = set_host_port(&addr, host, *port);
                opts.opts.upstream_servers.push(UpstreamServer { addr });
            }
        }
    }

    if errors.is_empty() {
        Ok(opts)
    } else {
        Err(errors)
    }
}

async fn get_interfaces() -> Result<Vec<NetworkInterface>, gloo_net::Error> {
    Request::get(&format!("{API_ENDPOINT}/ports/interfaces"))
        .send()
        .await?
        .json()
        .await
}
