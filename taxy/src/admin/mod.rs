use self::auth::{SessionKind, SessionStore};
use self::log::LogReader;
use crate::command::ServerCommand;
use crate::server::rpc::ErasedRpcMethod;
use crate::server::rpc::{RpcCallback, RpcMethod, RpcWrapper};
use futures::{Stream, TryStreamExt};
use hyper::StatusCode;
use std::any::Any;
use std::collections::HashMap;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use std::time::Instant;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use taxy_api::app::{AppConfig, AppInfo};
use taxy_api::error::{Error, ErrorMessage};
use taxy_api::event::ServerEvent;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{error, info, trace, warn};
use utoipa::OpenApi;
use warp::filters::body::BodyDeserializeError;
use warp::{sse::Event, Filter, Rejection, Reply};

mod acme;
mod app_info;
mod auth;
mod certs;
mod config;
mod log;
mod ports;
mod proxies;
mod static_file;
mod swagger;

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

    let static_file = warp::get()
        .and(warp::path::full())
        .and(warp::header::optional::<String>("If-None-Match"))
        .and_then(static_file::get);

    let event_stream = EventStream {
        send: event.clone(),
        recv: event.subscribe(),
    };

    let mut event_recv = event.subscribe();

    let counter = app_state.event_listener_counter.clone();
    let sender = app_state.sender.clone();
    let api_events = warp::path("events")
        .and(with_state(app_state.clone()))
        .and(warp::path::end())
        .and(warp::get())
        .map(move |_| {
            let event_stream = event_stream.clone();
            let counter = counter.clone();
            let sender = sender.clone();
            warp::sse::reply(warp::sse::keep_alive().stream(StreamWrapper::new(
                BroadcastStream::new(event_stream.recv),
                counter,
                sender,
            )))
        });

    let options = warp::options().map(warp::reply);
    let not_found = warp::get().and_then(handle_not_found);

    let api_doc = warp::path("api-doc.json")
        .and(warp::get())
        .map(|| warp::reply::json(&swagger::ApiDoc::openapi()));

    let api = warp::path("api").and(
        options
            .or(app_info::api(app_state.clone()))
            .or(config::api(app_state.clone()))
            .or(ports::api(app_state.clone()))
            .or(proxies::api(app_state.clone()))
            .or(certs::api(app_state.clone()))
            .or(acme::api(app_state.clone()))
            .or(auth::api(app_state.clone()))
            .or(log::api(app_state))
            .or(api_events)
            .or(api_doc)
            .or(not_found),
    );

    let (_, server) = warp::serve(
        api.or(swagger::swagger_ui())
            .or(static_file)
            .recover(handle_rejection),
    )
    .try_bind_with_graceful_shutdown(addr, async move {
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
    })?;

    info!("webui server started on {}", addr);
    server.await;
    Ok(())
}

async fn handle_not_found() -> Result<&'static [u8], Rejection> {
    Err(warp::reject::not_found())
}

#[derive(Clone)]
pub struct AppState {
    sender: mpsc::Sender<ServerCommand>,
    event_listener_counter: Arc<AtomicUsize>,
    data: Arc<Mutex<Data>>,
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
                Poll::Ready(Some(Ok(Event::default().json_data(&event).unwrap())))
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

type CallbackData = Result<Box<dyn Any + Send + Sync>, Error>;

struct Data {
    app_info: AppInfo,
    config: AppConfig,
    sessions: SessionStore,
    log: Arc<LogReader>,

    rpc_counter: usize,
    rpc_callbacks: HashMap<usize, oneshot::Sender<CallbackData>>,

    rate_limiter: HashMap<IpAddr, (usize, Instant)>,
}

impl AppState {
    async fn call<T>(&self, method: T) -> Result<Box<T::Output>, Error>
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

impl Data {
    async fn new(app_info: AppInfo) -> anyhow::Result<Self> {
        let log = app_info.log_path.join("log.db");
        Ok(Self {
            app_info,
            config: AppConfig::default(),
            sessions: Default::default(),
            log: Arc::new(LogReader::new(&log).await?),
            rpc_counter: 0,
            rpc_callbacks: HashMap::new(),
            rate_limiter: HashMap::new(),
        })
    }
}

fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Rejection> + Clone {
    let data = state.data.clone();
    warp::any()
        .and(
            warp::cookie::optional("token").and_then(move |token: Option<String>| {
                let data = data.clone();
                async move {
                    if let Some(token) = token {
                        let mut data = data.lock().await;
                        let expiry = data.config.admin.session_expiry;
                        if data
                            .sessions
                            .verify(SessionKind::Admin, &token, expiry)
                            .is_some()
                        {
                            return Ok(());
                        }
                    }
                    Err(warp::reject::custom(Error::Unauthorized))
                }
            }),
        )
        .map(move |_| state.clone())
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

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message;
    let mut error = None;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_string();
    } else if err.find::<BodyDeserializeError>().is_some() {
        message = "BAD_REQUEST".to_string();
        code = StatusCode::BAD_REQUEST;
    } else if let Some(err) = err.find::<Error>() {
        message = err.to_string();
        code = StatusCode::from_u16(err.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        error = Some(err.clone());
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_string();
    } else {
        error!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION".to_string();
    }

    let json = warp::reply::json(&ErrorMessage { message, error });
    Ok(warp::reply::with_status(json, code))
}
