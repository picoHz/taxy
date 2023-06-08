use multiaddr::Protocol;
use taxy_api::port::Port;
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
    let stack = props.port.listen.iter().collect::<Vec<_>>();
    let tls = stack.iter().any(|p| matches!(p, Protocol::Tls));
    let http = stack.iter().any(|p| matches!(p, Protocol::Http));
    let (interface, port) = match &stack[..] {
        [Protocol::Ip4(addr), Protocol::Tcp(port), ..] => (addr.to_string(), *port),
        [Protocol::Ip6(addr), Protocol::Tcp(port), ..] => (addr.to_string(), *port),
        _ => ("0.0.0.0".to_string(), 8080),
    };

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
                            <input class="input" type="text" placeholder="Server Names" />
                            </p>
                            <p class="help">
                            {"You can use commas to list multiple names, e.g, example.com, *.test.examle.com."}
                            </p>
                        </div>
                    </div>
                </div>
            }
        </>
    }
}
