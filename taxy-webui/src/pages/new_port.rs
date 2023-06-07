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

            <div class="field is-horizontal m-5">
            <div class="field-label is-normal">
                <label class="label">{"Department"}</label>
            </div>
            <div class="field-body">
                <div class="field is-narrow">
                <div class="control">
                    <div class="select is-fullwidth">
                    <select>
                        <option>{"Business development"}</option>
                        <option>{"Marketing"}</option>
                        <option>{"Sales"}</option>
                    </select>
                    </div>
                </div>
                </div>
            </div>
            </div>


            <div class="field is-horizontal m-5">
            <div class="field-label is-normal">
              <label class="label">{"From"}</label>
            </div>
            <div class="field-body">
              <div class="field">
                <p class="control is-expanded has-icons-left">
                  <input class="input" type="text" placeholder="Name" />
                  <span class="icon is-small is-left">
                    <i class="fas fa-user"></i>
                  </span>
                </p>
              </div>
              <div class="field">
                <p class="control is-expanded has-icons-left has-icons-right">
                  <input class="input is-success" type="email" placeholder="Email" value="alex@smith.com" />
                  <span class="icon is-small is-left">
                    <i class="fas fa-envelope"></i>
                  </span>
                  <span class="icon is-small is-right">
                    <i class="fas fa-check"></i>
                  </span>
                </p>
              </div>
            </div>
          </div>

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
