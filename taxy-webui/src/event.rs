use crate::{
    store::{AcmeStore, CertStore, PortStore, ProxyStore},
    API_ENDPOINT,
};
use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use gloo_timers::callback::Timeout;
use gloo_utils::format::JsValueSerdeExt;
use serde_derive::{Deserialize, Serialize};
use taxy_api::event::ServerEvent;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
struct EventSession {
    active: bool,
}

#[hook]
pub fn use_event_subscriber() {
    let (event, dispatcher) = use_store::<EventSession>();
    let (_, ports) = use_store::<PortStore>();
    let (_, certs) = use_store::<CertStore>();
    let (_, acme) = use_store::<AcmeStore>();
    let (_, proxies) = use_store::<ProxyStore>();
    if !event.active {
        let mut es = EventSource::new(&format!("{API_ENDPOINT}/events")).unwrap();
        let mut stream = es.subscribe("message").unwrap();

        dispatcher.set(EventSession { active: true });
        spawn_local(async move {
            let _es = es;
            while let Some(Ok((_, msg))) = stream.next().await {
                if let Ok(s) = msg.data().into_serde::<String>() {
                    if let Ok(event) = serde_json::from_str::<ServerEvent>(&s) {
                        match event {
                            ServerEvent::PortTableUpdated { entries } => {
                                ports.reduce(|state| {
                                    PortStore {
                                        entries,
                                        ..(*state).clone()
                                    }
                                    .into()
                                });
                            }
                            ServerEvent::CertsUpdated { entries } => {
                                certs.set(CertStore { entries });
                            }
                            ServerEvent::AcmeUpdated { entries } => {
                                acme.set(AcmeStore { entries });
                            }
                            ServerEvent::ProxiesUpdated { entries } => {
                                proxies.reduce(|state| {
                                    ProxyStore {
                                        entries,
                                        ..(*state).clone()
                                    }
                                    .into()
                                });
                            }
                            ServerEvent::PortStatusUpdated { id, status } => {
                                ports.reduce(|state| {
                                    let mut cloned = (*state).clone();
                                    cloned.statuses.insert(id, status);
                                    cloned.into()
                                });
                            }
                            ServerEvent::ProxyStatusUpdated { id, status } => {
                                proxies.reduce(|state| {
                                    let mut cloned = (*state).clone();
                                    cloned.statuses.insert(id, status);
                                    cloned.into()
                                });
                            }
                            _ => (),
                        }
                    }
                }
            }
            Timeout::new(5000, move || {
                dispatcher.set(EventSession { active: false });
            })
            .forget();
        })
    }
}
