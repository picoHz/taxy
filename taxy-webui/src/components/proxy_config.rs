use crate::components::http_proxy_config::HttpProxyConfig;
use crate::components::tcp_proxy_config::TcpProxyConfig;
use crate::store::PortStore;
use crate::utils::format_multiaddr;
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use multiaddr::Protocol;
use std::collections::HashMap;
use taxy_api::site::{HttpProxy, ProxyKind, TcpProxy};
use taxy_api::{port::PortEntry, site::Proxy};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlOptionElement, HtmlSelectElement};
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
            ProxyProtocol::Tcp => "TCP / TLS".to_string(),
        }
    }
}

const PROTOCOLS: &[ProxyProtocol] = &[ProxyProtocol::Http, ProxyProtocol::Tcp];

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_else(create_default_site)]
    pub proxy: Proxy,
    pub on_changed: Callback<Result<Proxy, HashMap<String, String>>>,
}

fn create_default_site() -> Proxy {
    Proxy {
        name: String::new(),
        ports: vec![],
        kind: ProxyKind::Http(HttpProxy {
            vhosts: vec![],
            routes: vec![],
        }),
    }
}

#[function_component(ProxyConfig)]
pub fn proxy_config(props: &Props) -> Html {
    let (ports, dispatcher) = use_store::<PortStore>();

    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(res) = get_ports().await {
                    dispatcher.set(PortStore { entries: res });
                }
            });
        },
        (),
    );

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

    let name = use_state(|| props.proxy.name.clone());
    let name_onchange = Callback::from({
        let name = name.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            name.set(target.value());
        }
    });

    let bound_ports = use_state(|| props.proxy.ports.clone());
    let bound_ports_onchange = Callback::from({
        let bound_ports = bound_ports.clone();
        move |event: Event| {
            let target: HtmlSelectElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            let mut ports = Vec::new();
            let opts = target.selected_options();
            for i in 0..opts.length() {
                let opt: HtmlOptionElement = opts.item(i).unwrap_throw().dyn_into().unwrap_throw();
                ports.push(opt.value());
            }
            bound_ports.set(ports);
        }
    });

    let http_proxy = use_state::<Result<ProxyKind, HashMap<String, String>>, _>(|| {
        Ok(ProxyKind::Http(Default::default()))
    });
    let http_proxy_cloned = http_proxy.clone();
    let http_proxy_on_changed: Callback<Result<HttpProxy, HashMap<String, String>>> =
        Callback::from(move |updated: Result<HttpProxy, HashMap<String, String>>| {
            http_proxy_cloned.set(updated.map(ProxyKind::Http));
        });

    let tcp_proxy = use_state::<Result<ProxyKind, HashMap<String, String>>, _>(|| {
        Ok(ProxyKind::Http(Default::default()))
    });
    let tcp_proxy_cloned = tcp_proxy.clone();
    let tcp_proxy_on_changed: Callback<Result<TcpProxy, HashMap<String, String>>> =
        Callback::from(move |updated: Result<TcpProxy, HashMap<String, String>>| {
            tcp_proxy_cloned.set(updated.map(ProxyKind::Tcp));
        });

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
    );

    if entry != *prev_entry {
        prev_entry.set(entry.clone());
        props.on_changed.emit(entry);
    }

    let compatible_ports = ports.entries.iter().filter(|entry| {
        entry
            .port
            .listen
            .iter()
            .any(|item| matches!(item, Protocol::Http | Protocol::Https))
            ^ (*protocol == ProxyProtocol::Tcp)
    });

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
            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                    <label class="label">{"Protocol"}</label>
                </div>
                <div class="field-body">
                    <div class="field is-narrow">
                    <div class="control">
                        <div class="select is-fullwidth">
                        <select onchange={protocol_onchange}>
                            { PROTOCOLS.iter().enumerate().map(|(i, item)| {
                                html! {
                                    <option selected={&*protocol == item} value={i.to_string()}>{item.to_string()}</option>
                                }
                            }).collect::<Html>() }
                        </select>
                        </div>
                    </div>
                    </div>
                </div>
            </div>

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
                <label class="label">{"Ports"}</label>
                </div>
                <div class="field-body">
                    <div class="field">
                        <div class="select is-multiple is-fullwidth">
                            <select class="is-expanded" multiple={true} size="3" onchange={bound_ports_onchange}>
                                { compatible_ports.map(|entry| {
                                    html! {
                                        <option selected={bound_ports.contains(&entry.id)} value={entry.id.clone()}>
                                            {format_multiaddr(&entry.port.listen)}
                                        </option>
                                    }
                                }).collect::<Html>() }
                            </select>
                        </div>
                    </div>
                </div>
            </div>

            if *protocol == ProxyProtocol::Http {
                <HttpProxyConfig on_changed={http_proxy_on_changed} proxy={http_proxy} />
            } else {
                <TcpProxyConfig on_changed={tcp_proxy_on_changed} proxy={tcp_proxy} />
            }
        </>
    }
}

fn get_site(
    name: &str,
    ports: &[String],
    kind: &Result<ProxyKind, HashMap<String, String>>,
) -> Result<Proxy, HashMap<String, String>> {
    let mut errors = HashMap::new();
    let mut ports = ports.to_vec();
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
