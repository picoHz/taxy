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
            name: "Dashboard",
            icon: "grid",
            route: Route::Dashboard,
        },
        MenuItem {
            name: "Ports",
            icon: "wifi",
            route: Route::Ports,
        },
        MenuItem {
            name: "Proxies",
            icon: "swap-horizontal",
            route: Route::Proxies,
        },
        MenuItem {
            name: "Certificates",
            icon: "ribbon",
            route: Route::Certs,
        },
    ]
};

#[function_component(Navbar)]
pub fn navbar() -> Html {
    let navigator = use_navigator().unwrap();
    let route = use_route::<Route>().unwrap();

    let navigator_cloned = navigator.clone();
    let logout_onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        navigator_cloned.push(&Route::Logout);
    });

    html! {
        <ybc::Navbar
            classes={classes!("is-primary")}
            padded=true
            navburger={route != Route::Login}
            navbrand={html!{
                <span class="navbar-item taxy-logo">
                    <img src="/assets/logo.svg" class="is-48x48" alt="Taxy Logo" />
                </span>
            }}
            navstart={html!{
                if let Some(root) = route.root() {
                    <>
                    { ITEMS.iter().map(|entry| {
                        let navigator = navigator.clone();
                        let onclick = Callback::from(move |e: MouseEvent|  {
                            e.prevent_default();
                            navigator.push(&entry.route);
                        });
                        let is_active = root == entry.route;
                        html! {
                            <a class={classes!("navbar-item", "pr-5", is_active.then_some("is-active"))} {onclick}>
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
                }
            }}
            navend={html!{
                if route != Route::Login {
                    <a class="navbar-item" onclick={logout_onclick}>
                        <span class="icon-text">
                            <span class="icon">
                                <ion-icon name="exit"></ion-icon>
                            </span>
                            <span>{"Logout"}</span>
                        </span>
                    </a>
                }
            }}
        />
    }
}
