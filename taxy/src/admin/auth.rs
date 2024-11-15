use super::{AppError, AppState};
use crate::server::rpc::auth::VerifyAccount;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use rand::distributions::{Alphanumeric, DistString};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use taxy_api::{
    auth::{LoginMethod, LoginRequest, LoginResponse},
    error::Error,
};

const MINIMUM_SESSION_EXPIRY: Duration = Duration::from_secs(60 * 5); // 5 minutes
const SESSION_TOKEN_LENGTH: usize = 32;

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let username = request.username.clone();
    let token = jar.get("token").map(|c| c.value().to_string());

    if let LoginMethod::Totp { .. } = &request.method {
        let mut data = state.data.lock().await;
        let expiry = data.config.admin.session_expiry;
        let ok = data
            .sessions
            .verify(SessionKind::Login, &token.unwrap_or_default(), expiry)
            .map(|session| session.username == username)
            .unwrap_or_default();
        if !ok {
            return Err(Error::InvalidLoginCredentials.into());
        }
    }

    let insecure = request.insecure;
    let result = state.call(VerifyAccount { request }).await?;

    let session = match *result {
        LoginResponse::Success => SessionKind::Admin,
        _ => SessionKind::Login,
    };

    let token = state
        .data
        .lock()
        .await
        .sessions
        .new_token(session, &username);

    let cookie = Cookie::build(("token", token))
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(!insecure)
        .build();

    Ok((jar.add(cookie), Json(result)))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    if let Some(token) = jar.get("token") {
        state.data.lock().await.sessions.remove(token.value());
    }
    jar.remove("token")
}

pub async fn verify(
    State(state): State<AppState>,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Response {
    if let Some(token) = jar.get("token") {
        let mut data = state.data.lock().await;
        let expiry = data.config.admin.session_expiry;
        if data
            .sessions
            .verify(SessionKind::Admin, token.value(), expiry)
            .is_some()
        {
            std::mem::drop(data);
            let response = next.run(request).await;
            return response;
        }
    }
    AppError::Taxy(Error::Unauthorized).into_response()
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
