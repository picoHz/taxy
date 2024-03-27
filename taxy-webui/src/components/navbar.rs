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
            icon: "/assets/icons/wifi.svg",
            route: Route::Ports,
        },
        MenuItem {
            name: "Proxies",
            icon: "/assets/icons/swap-horizontal.svg",
            route: Route::Proxies,
        },
        MenuItem {
            name: "Certificates",
            icon: "/assets/icons/ribbon.svg",
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
        if gloo_dialogs::confirm("Are you sure to log out?") {
            navigator_cloned.push(&Route::Logout);
        }
    });

    let navigator_cloned = navigator.clone();
    let logo_onclick = Callback::from(move |e: MouseEvent| {
        e.prevent_default();
        navigator_cloned.push(&Route::Home);
    });

    if route == Route::Login {
        return html! {};
    }

    html! {
        <>
        <div class="lg:w-4/5 lg:mt-6 lg:rounded-md mx-auto text-neutral-100 bg-neutral-800 shadow-lg font-medium flex items-stretch">
            <div class="flex justify-start">
                <span class="lg:rounded-l-md inline-block cursor-pointer bg-yellow-300 text-md" onclick={logo_onclick}>
                    <span class="flex h-full justify-center items-center px-3">
                        <img src="/assets/logo.svg" class="object-center w-7 h-7" />
                    </span>
                </span>
                if let Some(root) = route.root() {
                    { ITEMS.iter().map(|entry| {
                        let navigator = navigator.clone();
                        let onclick = Callback::from(move |e: MouseEvent|  {
                            e.prevent_default();
                            navigator.push(&entry.route);
                        });
                        let is_active = root == entry.route;
                        let mut classes = vec!["px-4", "py-3", "border-neutral-800", "border-b-2", "inline-block", "cursor-pointer", "hover:bg-neutral-600", "text-md", "flex", "items-center"];
                        if is_active {
                            classes.push("border-b-neutral-100");
                            classes.push("bg-neutral-700");
                        }
                        html! {
                            <span class={classes!(classes)} onclick={onclick}>
                                <img src={entry.icon} class="w-5 h-5" />
                                <span class="ml-2 hidden md:inline">{entry.name}</span>
                            </span>
                        }
                    }).collect::<Html>() }
                }
            </div>
            <div class="flex justify-end ml-auto">
                <span class="lg:rounded-r-md px-4 py-3 inline-block cursor-pointer hover:bg-neutral-600 text-md flex items-center" onclick={logout_onclick}>
                    <img src="/assets/icons/log-out.svg" class="w-5 h-5" />
                    <span class="ml-2 hidden md:inline">{"Logout"}</span>
                </span>
            </div>
      </div>
        </>
    }
}
