use crate::auth::use_ensure_auth;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(PortView)]
pub fn port_view(_props: &Props) -> Html {
    use_ensure_auth();

    html! {
        <ybc::Columns classes={classes!("is-centered")}>
            <ybc::Column classes={classes!("is-three-fifths-desktop")}>
                <div class="list has-visible-pointer-controls">
                    <div class="list-item">
                
                    <div class="list-item-content">
                        <div class="list-item-title">{"List item"}</div>
                        <div class="list-item-description">{"List item description"}</div>
                    </div>
                
                    <div class="list-item-controls">
                        <div class="buttons is-right">
                            <button class="button">
                                <span class="icon is-small">
                                    <ion-icon name="settings"></ion-icon>
                                </span>
                                <span>{"Edit"}</span>
                            </button>
                    
                            <button class="button">
                                <span class="icon is-small">
                                    <ion-icon name="ellipsis-horizontal"></ion-icon>
                                </span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </ybc::Column>
      </ybc::Columns>
    }
}
