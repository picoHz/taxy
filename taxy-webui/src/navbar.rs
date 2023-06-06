use crate::pages::Route;
use yew::prelude::*;
use yew_router::prelude::*;

struct MenuItem {
    name: &'static str,
    icon: &'static str,
    route: Route,
}

const ITEMS: &[MenuItem] = {
    &[
        MenuItem {
            name: "Ports",
            icon: "wifi",
            route: Route::Ports,
        },
        MenuItem {
            name: "Sites",
            icon: "globe",
            route: Route::Sites,
        },
        MenuItem {
            name: "Certs",
            icon: "ribbon",
            route: Route::Certs,
        },
    ]
};

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let navigator = use_navigator().unwrap();
    let route = use_route::<Route>().unwrap();
    if route == Route::Login {
        return html! {};
    }

    let navigator_cloned = navigator.clone();
    let home_onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        navigator_cloned.push(&Route::Home);
    });

    let navigator_cloned = navigator.clone();
    let logout_onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        navigator_cloned.push(&Route::Logout);
    });

    html! {
        <ybc::Navbar
            classes={classes!("is-light")}
            padded=true
            navbrand={html!{
                <a class="navbar-item" onclick={home_onclick}>
                    <ybc::Title size={ybc::HeaderSize::Is4}>{"Taxy"}</ybc::Title>
                </a>
            }}
            navstart={html!{
                <>
                { ITEMS.iter().map(|entry| {
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
                </>
            }}
            navend={html!{
                <a class="navbar-item" onclick={logout_onclick}>
                    <span class="icon-text">
                        <span class="icon">
                            <ion-icon name="exit"></ion-icon>
                        </span>
                        <span>{"Logout"}</span>
                    </span>
                </a>
            }}
        />
    }
}
