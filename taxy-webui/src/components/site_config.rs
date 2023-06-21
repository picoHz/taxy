use crate::store::PortStore;
use crate::API_ENDPOINT;
use gloo_net::http::Request;
use multiaddr::Protocol;
use std::collections::HashMap;
use std::str::FromStr;
use taxy_api::site::{Route, Server};
use taxy_api::subject_name::SubjectName;
use taxy_api::{port::PortEntry, site::Site};
use url::Url;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, HtmlOptionElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_else(create_default_port)]
    pub site: Site,
    pub on_changed: Callback<Result<Site, HashMap<String, String>>>,
}

fn create_default_port() -> Site {
    Site {
        ports: vec![],
        vhosts: vec![],
        routes: vec![],
    }
}

#[function_component(SiteConfig)]
pub fn site_config(props: &Props) -> Html {
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

    let bound_ports = use_state(|| props.site.ports.clone());
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

    let vhosts = use_state(|| {
        props
            .site
            .vhosts
            .iter()
            .map(|host| host.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    });
    let vhosts_onchange = Callback::from({
        let vhosts = vhosts.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            vhosts.set(target.value());
        }
    });

    let routes = use_state(|| {
        props
            .site
            .routes
            .iter()
            .map(|route| {
                (
                    route.path.clone(),
                    route
                        .servers
                        .iter()
                        .map(|server| server.url.to_string())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    });

    if routes.is_empty() {
        routes.set(vec![("/".into(), Vec::new())]);
    }

    let prev_entry =
        use_state::<Result<Site, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry = get_site(&bound_ports, &vhosts, &routes);

    if entry != *prev_entry {
        prev_entry.set(entry.clone());
        props.on_changed.emit(entry);
    }

    let err = Default::default();
    let errors = prev_entry.as_ref().err().unwrap_or(&err);
    let vhosts_err = errors.get("vhosts").map(|e| e.as_str());

    let http_ports = ports.entries.iter().filter(|entry| {
        entry
            .port
            .listen
            .iter()
            .any(|protocol| matches!(protocol, Protocol::Http | Protocol::Https))
    });

    html! {
        <>
            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                <label class="label">{"Ports"}</label>
                </div>
                <div class="field-body">
                    <div class="field">
                        <div class="select is-multiple is-fullwidth">
                            <select class="is-expanded" multiple={true} size="3" onchange={bound_ports_onchange}>
                                { http_ports.map(|entry| {
                                    html! {
                                        <option selected={bound_ports.contains(&entry.id)} value={entry.id.clone()}>
                                            {entry.port.listen.to_string()}
                                        </option>
                                    }
                                }).collect::<Html>() }
                            </select>
                        </div>
                    </div>
                </div>
            </div>

            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                <label class="label">{"Virtual Hosts"}</label>
                </div>
                <div class="field-body">
                    <div class="field">
                        <p class="control is-expanded">
                        <input class="input" type="text" placeholder="Host Names" autocapitalize="off" value={vhosts.to_string()} onchange={vhosts_onchange} />
                        </p>
                        if let Some(err) = vhosts_err {
                            <p class="help is-danger">{err}</p>
                        } else {
                            <p class="help">
                            {"You can use commas to list multiple names, e.g, example.com, *.test.examle.com."}
                            </p>
                        }
                    </div>
                </div>
            </div>

            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                <label class="label">{"Routes"}</label>
                </div>

                <div class="is-flex-grow-5" style="flex-basis: 0">

                { routes.iter().enumerate().map(|(i, (path, servers))| {
                    let routes_len = routes.len();

                    let routes_cloned = routes.clone();
                    let add_onclick = Callback::from(move |_| {
                        let mut routes = (*routes_cloned).clone();
                        routes.insert(i + 1, ("/".into(), Vec::new()));
                        routes_cloned.set(routes);
                    });

                    let routes_cloned = routes.clone();
                    let remove_onclick = Callback::from(move |_| {
                        if routes_len > 1 {
                            let mut routes = (*routes_cloned).clone();
                            routes.remove(i);
                            routes_cloned.set(routes);
                        }
                    });

                    let routes_cloned = routes.clone();
                    let path_onchange = Callback::from(move |event: Event| {
                        let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                        let mut routes = (*routes_cloned).clone();
                        routes[i].0 = target.value();
                        routes_cloned.set(routes);
                    });

                    let routes_cloned = routes.clone();
                    let servers_onchange = Callback::from(move |event: Event| {
                        let target: HtmlTextAreaElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
                        let mut routes = (*routes_cloned).clone();
                        routes[i].1 = target.value().split('\n').map(|s| s.to_string()).collect();
                        routes_cloned.set(routes);
                    });

                    let not_first = i > 0;
                    let err = errors.get(&format!("routes_{}", i)).map(|s| s.as_str());

                    html! {
                        <div class={classes!(not_first.then_some("mt-3"))}>
                            <div class={classes!("field-body")}>
                            <div class="field has-addons">
                                <div class="control is-expanded">
                                    <input class={classes!("input", err.map(|_| "is-danger"))} type="text" autocapitalize="off" placeholder="Path" onchange={path_onchange} value={path.clone()} />
                                </div>
                                <div class="control">
                                    <button class={classes!("button", err.map(|_| "is-danger"))} onclick={add_onclick}>
                                        <span class="icon">
                                            <ion-icon name="add"></ion-icon>
                                        </span>
                                    </button>
                                </div>
                                <div class="control">
                                    <button class={classes!("button", err.map(|_| "is-danger"))} onclick={remove_onclick} disabled={routes_len <= 1}>
                                        <span class="icon">
                                            <ion-icon name="remove"></ion-icon>
                                        </span>
                                    </button>
                                </div>
                            </div>
                        </div>
                        <div class="mt-2">
                            <div class="field">
                                <div class="control">
                                    <textarea class={classes!("textarea", err.map(|_| "is-danger"))} autocapitalize="off" placeholder="https://example.com/backend" onchange={servers_onchange} value={servers.join("\n").to_string()}></textarea>
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

fn get_site(
    ports: &[String],
    vhosts: &str,
    routes: &[(String, Vec<String>)],
) -> Result<Site, HashMap<String, String>> {
    let mut errors = HashMap::new();
    let mut ports = ports.to_vec();
    ports.sort();
    ports.dedup();
    let mut hosts = Vec::new();
    for host in vhosts
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        match SubjectName::from_str(&host) {
            Ok(host) => hosts.push(host),
            Err(err) => {
                errors.insert("vhosts".into(), err.to_string());
            }
        }
    }
    let mut parsed_routes = Vec::new();
    for (i, route) in routes.iter().enumerate() {
        let path = route.0.clone();
        if !path.starts_with('/') {
            errors.insert(format!("routes_{}", i), "Path must start with /".into());
            continue;
        }
        let servers = route.1.clone();
        let mut urls = Vec::new();
        for url in servers {
            match Url::from_str(&url) {
                Ok(url) => urls.push(Server { url }),
                Err(err) => {
                    errors.insert(format!("routes_{}", i), err.to_string());
                }
            }
        }
        parsed_routes.push(Route {
            path,
            servers: urls,
        });
    }

    if errors.is_empty() {
        Ok(Site {
            ports,
            vhosts: hosts,
            routes: parsed_routes,
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
