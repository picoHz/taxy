use crate::{auth::UserSession, API_ENDPOINT};
use futures::StreamExt;
use gloo_net::eventsource::futures::EventSource;
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize, Store)]
struct EventSession {
    active: bool,
}

#[hook]
pub fn use_event_subscriber() {
    let (session, _) = use_store::<UserSession>();
    let (event, dispatcher) = use_store::<EventSession>();
    if !event.active {
        if let Some(token) = &session.token {
            let mut es = EventSource::new(&format!("{API_ENDPOINT}/events?token={token}")).unwrap();
            let mut stream = es.subscribe("message").unwrap();

            dispatcher.set(EventSession { active: true });
            spawn_local(async move {
                let _es = es;
                while let Some(Ok((event_type, msg))) = stream.next().await {
                    gloo_console::log!(&format!("1. {}: {:?}", event_type, msg))
                }
                gloo_console::log!("EventSource Closed");
                dispatcher.set(EventSession { active: false });
            })
        }
    }
}
