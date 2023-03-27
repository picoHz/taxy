use crate::proxy::PortStatus;
use crate::{command::ServerCommand, config::port::PortEntry, error::Error, event::ServerEvent};
use hyper::StatusCode;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tracing::{error, info, trace, warn};
use warp::filters::body::BodyDeserializeError;
use warp::{sse::Event, Filter, Rejection, Reply};

mod app_info;
mod port;
mod static_file;

pub async fn start_admin(
    addr: SocketAddr,
    command: mpsc::Sender<ServerCommand>,
    event: broadcast::Sender<ServerEvent>,
) -> anyhow::Result<()> {
    let data = Arc::new(Mutex::new(Data::default()));
    let app_state = AppState {
        sender: command,
        data: data.clone(),
    };

    let mut event_recv = event.subscribe();
    tokio::spawn(async move {
        loop {
            match event_recv.recv().await {
                Ok(ServerEvent::PortTableUpdated { entries: ports, .. }) => {
                    data.lock().await.entries = ports;
                }
                Ok(ServerEvent::PortStatusUpdated { name, status }) => {
                    data.lock().await.status.insert(name, status);
                }
                Ok(ServerEvent::Shutdown) => break,
                Err(RecvError::Lagged(n)) => {
                    warn!("event stream lagged: {}", n);
                }
                _ => (),
            }
        }
    });

    let api_ports_list = warp::get()
        .and(warp::path::end())
        .and(with_state(app_state.clone()).and_then(port::list));

    let api_ports_status = warp::get()
        .and(with_state(app_state.clone()))
        .and(warp::path::param())
        .and(warp::path("status"))
        .and(warp::path::end())
        .and_then(port::status);

    let api_ports_delete = warp::delete().and(
        with_state(app_state.clone())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(port::delete),
    );

    let api_ports_put = warp::put().and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::param())
            .and(warp::path::end())
            .and_then(port::put),
    );

    let api_ports_post = warp::post().and(
        with_state(app_state.clone())
            .and(warp::body::json())
            .and(warp::path::end())
            .and_then(port::post),
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
        .and(warp::path::end())
        .and(warp::get())
        .map(move || {
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

    let app_info = warp::path("app_info")
        .and(warp::get())
        .and(warp::path::end())
        .and_then(app_info::get);

    let port = warp::path("ports").and(
        api_ports_delete
            .or(api_ports_put)
            .or(api_ports_status)
            .or(api_ports_list)
            .or(api_ports_post),
    );
    let options = warp::options().map(warp::reply);
    let not_found = warp::get().and_then(handle_not_found);

    let api = warp::path("api")
        .and(options.or(app_info).or(port).or(api_events).or(not_found))
        .with(warp::reply::with::header(
            "Access-Control-Allow-Headers",
            "content-type",
        ))
        .with(warp::reply::with::header(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, DELETE",
        ))
        .with(warp::reply::with::header(
            "Access-Control-Allow-Origin",
            "http://localhost:3000",
        ));

    let (_, server) = warp::serve(api.or(static_file).recover(handle_rejection))
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

#[derive(Default)]
struct Data {
    entries: Vec<PortEntry>,
    status: HashMap<String, PortStatus>,
}

fn with_state(
    state: AppState,
) -> impl Filter<Extract = (AppState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
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
        code = StatusCode::BAD_REQUEST;
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
    let reply = warp::reply::with_header(
        reply,
        "Access-Control-Allow-Origin",
        "http://localhost:3000",
    );
    Ok(reply)
}
