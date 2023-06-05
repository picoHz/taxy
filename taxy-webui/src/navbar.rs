use crate::pages::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let route = use_route::<Route>().unwrap();
    if route == Route::Login {
        return html! {};
    }
    html! {
        <ybc::Navbar
            classes={classes!("is-success")}
            padded=true
            navbrand={html!{
                <ybc::NavbarItem>
                    <ybc::Title classes={classes!("has-text-white")} size={ybc::HeaderSize::Is4}>{"Taxy"}</ybc::Title>
                </ybc::NavbarItem>
            }}
            navstart={html!{
                <>
                <ybc::NavbarItem tag={ybc::NavbarItemTag::A}>
                    <span class="icon-text">
                        <span class="icon">
                            <ion-icon name="wifi"></ion-icon>
                        </span>
                        <span>{"Ports"}</span>
                    </span>
                </ybc::NavbarItem>
                <ybc::NavbarItem>
                    <ybc::ButtonAnchor classes={classes!("is-text")} rel={String::from("noopener noreferrer")} target={String::from("_blank")} href="https://yew.rs">
                        {"Yew"}
                    </ybc::ButtonAnchor>
                </ybc::NavbarItem>
                </>
            }}
            navend={html!{}}
        />
    }
}
