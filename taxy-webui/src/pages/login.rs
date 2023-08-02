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
                        navigator.push(&Route::Home);
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
        <>
        <form class="mx-auto max-w-sm mt-4" {onsubmit}>
            <div class="mx-auto flex w-full justify-center items-center mb-2">
                <img class="w-8 h-8" src="/assets/logo.svg" />
            </div>
            <div class="mx-auto flex w-full justify-center items-center mb-5">
                <h1 class="font-semibold text-2xl text-neutral-700">{"Taxy Admin"}</h1>
            </div>

            if let Some(err) = &*error {
                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4" role="alert">
                    <span class="block sm:inline">{&err.message}</span>
                </div>
            }

            if let Some(totp) = &*totp {
                <label class="mr-4 text-neutral-700 font-bold inline-block mb-2" for="name">{"One Time Password"}</label>
                <input type="number" class="border bg-white py-2 px-4 w-full outline-none focus:ring-2 focus:ring-neutral-400 rounded" oninput={oninput_totp} />
                <input type="submit" class="w-full mt-4 text-neutral-50 font-bold bg-neutral-800 py-3 rounded-md hover:bg-neutral-600 transition duration-300" value={"Continue"} disabled={totp.is_empty()} />
            } else {
                <div class="mb-4">
                    <label class="mr-4 text-neutral-700 font-bold inline-block mb-2" for="name">{"Username"}</label>
                    <input type="text" class="border bg-white py-2 px-4 w-full outline-none focus:ring-2 focus:ring-neutral-400 rounded" autocapitalize="off" autofocus={true} oninput={oninput_username} />
                </div>
                <label class="mr-4 text-neutral-700 font-bold inline-block mb-2" for="name">{"Password"}</label>
                <input type="password" class="border bg-white py-2 px-4 w-full outline-none focus:ring-2 focus:ring-neutral-400 rounded" oninput={oninput_password} />
                <input type="submit" class="w-full mt-4 text-neutral-50 font-bold bg-neutral-800 py-3 rounded-md hover:bg-neutral-600 transition duration-300" value={"Login"} disabled={username.is_empty() || password.is_empty()} />
            }
        </form>
        </>
    }
}
