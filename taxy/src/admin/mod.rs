use crate::command::ServerCommand;
use crate::server::rpc::ErasedRpcMethod;
use crate::server::rpc::{RpcCallback, RpcMethod, RpcWrapper};
use hyper::StatusCode;
use serde_derive::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use taxy_api::app::{AppConfig, AppInfo};
use taxy_api::error::Error;
use taxy_api::event::ServerEvent;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::{error, info, trace, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::Config;
use warp::filters::body::BodyDeserializeError;
use warp::{sse::Event, Filter, Rejection, Reply};

use self::auth::SessionStore;
use self::log::LogReader;

mod acme;
mod app_info;
mod auth;
mod config;
mod log;
mod ports;
mod server_certs;
mod sites;
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
                Ok(ServerEvent::AppConfigUpdated { config, .. }) => {
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
        .and_then(static_file::get);

    let event_stream = EventStream {
        send: event.clone(),
        recv: event.subscribe(),
    };

    let mut event_recv = event.subscribe();

    let api_events = warp::path("events")
        .and(with_state(app_state.clone()))
        .and(warp::path::end())
        .and(warp::get())
        .map(move |_| {
            let event_stream = event_stream.clone();
            warp::sse::reply(
                warp::sse::keep_alive().stream(
                    BroadcastStream::new(event_stream.recv)
                        .map_while(|e| match e {
                            Ok(ServerEvent::Shutdown) => None,
                            Ok(event) => Some(event),
                            _ => None,
                        })
                        .map(|e| Event::default().json_data(&e)),
                ),
            )
        });

    let options = warp::options().map(warp::reply);
    let not_found = warp::get().and_then(handle_not_found);

    let api_doc = warp::path("api-doc.json")
        .and(warp::get())
        .map(|| warp::reply::json(&swagger::ApiDoc::openapi()));

    let api_config = Arc::new(Config::from("/api/api-doc.json"));

    let swagger_ui = warp::path("swagger-ui")
        .and(warp::get())
        .and(warp::path::full())
        .and(warp::path::tail())
        .and(warp::any().map(move || api_config.clone()))
        .and_then(swagger::serve_swagger);

    let api = warp::path("api").and(
        options
            .or(app_info::api(app_state.clone()))
            .or(config::api(app_state.clone()))
            .or(ports::api(app_state.clone()))
            .or(sites::api(app_state.clone()))
            .or(server_certs::api(app_state.clone()))
            .or(acme::api(app_state.clone()))
            .or(auth::api(app_state.clone()))
            .or(log::api(app_state))
            .or(api_events)
            .or(api_doc)
            .or(not_found),
    );

    #[cfg(debug_assertions)]
    let api = api
        .with(warp::reply::with::header(
            "Access-Control-Allow-Headers",
            "content-type, authorization",
        ))
        .with(warp::reply::with::header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE",
        ))
        .with(warp::reply::with::header(
            "Access-Control-Allow-Origin",
            "http://localhost:3000",
        ));

    let (_, server) = warp::serve(api.or(swagger_ui).or(static_file).recover(handle_rejection))
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
    data: Arc<Mutex<Data>>,
}

type CallbackData = Result<Box<dyn Any + Send + Sync>, Error>;

struct Data {
    app_info: AppInfo,
    config: AppConfig,
    sessions: SessionStore,
    log: Arc<LogReader>,

    rpc_counter: usize,
    rpc_callbacks: HashMap<usize, oneshot::Sender<CallbackData>>,
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
                Ok(value) => value.downcast().map_err(|_| Error::RpcError),
                Err(err) => Err(err),
            },
            Err(_) => Err(Error::RpcError),
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
        })
    }
}

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

fn with_state(state: AppState) -> impl Filter<Extract = (AppState,), Error = Rejection> + Clone {
    let data = state.data.clone();
    warp::any()
        .and(
            warp::header::optional("authorization")
                .and(warp::query::<TokenQuery>())
                .and_then(move |header: Option<String>, query: TokenQuery| {
                    let data = data.clone();
                    async move {
                        if let Some(token) =
                            auth::get_auth_token(&header).or(query.token.as_deref())
                        {
                            let mut data = data.lock().await;
                            let expiry = data.config.admin_session_expiry;
                            if data.sessions.verify(token, expiry) {
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

#[derive(Serialize)]
struct ErrorMessage {
    message: String,
    error: Option<Error>,
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

    let reply = warp::reply::with_status(json, code);

    #[cfg(debug_assertions)]
    let reply = warp::reply::with_header(
        reply,
        "Access-Control-Allow-Origin",
        "http://localhost:3000",
    );

    Ok(reply)
}
