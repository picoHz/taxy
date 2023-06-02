use crate::Route;
use gloo_net::http::Request;
use serde_derive::{Deserialize, Serialize};
// use taxy_api::auth::LoginRequest;
use ybc::TileCtx::{Ancestor, Parent};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[function_component(Secure)]
pub fn secure() -> Html {
    let navigator = use_navigator().unwrap();

    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let _login: LoginRequest = Request::post("http://127.0.0.1:46492/api/login")
                    .json(&LoginRequest {
                        username: "admin".to_string(),
                        password: "passw0rd".to_string(),
                    })
                    .unwrap()
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
            });
            || ()
        },
        (),
    );

    let onclick = Callback::from(move |_| navigator.push(&Route::Home));
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
                    </ybc::Field>
                </ybc::Tile>
            </ybc::Tile>
        </ybc::Tile>
        </ybc::Container>
    }
}
