use crate::command::ServerCommand;
use crate::server::rpc::{ErasedRpcMethod, RpcCallback, RpcMethod, RpcWrapper};
use auth::SessionStore;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, post, put};
use axum::{middleware, Json};
use axum::{
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::get,
    Router,
};
use futures::{Stream, TryStreamExt};
use logs::LogReader;
use std::any::Any;
use std::collections::HashMap;
use std::{
    net::SocketAddr,
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll},
};
use taxy_api::app::{AppConfig, AppInfo};
use taxy_api::error::{Error, ErrorMessage};
use taxy_api::event::ServerEvent;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::{GovernorError, GovernorLayer};
use tracing::{trace, warn};

mod acme;
mod app_info;
mod auth;
mod certs;
mod config;
mod logs;
mod ports;
mod proxies;
mod static_file;

pub async fn start_admin(
    app_info: AppInfo,
    addr: SocketAddr,
    command: mpsc::Sender<ServerCommand>,
    mut callback: mpsc::Receiver<RpcCallback>,
    event: broadcast::Sender<ServerEvent>,
) -> anyhow::Result<()> {
    let data = Data::new(app_info).await?;
    let data = Arc::new(Mutex::new(data));
    let app_state = AppState {
        sender: command,
        event_listener_counter: Arc::new(AtomicUsize::new(0)),
        data: data.clone(),
    };

    let data_clone = data.clone();
    tokio::spawn(async move {
        while let Some(cb) = callback.recv().await {
            let mut data = data_clone.lock().await;
            if let Some(tx) = data.rpc_callbacks.remove(&cb.id) {
                let _ = tx.send(cb.result);
            }
        }
    });

    let mut event_recv = event.subscribe();
    tokio::spawn(async move {
        loop {
            match event_recv.recv().await {
                Ok(ServerEvent::AppConfigUpdated { config }) => {
                    data.lock().await.config = config;
                }
                Ok(ServerEvent::Shutdown) => break,
                Err(RecvError::Lagged(n)) => {
                    warn!("event stream lagged: {}", n);
                }
                _ => (),
            }
        }
    });

    let event_stream = EventStream {
        send: event.clone(),
        recv: event.subscribe(),
    };

    let mut event_recv = event.subscribe();

    let counter = app_state.event_listener_counter.clone();
    let sender = app_state.sender.clone();

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(4)
            .burst_size(2)
            .error_handler(|error| match error {
                GovernorError::TooManyRequests { .. } => {
                    AppError::Taxy(Error::TooManyLoginAttempts).into_response()
                }
                _ => AppError::Anyhow(anyhow::anyhow!(error)).into_response(),
            })
            .finish()
            .unwrap(),
    );

    let event_routes = Router::new().route(
        "/",
        get(move || async move {
            let event_stream = event_stream.clone();
            let stream =
                StreamWrapper::new(BroadcastStream::new(event_stream.recv), counter, sender);
            Sse::new(stream).keep_alive(KeepAlive::default())
        }),
    );

    let auth_routes = Router::new()
        .route(
            "/login",
            post(auth::login).layer(GovernorLayer {
                config: governor_conf,
            }),
        )
        .route("/logout", get(auth::logout));

    let config_routes = Router::new()
        .route("/", get(config::get))
        .route("/", put(config::put));

    let ports_routes = Router::new()
        .route("/", get(ports::list))
        .route("/", post(ports::add))
        .route("/{id}", get(ports::get))
        .route("/{id}/status", get(ports::status))
        .route("/{id}", put(ports::put))
        .route("/{id}", delete(ports::delete))
        .route("/{id}/reset", get(ports::reset));

    let proxies_routes = Router::new()
        .route("/", get(proxies::list))
        .route("/", post(proxies::add))
        .route("/{id}", get(proxies::get))
        .route("/{id}/status", get(proxies::status))
        .route("/{id}", put(proxies::put))
        .route("/{id}", delete(proxies::delete));

    let certs_routes = Router::new()
        .route("/", get(certs::list))
        .route("/self_sign", post(certs::self_sign))
        .route("/upload", post(certs::upload))
        .route("/{id}/download", get(certs::download))
        .route("/{id}", get(certs::get))
        .route("/{id}", delete(certs::delete));

    let acme_routes = Router::new()
        .route("/", get(acme::list))
        .route("/{id}", get(acme::get))
        .route("/{id}", put(acme::put))
        .route("/", post(acme::add))
        .route("/{id}", delete(acme::delete));

    let logs_routes = Router::new().route("/{id}", get(logs::get));

    let app_info_routes = Router::new().route("/", get(app_info::get));

    let api_routes = Router::new()
        .nest("/events", event_routes)
        .nest("/config", config_routes)
        .nest("/ports", ports_routes)
        .nest("/proxies", proxies_routes)
        .nest("/certs", certs_routes)
        .nest("/acme", acme_routes)
        .nest("/logs", logs_routes)
        .nest("/app_info", app_info_routes)
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth::verify,
        ));

    let app = Router::new()
        .nest("/api", auth_routes)
        .nest("/api", api_routes)
        .fallback(static_file::fallback)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async move {
        loop {
            let event = event_recv.recv().await;
            trace!("received server event: {:?}", event);
            match event {
                Ok(ServerEvent::Shutdown) => {
                    break;
                }
                Err(RecvError::Lagged(n)) => {
                    warn!("event stream lagged: {}", n);
                }
                _ => {}
            }
        }
    })
    .await?;
    Ok(())
}

struct StreamWrapper {
    inner: BroadcastStream<ServerEvent>,
    counter: Arc<AtomicUsize>,
    sender: Sender<ServerCommand>,
}

impl StreamWrapper {
    fn new(
        stream: BroadcastStream<ServerEvent>,
        counter: Arc<AtomicUsize>,
        sender: Sender<ServerCommand>,
    ) -> Self {
        if counter.fetch_add(1, Ordering::Relaxed) == 0 {
            let _ = sender.try_send(ServerCommand::SetBroadcastEvents { enabled: true });
        }
        Self {
            inner: stream,
            counter,
            sender,
        }
    }
}

impl Stream for StreamWrapper {
    type Item = Result<Event, BroadcastStreamRecvError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.try_poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(event))) => {
                Poll::Ready(Some(Ok(Event::default().json_data(event).unwrap())))
            }
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(err))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for StreamWrapper {
    fn drop(&mut self) {
        if self.counter.fetch_sub(1, Ordering::Release) == 1 {
            let _ = self
                .sender
                .try_send(ServerCommand::SetBroadcastEvents { enabled: false });
        }
    }
}

struct EventStream {
    send: broadcast::Sender<ServerEvent>,
    recv: broadcast::Receiver<ServerEvent>,
}

impl Clone for EventStream {
    fn clone(&self) -> Self {
        Self {
            send: self.send.clone(),
            recv: self.send.subscribe(),
        }
    }
}

pub enum AppError {
    NotFound,
    Anyhow(anyhow::Error),
    Taxy(taxy_api::error::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Anyhow(err)
    }
}

impl From<taxy_api::error::Error> for AppError {
    fn from(err: taxy_api::error::Error) -> Self {
        AppError::Taxy(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code;
        let message;
        let mut error = None;
        match self {
            AppError::NotFound => {
                message = "NOT_FOUND".to_string();
                code = StatusCode::NOT_FOUND;
            }
            AppError::Anyhow(err) => {
                message = err.to_string();
                code = StatusCode::INTERNAL_SERVER_ERROR;
            }
            AppError::Taxy(err) => {
                message = err.to_string();
                code = StatusCode::from_u16(err.status_code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                error = Some(err);
            }
        }
        (code, Json(ErrorMessage { message, error })).into_response()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub sender: mpsc::Sender<ServerCommand>,
    pub event_listener_counter: Arc<AtomicUsize>,
    pub data: Arc<Mutex<Data>>,
}

impl AppState {
    pub async fn call<T>(&self, method: T) -> Result<Box<T::Output>, Error>
    where
        T: RpcMethod,
    {
        let mut data = self.data.lock().await;
        let id = data.rpc_counter;
        data.rpc_counter += 1;

        let (tx, rx) = oneshot::channel();
        data.rpc_callbacks.insert(id, tx);
        std::mem::drop(data);

        let arg = Box::new(RpcWrapper::new(method)) as Box<dyn ErasedRpcMethod>;
        let _ = self
            .sender
            .send(ServerCommand::CallMethod { id, arg })
            .await;

        match rx.await {
            Ok(v) => match v {
                Ok(value) => value.downcast().map_err(|_| Error::FailedToInvokeRpc),
                Err(err) => Err(err),
            },
            Err(_) => Err(Error::FailedToInvokeRpc),
        }
    }
}

pub type CallbackData = Result<Box<dyn Any + Send + Sync>, Error>;

pub struct Data {
    pub app_info: AppInfo,
    pub config: AppConfig,
    pub sessions: SessionStore,
    pub log: Arc<LogReader>,

    pub rpc_counter: usize,
    pub rpc_callbacks: HashMap<usize, oneshot::Sender<CallbackData>>,
}

impl Data {
    pub async fn new(app_info: AppInfo) -> anyhow::Result<Self> {
        let log = app_info.log_path.join("log.db");
        Ok(Self {
            app_info,
            config: AppConfig::default(),
            sessions: Default::default(),
            log: Arc::new(LogReader::new(&log).await?),
            rpc_counter: 0,
            rpc_callbacks: HashMap::new(),
        })
    }
}
