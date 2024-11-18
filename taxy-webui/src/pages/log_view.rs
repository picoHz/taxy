use crate::{auth::use_ensure_auth, API_ENDPOINT};
use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use taxy_api::log::{LogLevel, SystemLogRow};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use web_sys::Element;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

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
    use_effect_with((),move |_| {
        poll_log(id.clone(), ul_ref_cloned.clone(), log_cloned, vec![], None);
    });

    let navigator = use_navigator().unwrap();
    let back_onclick = Callback::from(move |_| {
        navigator.back();
    });

    html! {
        <>
            <div class="flex items-center justify-start px-4 lg:px-0 mb-4">
                <div>
                    <button onclick={back_onclick} class="inline-flex items-center text-neutral-500 dark:text-neutral-200 bg-white dark:bg-neutral-800 border border-neutral-300 dark:border-neutral-700 focus:outline-none hover:bg-neutral-100 hover:dark:bg-neutral-900 focus:ring-4 focus:ring-neutral-200 dark:focus:ring-neutral-600 font-medium rounded-lg text-sm px-4 py-2" type="button">
                        <img src="/assets/icons/arrow-back.svg" class="w-4 h-4 mr-1" />
                        {"Back"}
                    </button>
                </div>
            </div>
            <ul ref={ul_ref.clone()} class="overflow-scroll max-h-96 bg-white dark:bg-neutral-800 shadow-sm border border-neutral-300 dark:border-neutral-700 lg:rounded-md">
            { log.iter().map(|entry| {
                let timestamp = entry.timestamp.format(&Rfc3339).unwrap();
                let fields = entry.fields.iter().map(|(k, v)| {
                    format!("{}={}", k, v)
                }).collect::<Vec<String>>().join(" ");
                let log_class = match entry.level {
                    LogLevel::Error => "text-red-600",
                    LogLevel::Warn => "text-yellow-600",
                    LogLevel::Info => "text-green-600",
                    LogLevel::Debug => "text-blue-600",
                    LogLevel::Trace => "text-neutral-600",
                };
                html! {
                    <li class="font-mono text-sm px-4 py-1 border-b dark:border-neutral-700">
                        <span class="mr-2">{timestamp}</span>
                        <span class={classes!("font-bold", "mr-2", log_class)}>{
                            format!("{: <5}", entry.level.to_string().to_ascii_uppercase())
                        }</span>
                        <span class="mr-2">{entry.message.clone()}</span>
                        <span class="fields">{fields}</span>
                    </li>
                }
                }).collect::<Html>()
            }
            if log.is_empty() {
                <li class="mb-8 mt-8 text-xl font-bold text-neutral-500 px-16 text-center">{"No logs."}</li>
            }
            </ul>
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
                if elem.scroll_top() == elem.scroll_height() - elem.client_height() {
                    Timeout::new(0, move || {
                        elem.set_scroll_top(elem.scroll_height());
                    })
                    .forget();
                }
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
