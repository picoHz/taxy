use crate::{Route, UserSession};
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[hook]
pub fn use_ensure_auth() {
    let navigator = use_navigator().unwrap();
    let (session, _) = use_store::<UserSession>();
    if session.token.is_empty() {
        navigator.replace(&Route::Login);
    }
}
