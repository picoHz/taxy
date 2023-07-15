use super::{with_state, AppState};
use crate::server::rpc::auth::VerifyAccount;
use rand::distributions::{Alphanumeric, DistString};
use serde_json::Value;
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::{Duration, Instant},
};
use taxy_api::{
    auth::{LoginRequest, LoginResponse},
    error::Error,
};
use warp::{filters::BoxedFilter, Filter, Rejection, Reply};

const MINIMUM_SESSION_EXPIRY: Duration = Duration::from_secs(60 * 5); // 5 minutes
const SESSION_TOKEN_LENGTH: usize = 32;

pub fn api(app_state: AppState) -> BoxedFilter<(impl Reply,)> {
    let api_login = warp::post().and(
        rate_limit(app_state.clone())
            .and(warp::path("login"))
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(login),
    );

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
        (status = 200, body = LoginResponse),
        (status = 400),
    )
)]
pub async fn login(state: AppState, req: LoginRequest) -> Result<impl Reply, Rejection> {
    let result = state
        .call(VerifyAccount {
            username: req.username,
            password: req.password,
        })
        .await;
    if let Err(err) = result {
        Err(warp::reject::custom(err))
    } else {
        Ok(warp::reply::with_header(
            warp::reply::json(&LoginResponse { success: true }),
            "Set-Cookie",
            &format!(
                "token={}; HttpOnly; SameProxy=Strict; Secure",
                state.data.lock().await.sessions.new_token()
            ),
        ))
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

fn rate_limit(state: AppState) -> impl Filter<Extract = (AppState,), Error = Rejection> + Clone {
    let data = state.data.clone();
    warp::any()
        .and(
            warp::addr::remote().and_then(move |addr: Option<SocketAddr>| {
                let data = data.clone();
                async move {
                    let mut data = data.lock().await;
                    let config = data.config.admin;
                    let rate_limiter = &mut data.rate_limiter;
                    *rate_limiter = rate_limiter
                        .drain()
                        .filter(|(_, (_, t))| t.elapsed() < config.login_attempts_reset)
                        .collect();
                    let addr = addr.map(|addr| addr.ip()).unwrap_or([0, 0, 0, 0].into());
                    let entry = rate_limiter
                        .entry(addr)
                        .or_insert_with(|| (0, Instant::now()));
                    entry.0 += 1;
                    if entry.0 > config.max_login_attempts as _ {
                        Err(warp::reject::custom(Error::TooManyLoginAttempts))
                    } else {
                        Ok(())
                    }
                }
            }),
        )
        .map(move |_| state.clone())
}
