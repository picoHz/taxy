use crate::{
    auth::{test_token, LoginQuery},
    pages::Route,
    API_ENDPOINT,
};
use gloo_events::EventListener;
use gloo_net::http::Request;
use serde_derive::Deserialize;
use taxy_api::{
    auth::{LoginMethod, LoginRequest, LoginResponse},
    error::ErrorMessage,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResult<T> {
    Ok(T),
    Err(ErrorMessage),
}

#[wasm_bindgen(module = "/js/logout.js")]
extern "C" {
    fn logout();
}

#[function_component(Login)]
pub fn login() -> Html {
    let navigator = use_navigator().unwrap();

    use_effect_with_deps(
        move |_| {
            EventListener::new(&gloo_utils::document(), "visibilitychange", move |_event| {
                wasm_bindgen_futures::spawn_local(async move {
                    if !test_token().await {
                        logout();
                    }
                });
            })
            .forget();
        },
        (),
    );

    let location = use_location().unwrap();
    let query: LoginQuery = location.query().unwrap_or_default();

    let username = use_state(String::new);
    let password = use_state(String::new);
    let totp = use_state(|| Option::<String>::None);
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

    let totp_cloned = totp.clone();
    let oninput_totp = Callback::from({
        let totp = totp_cloned.clone();
        move |input_event: InputEvent| {
            let target: HtmlInputElement = input_event
                .target()
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw();
            totp.set(Some(target.value()));
        }
    });

    let totp_cloned = totp.clone();
    let error_cloned = error.clone();
    let username_cloned = username.clone();
    let password_cloned = password.clone();
    let onsubmit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();

        let navigator = navigator.clone();
        let username = username_cloned.clone();
        let password = password_cloned.clone();
        let totp = totp_cloned.clone();
        let query = query.clone();
        let error = error_cloned.clone();

        let method = if let Some(totp) = &*totp {
            LoginMethod::Totp {
                token: totp.to_string(),
            }
        } else {
            LoginMethod::Password {
                password: password.to_string(),
            }
        };

        wasm_bindgen_futures::spawn_local(async move {
            let login: ApiResult<LoginResponse> = Request::post(&format!("{API_ENDPOINT}/login"))
                .json(&LoginRequest {
                    username: username.to_string(),
                    method,
                })
                .unwrap()
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            match login {
                ApiResult::Ok(LoginResponse::Success) => {
                    if let Some(redirect) = query.redirect {
                        navigator.replace(&redirect);
                    } else {
                        navigator.push(&Route::Dashboard);
                    }
                }
                ApiResult::Ok(LoginResponse::TotpRequired) => totp.set(Some(String::new())),
                ApiResult::Err(err) => {
                    error.set(Some(err));
                }
            }
        });
    });

    html! {
        <ybc::Columns classes={classes!("is-centered")}>
            <ybc::Column classes={classes!("login-form")}>
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
                        if let Some(totp) = &*totp {
                            <label class={classes!("label", "mt-5")}>{ "One Time Password" }</label>
                            <div class={classes!("control")}>
                                <input class="input" type="number" value={totp.to_string()} autocapitalize="off" oninput={oninput_totp} />
                            </div>
                            <div class={classes!("control", "mt-5")}>
                                <input type="submit" value={"Continue"} disabled={totp.is_empty()} class={classes!("button", "is-primary", "is-fullwidth")} />
                            </div>
                        } else {
                            <label class={classes!("label", "mt-5")}>{ "Username" }</label>
                            <div class={classes!("control")}>
                                <input class="input" type="text" autocapitalize="off" oninput={oninput_username} />
                            </div>
                            <label class={classes!("label", "mt-5")}>{ "Password" }</label>
                            <div class={classes!("control")}>
                                <input class="input" type="password" oninput={oninput_password} />
                            </div>
                            <div class={classes!("control", "mt-5")}>
                                <input type="submit" value={"Login"} disabled={username.is_empty() || password.is_empty()} class={classes!("button", "is-primary", "is-fullwidth")} />
                            </div>
                        }
                    </form>
                </ybc::Field>
            </ybc::Column>
        </ybc::Columns>
    }
}
