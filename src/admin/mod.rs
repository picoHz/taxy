use crate::config::{AppConfig, AppInfo};
use crate::keyring::KeyringInfo;
use crate::server::rpc::{RpcCallback, RpcMethod};
use crate::{command::ServerCommand, error::Error, event::ServerEvent};
use hyper::StatusCode;
use serde_derive::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::{error, info, trace, warn};
use utoipa::OpenApi;
use utoipa_swagger_ui::Config;
use warp::filters::body::BodyDeserializeError;
use warp::{sse::Event, Filter, Rejection, Reply};

use self::auth::SessionStore;
use self::log::LogReader;

mod app_info;
mod auth;
mod config;
mod keyring;
mod log;
mod ports;
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
                Ok(ServerEvent::KeyringUpdated { items }) => {
                    data.lock().await.keyring_items = items;
                }
                Ok(ServerEvent::Shutdown) => break,
                Err(RecvError::Lagged(n)) => {
                    warn!("event stream lagged: {}", n);
                }
                _ => (),
            }
        }
    });

    let api_config_get = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(config::get));

    let api_config_put = warp::put().and(warp::path::end()).and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and_then(config::put),
    );

    let api_ports_list = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(ports::list));

    let api_ports_status = warp::get()
        .and(with_state(app_state.clone()))
        .and(warp::path::param())
        .and(warp::path("status"))
        .and(warp::path::end())
        .and_then(ports::status);

    let api_ports_delete = warp::delete().and(
        with_state(app_state.clone())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(ports::delete),
    );

    let api_ports_put = warp::put().and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(ports::put),
    );

    let api_ports_post = warp::post().and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(ports::post),
    );

    let api_ports_log = warp::get().and(
        with_state(app_state.clone())
            .and(warp::path::param())
            .and(warp::path("log"))
            .and(warp::query())
            .and(warp::path::end())
            .and_then(ports::log),
    );

    let api_keyring_list = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(keyring::list));

    let api_keyring_self_signed = warp::post().and(warp::path("self_signed")).and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(keyring::self_signed),
    );

    let api_keyring_upload = warp::post().and(warp::path("upload")).and(
        with_state(app_state.clone())
            .and(warp::multipart::form())
            .and(warp::path::end())
            .and_then(keyring::upload),
    );

    let api_keyring_acme = warp::post().and(warp::path("acme")).and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(keyring::acme),
    );

    let api_keyring_delete = warp::delete().and(
        with_state(app_state.clone())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(keyring::delete),
    );

    let api_keyring_log = warp::get().and(
        with_state(app_state.clone())
            .and(warp::path::param())
            .and(warp::path("log"))
            .and(warp::query())
            .and(warp::path::end())
            .and_then(keyring::log),
    );

    let app_state_clone = app_state.clone();
    let api_auth_login = warp::post()
        .and(warp::path("login"))
        .map(move || app_state_clone.clone())
        .and(warp::body::json())
        .and(warp::path::end())
        .and_then(auth::login);

    let api_auth_logout = warp::get().and(warp::path("logout")).and(
        with_state(app_state.clone())
            .and(warp::header::optional("authorization"))
            .and(warp::path::end())
            .and_then(auth::logout),
    );

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

    let app_info = warp::path("app_info").and(warp::get()).and(
        with_state(app_state.clone())
            .and(warp::path::end())
            .and_then(app_info::get),
    );

    let config = warp::path("config").and(api_config_get.or(api_config_put));

    let port = warp::path("ports").and(
        api_ports_delete
            .or(api_ports_log)
            .or(api_ports_put)
            .or(api_ports_status)
            .or(api_ports_list)
            .or(api_ports_post),
    );

    let keyring = warp::path("keyring").and(
        api_keyring_delete
            .or(api_keyring_log)
            .or(api_keyring_self_signed)
            .or(api_keyring_upload)
            .or(api_keyring_acme)
            .or(api_keyring_list),
    );

    let auth = api_auth_login.or(api_auth_logout);

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
            .or(app_info)
            .or(config)
            .or(port)
            .or(keyring)
            .or(api_events)
            .or(auth)
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
    keyring_items: Vec<KeyringInfo>,
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

        let arg = Box::new(method) as Box<dyn Any + Send + Sync>;
        let _ = self
            .sender
            .send(ServerCommand::CallMethod {
                id,
                method: T::NAME.to_string(),
                arg,
            })
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
            keyring_items: Vec::new(),
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
        code = err.status_code();
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
