use crate::{Route, UserSession};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(Logout)]
pub fn logout() -> Html {
    let (_, dispatch) = use_store::<UserSession>();
    dispatch.set(UserSession {
        token: "".to_string(),
    });

    html! {
        <Redirect<Route> to={Route::Login}/>
    }
}
