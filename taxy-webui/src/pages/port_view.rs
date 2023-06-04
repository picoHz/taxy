use crate::auth::use_ensure_auth;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(PortView)]
pub fn port_view(props: &Props) -> Html {
    use_ensure_auth();

    html! {
        <h1>{props.id.clone()}</h1>
    }
}
