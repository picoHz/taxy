use std::collections::HashMap;
use taxy_api::{
    acme::{Acme, AcmeRequest},
    subject_name::SubjectName,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub staging: bool,
    pub on_changed: Callback<Result<AcmeRequest, HashMap<String, String>>>,
}

#[function_component(LetsEncrypt)]
pub fn letsencrypt(props: &Props) -> Html {
    let email = use_state(String::new);
    let email_onchange = Callback::from({
        let email = email.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            email.set(target.value());
        }
    });

    let domain_name = use_state(String::new);
    let domain_name_onchange = Callback::from({
        let domain_name = domain_name.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            domain_name.set(target.value());
        }
    });

    let prev_entry =
        use_state::<Result<AcmeRequest, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry = get_request(&email, &domain_name, props.staging);
    if entry != *prev_entry {
        prev_entry.set(entry.clone());
        props.on_changed.emit(entry);
    }

    html! {
        <>
            <div class="field is-horizontal m-5">
            <div class="field-label is-normal">
            <label class="label">{"Email Address"}</label>
            </div>
            <div class="field-body">
                <div class="field">
                    <p class="control is-expanded">
                    <input class="input" type="email" placeholder="admin@example.com" onchange={email_onchange} />
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
                    <input class="input" type="input" placeholder="example.com" onchange={domain_name_onchange} />
                    </p>
                </div>
            </div>
            </div>
        </>
    }
}

fn get_request(
    email: &str,
    domain_name: &str,
    staging: bool,
) -> Result<AcmeRequest, HashMap<String, String>> {
    let mut errors = HashMap::new();
    if email.is_empty() {
        errors.insert("email".to_string(), "Email is required".to_string());
    }
    if domain_name.is_empty() {
        errors.insert(
            "domain_name".to_string(),
            "Domain name is required".to_string(),
        );
    }
    let domain_name: SubjectName = match domain_name.parse() {
        Ok(domain_name) => domain_name,
        Err(err) => {
            errors.insert("domain_name".to_string(), err.to_string());
            return Err(errors);
        }
    };
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(AcmeRequest {
        server_url: if staging {
            "https://acme-staging-v02.api.letsencrypt.org/directory".to_string()
        } else {
            "https://acme-v02.api.letsencrypt.org/directory".to_string()
        },
        contacts: vec![format!("mailto:{}", email)],
        eab: None,
        acme: Acme {
            provider: if staging {
                "Let's Encrypt (Staging)".to_string()
            } else {
                "Let's Encrypt".to_string()
            },
            identifiers: vec![domain_name],
            challenge_type: "http-01".to_string(),
            renewal_days: 60,
            is_trusted: !staging,
        },
    })
}
