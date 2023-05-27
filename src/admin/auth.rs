use super::{with_state, AppState};
use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use utoipa::ToSchema;
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

const MINIMUM_SESSION_EXPIRY: Duration = Duration::from_secs(60 * 5); // 5 minutes

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
            .and(warp::header::optional("authorization"))
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
        Ok(warp::reply::json(&LoginResult {
            token: data.sessions.new_token(),
        }))
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
        ("authorization"=[])
    )
)]
pub async fn logout(state: AppState, header: Option<String>) -> Result<impl Reply, Rejection> {
    if let Some(token) = get_auth_token(&header) {
        state.data.lock().await.sessions.remove(token);
    }
    Ok(warp::reply::reply())
}

pub fn get_auth_token(header: &Option<String>) -> Option<&str> {
    if let Some(header) = header {
        let parts: Vec<&str> = header.split(' ').collect();
        if let [bearer, token] = &parts[..] {
            if bearer.to_lowercase() == "bearer" {
                return Some(*token);
            }
        }
    }
    None
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin")]
    pub username: String,
    #[schema(example = "passw0rd")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResult {
    #[schema(example = "nidhmyh9c7txiyqe53ttsxyq")]
    pub token: String,
}

#[derive(Default)]
pub struct SessionStore {
    tokens: HashMap<String, Instant>,
}

impl SessionStore {
    pub fn new_token(&mut self) -> String {
        let token = cuid2::cuid();
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
