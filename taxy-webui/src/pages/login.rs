use crate::{auth::LoginQuery, pages::Route, API_ENDPOINT};
use gloo_net::http::Request;
use serde_derive::Deserialize;
use taxy_api::{
    auth::{LoginRequest, LoginResponse},
    error::ErrorMessage,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlInputElement, RequestCredentials};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResult<T> {
    Ok(T),
    Err(ErrorMessage),
}

#[function_component(Login)]
pub fn login() -> Html {
    let navigator = use_navigator().unwrap();

    let location = use_location().unwrap();
    let query: LoginQuery = location.query().unwrap_or_default();

    let username = use_state(String::new);
    let password = use_state(String::new);
    let error: UseStateHandle<Option<ErrorMessage>> = use_state(|| Option::<ErrorMessage>::None);

    let oninput_username = Callback::from({
        let username = username.clone();
        move |input_event: InputEvent| {
            let target: HtmlInputElement = input_event
                .target()
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw();
            username.set(target.value());
        }
    });

    let oninput_password = Callback::from({
        let password = password.clone();
        move |input_event: InputEvent| {
            let target: HtmlInputElement = input_event
                .target()
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw();
            password.set(target.value());
        }
    });

    let error_cloned = error.clone();
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();

        let navigator = navigator.clone();
        let username = username.clone();
        let password = password.clone();
        let query = query.clone();
        let error = error_cloned.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let login: ApiResult<LoginResponse> = Request::post(&format!("{API_ENDPOINT}/login"))
                .credentials(RequestCredentials::Include)
                .json(&LoginRequest {
                    username: username.to_string(),
                    password: password.to_string(),
                })
                .unwrap()
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            match login {
                ApiResult::Ok(_) => {
                    if let Some(redirect) = query.redirect {
                        navigator.replace(&redirect);
                    } else {
                        navigator.push(&Route::Home);
                    }
                }
                ApiResult::Err(err) => {
                    error.set(Some(err));
                }
            }
        });
    });

    html! {
        <ybc::Columns classes={classes!("is-centered")}>
            <ybc::Column classes={classes!("is-half")}>
                <ybc::Field>
                    if let Some(err) = &*error {
                        <article class="message is-danger">
                            <div class="message-header">
                                <p>{"Error"}</p>
                            </div>
                            <div class="message-body">
                                {&err.message}
                            </div>
                        </article>
                    }
                    <form {onsubmit}>
                        <label class={classes!("label", "mt-5")}>{ "Username" }</label>
                        <div class={classes!("control")}>
                            <input class="input" type="text" autocapitalize="off" oninput={oninput_username} />
                        </div>
                        <label class={classes!("label", "mt-5")}>{ "Password" }</label>
                        <div class={classes!("control")}>
                            <input class="input" type="password" oninput={oninput_password} />
                        </div>
                        <div class={classes!("control", "mt-5")}>
                            <input type="submit" value={"Login"} class={classes!("button", "is-primary", "is-fullwidth")} />
                        </div>
                    </form>
                </ybc::Field>
            </ybc::Column>
        </ybc::Columns>
    }
}
