use crate::{
    auth::use_ensure_auth,
    components::{breadcrumb::Breadcrumb, port_config::PortConfig},
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: String,
}

#[function_component(PortView)]
pub fn port_view(_props: &Props) -> Html {
    use_ensure_auth();

    html! {
        <>
            <ybc::Card>
            <ybc::CardHeader>
                <p class="card-header-title">
                    <Breadcrumb />
                </p>
            </ybc::CardHeader>

            <PortConfig />

            <ybc::CardFooter>
                <a class="card-footer-item">
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
                    <span>{"Update"}</span>
                    </span>
                </a>
            </ybc::CardFooter>

            </ybc::Card>
        </>
    }
}
