use crate::{pages::Route, store::SessionStore};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(Logout)]
pub fn logout() -> Html {
    let (_, dispatch) = use_store::<SessionStore>();
    dispatch.set(SessionStore { token: None });

    html! {
        <Redirect<Route> to={Route::Login}/>
    }
}
