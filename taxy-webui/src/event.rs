use crate::{
    store::{AcmeStore, CertStore, PortStore, SiteStore},
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
    let (_, sites) = use_store::<SiteStore>();
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
                                ports.set(PortStore { entries });
                            }
                            ServerEvent::ServerCertsUpdated { entries } => {
                                certs.set(CertStore { entries });
                            }
                            ServerEvent::AcmeUpdated { entries } => {
                                acme.set(AcmeStore { entries });
                            }
                            ServerEvent::SitesUpdated { entries } => {
                                sites.set(SiteStore { entries });
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
