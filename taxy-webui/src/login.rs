use crate::{Route, UserSession};
use gloo_net::http::Request;
use serde_derive::Deserialize;
use taxy_api::auth::{LoginRequest, LoginResult};
use ybc::TileCtx::{Ancestor, Parent};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Deserialize)]
struct Er {
    message: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Res<T, E> {
    Ok(T),
    Err(E),
}

#[hook]
pub fn use_x() {
    if let Some(route) = use_route::<Route>() {
        gloo_console::log!(&format!("{:?}", route));
    }
    let (_, dispatch) = use_store::<UserSession>();
    let navigator = use_navigator().unwrap();

    use_effect_with_deps(
        move |_| {
            let dispatch = dispatch.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let login: Res<LoginResult, Er> = Request::post("http://127.0.0.1:46492/api/login")
                    .json(&LoginRequest {
                        username: "admin".to_string(),
                        password: "admin".to_string(),
                    })
                    .unwrap()
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                if let Res::Ok(login) = login {
                    gloo_console::log!(&login.token);
                    // dispatch.set(UserSession { token: login.token });
                }
            });
        },
        (),
    );
}

#[function_component(Login)]
pub fn login() -> Html {
    use_x();

    let (_, dispatch) = use_store::<UserSession>();
    let navigator = use_navigator().unwrap();

    let onclick: Callback<_> = Callback::from(move |_| {
        let navigator = navigator.clone();
        let dispatch = dispatch.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let login: Res<LoginResult, Er> = Request::post("http://127.0.0.1:46492/api/login")
                .json(&LoginRequest {
                    username: "admin".to_string(),
                    password: "admin".to_string(),
                })
                .unwrap()
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if let Res::Ok(login) = login {
                gloo_console::log!(&login.token);
                dispatch.set(UserSession {
                    token: Some(login.token),
                });
            }
            navigator.push(&Route::Login);
        });
    });

    let navigator = use_navigator().unwrap();
    let onclick2: Callback<_> = Callback::from(move |_| {
        navigator.push(&Route::Ports);
    });

    html! {
        <ybc::Container classes={classes!("is-centered")}>
        <ybc::Tile ctx={Ancestor}>
            <ybc::Tile ctx={Parent} size={ybc::TileSize::Twelve}>
                <ybc::Tile ctx={Parent}>
                    <ybc::Field>
                        <label class={classes!("label")}>{ "Username" }</label>
                        <div class={classes!("control")}>
                            <input class="input" type="text" />
                        </div>
                        <label class={classes!("label")}>{ "Password" }</label>
                        <div class={classes!("control")}>
                            <input class="input" type="password" />
                        </div>
                        <div class={classes!("control")}>
                            <button class={classes!("button", "is-primary")} {onclick}>{ "Go Home" }</button>
                        </div>
                        <div class={classes!("control")}>
                            <button class={classes!("button", "is-primary")} onclick={onclick2}>{ "Go Home" }</button>
                        </div>
                    </ybc::Field>
                </ybc::Tile>
            </ybc::Tile>
        </ybc::Tile>
        </ybc::Container>
    }
}
