use crate::pages::Route;
use yew::prelude::*;
use yew_router::prelude::*;

struct MenuItem {
    name: &'static str,
    icon: &'static str,
    route: Route,
}

const ITEMS: &[MenuItem] = {
    use Route::*;
    &[MenuItem {
        name: "Ports",
        icon: "wifi",
        route: Ports,
    }]
};

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let navigator = use_navigator().unwrap();
    let route = use_route::<Route>().unwrap();
    if route == Route::Login {
        return html! {};
    }

    let navigator_cloned = navigator.clone();
    let onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        navigator_cloned.push(&Route::Home);
    });

    html! {
        <ybc::Navbar
            classes={classes!("is-success")}
            padded=true
            navbrand={html!{
                <a class="navbar-item" {onclick}>
                    <ybc::Title classes={classes!("has-text-white")} size={ybc::HeaderSize::Is4}>{"Taxy"}</ybc::Title>
                </a>
            }}
            navstart={html!{
                <>
                { ITEMS.into_iter().map(|entry| {
                    let navigator = navigator.clone();
                    let onclick = Callback::from(move |e: MouseEvent|  {
                        e.prevent_default();
                        navigator.push(&entry.route);
                    });
                    html! {
                        <a class="navbar-item" {onclick}>
                            <span class="icon-text">
                                <span class="icon">
                                    <ion-icon name={entry.icon}></ion-icon>
                                </span>
                                <span>{entry.name}</span>
                            </span>
                        </a>
                    }
                }).collect::<Html>() }
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
