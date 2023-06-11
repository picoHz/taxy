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
            <label class="label">{"Email Address"}</label>
            </div>
            <div class="field-body">
                <div class="field">
                    <p class="control is-expanded">
                    <input class="input" type="email" placeholder="admin@example.com" />
                    </p>
                </div>
            </div>
            </div>

            <div class="field is-horizontal m-5">
                <div class="field-label is-normal">
                    <label class="label">{"Challenge"}</label>
                </div>
                <div class="field-body">
                    <div class="field is-narrow">
                    <div class="control">
                        <div class="select is-fullwidth">
                        <select>
                            <option selected={true}>{"HTTP"}</option>
                        </select>
                        </div>
                    </div>
                    </div>
                </div>
            </div>

            <div class="field is-horizontal m-5">
            <div class="field-label is-normal">
            <label class="label">{"Domain Name"}</label>
            </div>
            <div class="field-body">
                <div class="field">
                    <p class="control is-expanded">
                    <input class="input" type="input" placeholder="example.com" />
                    </p>
                </div>
            </div>
            </div>
        </>
    }
}
