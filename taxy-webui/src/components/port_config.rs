use multiaddr::{Multiaddr, Protocol};
use taxy_api::port::{Port, UpstreamServer};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_else(create_default_port)]
    pub port: Port,
}

fn create_default_port() -> Port {
    Port {
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
    let tls = stack.iter().any(|p| matches!(p, Protocol::Tls));
    let http = stack.iter().any(|p| matches!(p, Protocol::Http));
    let (interface, port) = extract_host_port(&props.port.listen);

    let protocol = match (tls, http) {
        (true, true) => "https",
        (true, false) => "tls",
        (false, true) => "http",
        (false, false) => "tcp",
    };

    let protocol = use_state(|| protocol.to_string());
    let protocol_onchange = Callback::from({
        let protocol = protocol.clone();
        move |event: Event| {
            let target: HtmlSelectElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            protocol.set(target.value());
        }
    });

    let interface = use_state(|| interface);
    let interface_oninput = Callback::from({
        let interface = interface.clone();
        move |event: InputEvent| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            interface.set(target.value());
        }
    });

    let port = use_state(|| port);
    let port_oninput = Callback::from({
        let port = port.clone();
        move |event: InputEvent| {
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
    let tls_server_names_oninput = Callback::from({
        let tls_server_names = tls_server_names.clone();
        move |event: InputEvent| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            tls_server_names.set(target.value());
        }
    });

    let upstream_servers = use_state(|| props.port.opts.upstream_servers.clone());
    if &*protocol == "tcp" || &*protocol == "tls" {
        if upstream_servers.is_empty() {
            upstream_servers.set(vec![UpstreamServer {
                addr: "/dns/example.com/tcp/8080".parse().unwrap(),
            }]);
        }
    } else if !upstream_servers.is_empty() {
        upstream_servers.set(Vec::new());
    }

    html! {
        <>
            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                <label class="label">{"Listener"}</label>
                </div>
                <div class="field-body">
                    <div class="field">
                        <p class="control is-expanded">
                        <input class="input" type="text" placeholder="Interface" oninput={interface_oninput} value={interface.to_string()} />
                        </p>
                        <p class="help is-danger">
                          {"This interface is not available on the server."}
                        </p>
                    </div>
                    <div class="field">
                        <p class="control is-expanded">
                        <input class="input" type="number" placeholder="Port" oninput={port_oninput} value={port.to_string()} max="65535" min="1" />
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
                            { PROTOCOLS.into_iter().map(|(value, label)| {
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
                            <input class="input" type="text" placeholder="Server Names" value={tls_server_names.to_string()} oninput={tls_server_names_oninput} />
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

                    { upstream_servers.iter().map(|server| {
                        let (host, port) = extract_host_port(&server.addr);
                        html! {
                            <div class="field-body">
                            <div class="field has-addons">
                                <div class="control is-expanded">
                                    <input class="input" type="text" placeholder="Host" value={host} />
                                </div>
                                <div class="control">
                                    <input class="input" type="number" placeholder="Port" max="65535" min="1" value={port.to_string()} />
                                </div>
                                <div class="control">
                                    <button class="button">
                                        <span class="icon">
                                            <ion-icon name="add"></ion-icon>
                                        </span>
                                    </button>
                                </div>
                                <div class="control">
                                    <button class="button">
                                        <span class="icon">
                                            <ion-icon name="remove"></ion-icon>
                                        </span>
                                    </button>
                                </div>
                            </div>
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
    let mut iter = addr.iter();
    let host = iter
        .find_map(|p| match p {
            Protocol::Dns4(host) => Some(host.to_string()),
            Protocol::Dns6(host) => Some(host.to_string()),
            Protocol::Ip4(host) => Some(host.to_string()),
            Protocol::Ip6(host) => Some(host.to_string()),
            _ => None,
        })
        .unwrap_or_else(|| "127.0.0.1".into());
    let port = iter
        .find_map(|p| match p {
            Protocol::Tcp(port) => Some(port),
            _ => None,
        })
        .unwrap_or(8080);
    (host, port)
}
