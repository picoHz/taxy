use crate::auth::use_ensure_auth;
use crate::event::use_event_subscriber;
use crate::pages::Route;
use yew_router::prelude::*;
use yew::prelude::*;

#[function_component(Home)]
pub fn home() -> Html {
    use_ensure_auth();
    use_event_subscriber();

    html! {
        <Redirect<Route> to={Route::Ports}/>
    }
}
