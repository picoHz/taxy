use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub staging: bool,
}

#[function_component(LetsEncrypt)]
pub fn letsencrypt(_props: &Props) -> Html {
    html! {
        <>
            <div class="field is-horizontal m-5">
            <div class="field-label is-normal">
            <label class="label">{"Contact"}</label>
            </div>
            <div class="field-body">
                <div class="field">
                    <p class="control is-expanded">
                    <input class="input" type="email" placeholder="Email" />
                    </p>
                </div>
            </div>
            </div>
        </>
    }
}
