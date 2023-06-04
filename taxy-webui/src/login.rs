use crate::{Route, UserSession, API_ENDPOINT};
use gloo_net::http::Request;
use serde_derive::Deserialize;
use taxy_api::{
    auth::{LoginRequest, LoginResult},
    error::ErrorMessage,
};
use wasm_bindgen::{UnwrapThrowExt, JsCast};
use ybc::TileCtx::{Ancestor, Parent};
use yew::prelude::*;
use web_sys::HtmlInputElement;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Deserialize)]
#[serde(untagged)]
enum ApiResult<T> {
    Ok(T),
    Err(ErrorMessage),
}

#[function_component(Login)]
pub fn login() -> Html {
    let (_, dispatch) = use_store::<UserSession>();
    let navigator = use_navigator().unwrap();

    let username = use_state(|| String::new());
    let password = use_state(|| String::new());

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
   
    let onclick: Callback<_> = Callback::from(move |_| {
        let navigator = navigator.clone();
        let dispatch = dispatch.clone();
        let username = username.clone();
        let password = password.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let login: ApiResult<LoginResult> = Request::post(&format!("{API_ENDPOINT}api/login"))
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
                ApiResult::Ok(login) => {
                    gloo_console::log!(&login.token);
                    dispatch.set(UserSession {
                        token: Some(login.token),
                    });
                }
                ApiResult::Err(err) => {
                    gloo_console::log!(&format!("{:?}", err.error));
                }
            }
            navigator.push(&Route::Login);
        });
    });
    

    html! {
        <ybc::Container classes={classes!("is-centered")}>
        <ybc::Tile ctx={Ancestor}>
            <ybc::Tile ctx={Parent} size={ybc::TileSize::Twelve}>
                <ybc::Tile ctx={Parent}>
                    <ybc::Field>
                        <label class={classes!("label")}>{ "Username" }</label>
                        <div class={classes!("control")}>
                            <input class="input" type="text" oninput={oninput_username} />
                        </div>
                        <label class={classes!("label")}>{ "Password" }</label>
                        <div class={classes!("control")}>
                            <input class="input" type="password" oninput={oninput_password} />
                        </div>
                        <div class={classes!("control")}>
                            <button class={classes!("button", "is-primary")} {onclick}>{ "Go Home" }</button>
                        </div>
                    </ybc::Field>
                </ybc::Tile>
            </ybc::Tile>
        </ybc::Tile>
        </ybc::Container>
    }
}
