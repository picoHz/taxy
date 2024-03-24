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
    auth::{LoginMethod, LoginRequest, LoginResponse},
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
            .and(warp::cookie::optional("token"))
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
pub async fn login(
    state: AppState,
    request: LoginRequest,
    token: Option<String>,
) -> Result<impl Reply, Rejection> {
    let username = request.username.clone();

    if let LoginMethod::Totp { .. } = &request.method {
        let mut data = state.data.lock().await;
        let expiry = data.config.admin.session_expiry;
        let ok = data
            .sessions
            .verify(SessionKind::Login, &token.unwrap_or_default(), expiry)
            .map(|session| session.username == username)
            .unwrap_or_default();
        if !ok {
            return Err(warp::reject::custom(Error::InvalidLoginCredentials));
        }
    }

    let insecure = request.insecure || state.data.lock().await.insecure;
    let result = state.call(VerifyAccount { request }).await;
    match result {
        Err(err) => Err(warp::reject::custom(err)),
        Ok(res) => {
            let session = match *res {
                LoginResponse::Success => SessionKind::Admin,
                _ => SessionKind::Login,
            };
            let secure_cookie = if insecure { "" } else { "Secure" };
            Ok(warp::reply::with_header(
                warp::reply::json(&res),
                "Set-Cookie",
                &format!(
                    "token={}; HttpOnly; SameSite=Strict; {secure_cookie}",
                    state
                        .data
                        .lock()
                        .await
                        .sessions
                        .new_token(session, &username)
                ),
            ))
        }
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

#[derive(Debug, Clone)]
pub struct Session {
    pub kind: SessionKind,
    pub username: String,
    pub started_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Login,
    Admin,
}

#[derive(Default)]
pub struct SessionStore {
    tokens: HashMap<String, Session>,
}

impl SessionStore {
    pub fn new_token(&mut self, kind: SessionKind, username: &str) -> String {
        let token = Alphanumeric.sample_string(&mut rand::thread_rng(), SESSION_TOKEN_LENGTH);
        self.tokens.insert(
            token.clone(),
            Session {
                kind,
                username: username.to_string(),
                started_at: Instant::now(),
            },
        );
        token
    }

    pub fn verify(&mut self, kind: SessionKind, token: &str, expiry: Duration) -> Option<&Session> {
        let expiry = expiry.max(MINIMUM_SESSION_EXPIRY);
        self.tokens = self
            .tokens
            .drain()
            .filter(|(_, t)| t.started_at.elapsed() < expiry)
            .collect();
        self.tokens
            .get(token)
            .filter(|session| session.kind == kind)
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
