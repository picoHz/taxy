use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlSelectElement;
use yew::prelude::*;

const PROTOCOLS: &[(&str, &str)] = &[
    ("tcp", "TCP"),
    ("tls", "TLS"),
    ("http", "HTTP"),
    ("https", "HTTPS"),
];

#[function_component(PortConfig)]
pub fn port_config() -> Html {
    let protocol = use_state(|| "tcp".to_string());

    let protocol_onchange = Callback::from({
        let protocol = protocol.clone();
        move |event: Event| {
            let target: HtmlSelectElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            protocol.set(target.value());
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
                        <input class="input" type="text" placeholder="Interface" value="0.0.0.0" />
                        </p>
                    </div>
                    <div class="field">
                        <p class="control is-expanded">
                        <input class="input" type="number" placeholder="Port" value="8080" max="65535" min="1" />
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
        </>
    }
}
