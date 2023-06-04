use crate::auth::use_ensure_auth;
use crate::event::use_event_subscriber;
use yew::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    use_ensure_auth();
    use_event_subscriber();

    html! {
        <h1>{"Home"}</h1>
    }
}
