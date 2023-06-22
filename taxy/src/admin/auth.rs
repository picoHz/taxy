use super::{with_state, AppState};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::Value;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use taxy_api::{auth::LoginRequest, error::Error};
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

const MINIMUM_SESSION_EXPIRY: Duration = Duration::from_secs(60 * 5); // 5 minutes
const SESSION_TOKEN_LENGTH: usize = 32;

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    let app_state_clone = app_state.clone();
    let api_login = warp::post()
        .and(warp::path("login"))
        .map(move || app_state_clone.clone())
        .and(warp::body::json())
        .and(warp::path::end())
        .and_then(login);

    let api_logout = warp::get().and(warp::path("logout")).and(
        with_state(app_state)
            .and(warp::cookie::cookie("token"))
            .and(warp::path::end())
            .and_then(logout),
    );

    api_login.or(api_logout).boxed()
}

/// Login.
#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    responses(
        (status = 200),
        (status = 400),
    )
)]
pub async fn login(state: AppState, req: LoginRequest) -> Result<impl Reply, Rejection> {
    let mut data = state.data.lock().await;
    if crate::auth::verify_account(&data.app_info.config_path, &req.username, &req.password).await {
        Ok(warp::reply::with_header(
            warp::reply::json(&Value::Object(Default::default())),
            "Set-Cookie",
            &format!(
                "token={}; HttpOnly; SameSite=Strict; Secure",
                data.sessions.new_token()
            ),
        ))
    } else {
        Err(warp::reject::custom(Error::InvalidLoginCredentials))
    }
}

/// Logout.
#[utoipa::path(
    get,
    path = "/api/logout",
    responses(
        (status = 200),
        (status = 401),
    ),
    security(
        ("cookie"=[])
    )
)]
pub async fn logout(state: AppState, token: String) -> Result<impl Reply, Rejection> {
    let mut data = state.data.lock().await;
    data.sessions.remove(&token);
    Ok(warp::reply::with_header(
        warp::reply::json(&Value::Object(Default::default())),
        "Set-Cookie",
        "token=",
    ))
}

#[derive(Default)]
pub struct SessionStore {
    tokens: HashMap<String, Instant>,
}

impl SessionStore {
    pub fn new_token(&mut self) -> String {
        let token = Alphanumeric.sample_string(&mut rand::thread_rng(), SESSION_TOKEN_LENGTH);
        self.tokens.insert(token.clone(), Instant::now());
        token
    }

    pub fn verify(&mut self, token: &str, expiry: Duration) -> bool {
        let expiry = expiry.max(MINIMUM_SESSION_EXPIRY);
        self.tokens = self
            .tokens
            .drain()
            .filter(|(_, t)| t.elapsed() < expiry)
            .collect();
        self.tokens.contains_key(token)
    }

    pub fn remove(&mut self, token: &str) {
        self.tokens.remove(token);
    }
}
