use crate::pages::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Breadcrumb)]
pub fn breadcrumb() -> Html {
    let navigator = use_navigator().unwrap();
    let route = use_route::<Route>().unwrap();

    html! {
        <ybc::Breadcrumb>
            <ul>
                { route.breadcrumb().into_iter().map(|item| {
                    let navigator = navigator.clone();
                    let onclick = Callback::from(move |e: MouseEvent|  {
                        e.prevent_default();
                        navigator.push(&item.route);
                    });
                    html! {
                        <li><a {onclick}>{&item.name}</a></li>
                    }
                }).collect::<Html>() }


            </ul>
        </ybc::Breadcrumb>
    }
}
