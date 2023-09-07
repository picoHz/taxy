use base64::{engine::general_purpose, Engine};
use std::collections::HashMap;
use taxy_api::{
    acme::{Acme, AcmeConfig, AcmeRequest, ExternalAccountBinding},
    subject_name::SubjectName,
};
use url::Url;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub onchanged: Callback<Result<AcmeRequest, HashMap<String, String>>>,
}

#[function_component(CustomAcme)]
pub fn custom_acme(props: &Props) -> Html {
    let name = use_state(String::new);
    let name_onchange = Callback::from({
        let name = name.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            name.set(target.value());
        }
    });

    let server_url = use_state(String::new);
    let server_url_onchange = Callback::from({
        let server_url = server_url.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            server_url.set(target.value());
        }
    });

    let eab_kid = use_state(String::new);
    let eab_kid_onchange = Callback::from({
        let eab_kid: UseStateHandle<String> = eab_kid.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            eab_kid.set(target.value());
        }
    });

    let eab_hmac_key = use_state(String::new);
    let eab_hmac_key_onchange = Callback::from({
        let eab_hmac_key: UseStateHandle<String> = eab_hmac_key.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            eab_hmac_key.set(target.value());
        }
    });

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

    let renewal = use_state(|| 60);
    let renewal_onchange = Callback::from({
        let renewal = renewal.clone();
        move |event: Event| {
            let target: HtmlInputElement = event.target().unwrap_throw().dyn_into().unwrap_throw();
            renewal.set(target.value().parse().unwrap_or(60));
        }
    });

    let prev_entry =
        use_state::<Result<AcmeRequest, HashMap<String, String>>, _>(|| Err(Default::default()));
    let entry = get_request(
        &name,
        &server_url,
        &eab_kid,
        &eab_hmac_key,
        &email,
        &domain_name,
        *renewal,
    );
    if entry != *prev_entry {
        prev_entry.set(entry.clone());
        props.onchanged.emit(entry);
    }

    html! {
        <>
            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Name"}</label>
            <input type="text" placeholder="ACME Provider" onchange={name_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Server URL"}</label>
            <input type="url" placeholder="https://example.com/" onchange={server_url_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"EAB Key ID (Optional)"}</label>
            <input type="text" onchange={eab_kid_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"EAB HMAC Key (Optional)"}</label>
            <input type="text" onchange={eab_hmac_key_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Email Address"}</label>
            <input type="email" placeholder="admin@example.com" onchange={email_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Challenge"}</label>
            <select class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5">
                <option selected={true}>{"HTTP"}</option>
            </select>

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Domain Name"}</label>
            <input type="taxt" autocapitalize="off" placeholder="example.com" onchange={domain_name_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />

            <label class="block mt-4 mb-2 text-sm font-medium text-neutral-900">{"Renewal Interval (days)"}</label>
            <input type="number" value={renewal.to_string()} min="1" onchange={renewal_onchange} class="bg-neutral-50 border border-neutral-300 text-neutral-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5" />
        </>
    }
}

fn get_request(
    name: &str,
    server_url: &str,
    eab_kid: &str,
    eab_hmac_key: &str,
    email: &str,
    domain_name: &str,
    renewal: u64,
) -> Result<AcmeRequest, HashMap<String, String>> {
    let mut errors = HashMap::new();

    let eab_kid = eab_kid.trim();
    let eab_hmac_key = eab_hmac_key.trim();
    let eab = if !eab_kid.is_empty() || !eab_hmac_key.is_empty() {
        if eab_kid.is_empty() {
            errors.insert("eab_kid".to_string(), "Key ID is required".to_string());
        }
        let eab_hmac_key = match general_purpose::URL_SAFE_NO_PAD.decode(eab_hmac_key.as_bytes()) {
            Ok(key) => key,
            Err(_) => {
                errors.insert("eab_hmac_key".to_string(), "Invalid HMAC Key".to_string());
                Default::default()
            }
        };
        Some(ExternalAccountBinding {
            key_id: eab_kid.to_string(),
            hmac_key: eab_hmac_key,
        })
    } else {
        None
    };

    if name.trim().is_empty() {
        errors.insert("name".to_string(), "Name is required".to_string());
    }

    if Url::parse(server_url).is_err() {
        errors.insert("server_url".to_string(), "Invalid URL".to_string());
    }

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
        server_url: server_url.to_string(),
        contacts: vec![format!("mailto:{}", email)],
        eab,
        acme: Acme {
            config: AcmeConfig {
                provider: name.trim().to_string(),
                renewal_days: renewal,
            },
            identifiers: vec![domain_name],
            challenge_type: "http-01".to_string(),
        },
    })
}
