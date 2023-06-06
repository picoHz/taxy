use crate::{auth::use_ensure_auth, breadcrumb::Breadcrumb};
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
            </ybc::Card>
        </>
    }
}
