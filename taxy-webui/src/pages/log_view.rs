use crate::{auth::use_ensure_auth, components::breadcrumb::Breadcrumb, API_ENDPOINT};
use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use taxy_api::log::SystemLogRow;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use web_sys::Element;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(LogView)]
pub fn log_view(props: &Props) -> Html {
    use_ensure_auth();

    let ul_ref = use_node_ref();

    let log: UseStateHandle<Vec<SystemLogRow>> = use_state(Vec::<SystemLogRow>::new);
    let id = props.id.clone();
    let log_cloned = log.clone();
    let ul_ref_cloned = ul_ref.clone();
    use_effect_with_deps(
        move |_| {
            poll_log(
                id.clone(),
                ul_ref_cloned.clone(),
                log_cloned.clone(),
                vec![],
                None,
            );
        },
        (),
    );

    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>

            <ul ref={ul_ref.clone()} class="log-viewer">
            { log.iter().map(|entry| {
                let timestamp = entry.timestamp.format(&Rfc3339).unwrap();
                let fields = entry.fields.iter().map(|(k, v)| {
                    format!("{}={}", k, v)
                }).collect::<Vec<String>>().join(" ");
                html! {
                    <li class="log">
                        <span class="timestamp">{timestamp}</span>
                        <span class={classes!("loglevel", entry.level.to_string())}>{
                            format!("{: <5}", entry.level.to_string().to_ascii_uppercase())
                        }</span>
                        <span class="logmessage">{entry.message.clone()}</span>
                        <span class="fields">{fields}</span>
                    </li>
                }
                }).collect::<Html>()
            }
            </ul>

            </ybc::Card>
        </>
    }
}

fn poll_log(
    id: String,
    ul_ref: NodeRef,
    setter: UseStateHandle<Vec<SystemLogRow>>,
    mut history: Vec<SystemLogRow>,
    time: Option<OffsetDateTime>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(mut list) = get_log(&id, time).await {
            let time = list.last().map(|row| row.timestamp).or(time);
            history.append(&mut list);
            setter.set(history.clone());

            if let Some(elem) = ul_ref.cast::<Element>() {
                Timeout::new(0, move || {
                    elem.set_scroll_top(elem.scroll_height());
                })
                .forget();
                poll_log(id, ul_ref, setter, history, time);
            }
        }
    });
}

async fn get_log(
    id: &str,
    time: Option<OffsetDateTime>,
) -> Result<Vec<SystemLogRow>, gloo_net::Error> {
    let mut req = Request::get(&format!("{API_ENDPOINT}/logs/{id}"));
    if let Some(time) = time {
        req = req.query([("since", &time.unix_timestamp().to_string())]);
    }
    req.send().await?.json().await
}
