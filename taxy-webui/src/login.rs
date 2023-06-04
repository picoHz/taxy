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
    let name = use_state(|| String::new());
   
    let name_clone = name.clone();
    let onclick: Callback<_> = Callback::from(move |_| {
        let navigator = navigator.clone();
        let dispatch = dispatch.clone();
        let name_clone = name_clone.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let login: ApiResult<LoginResult> = Request::post(&format!("{API_ENDPOINT}api/login"))
                .json(&LoginRequest {
                    username: name_clone.to_string(),
                    password: "adminx".to_string(),
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

    let oninput = Callback::from({
        let name = name.clone();
        move |input_event: InputEvent| {
            let target: HtmlInputElement = input_event
                .target()
                .unwrap_throw()
                .dyn_into()
                .unwrap_throw();
            gloo_console::log!(&format!("{:?}", target.value()));
            name.set(target.value());
        }
    });
    

    html! {
        <ybc::Container classes={classes!("is-centered")}>
        <ybc::Tile ctx={Ancestor}>
            <ybc::Tile ctx={Parent} size={ybc::TileSize::Twelve}>
                <ybc::Tile ctx={Parent}>
                    <ybc::Field>
                        <label class={classes!("label")}>{ "Username" }</label>
                        <div class={classes!("control")}>
                            <input class="input" type="text" {oninput} />
                        </div>
                        <label class={classes!("label")}>{ "Password" }</label>
                        <div class={classes!("control")}>
                            <input class="input" type="password" />
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
