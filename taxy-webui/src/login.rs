use crate::Route;
use ybc::TileCtx::{Ancestor, Parent};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Secure)]
pub fn secure() -> Html {
    let navigator = use_navigator().unwrap();

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
