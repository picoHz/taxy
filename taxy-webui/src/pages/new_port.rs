use crate::{auth::use_ensure_auth, components::breadcrumb::Breadcrumb, pages::Route};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(NewPort)]
pub fn new_port() -> Html {
    use_ensure_auth();

    let navigator = use_navigator().unwrap();

    let navigator_cloned = navigator.clone();
    let cancel_onclick = Callback::from(move |_| {
        navigator_cloned.push(&Route::Ports);
    });

    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>
            <ybc::CardFooter>
                <a class="card-footer-item" onclick={cancel_onclick}>
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="close"></ion-icon>
                    </span>
                    <span>{"Cancel"}</span>
                    </span>
                </a>
                <a class="card-footer-item">
                    <span class="icon-text">
                    <span class="icon">
                        <ion-icon name="checkmark"></ion-icon>
                    </span>
                    <span>{"Create"}</span>
                    </span>
                </a>
            </ybc::CardFooter>
            </ybc::Card>
        </>
    }
}
