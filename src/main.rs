#![forbid(unsafe_code)]

use crate::config::storage::ConfigStorage;
use clap::Parser;
use directories::ProjectDirs;
use std::fs;
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info};
use tracing_subscriber::filter::{self, FilterExt};
use tracing_subscriber::prelude::*;

mod admin;
mod args;
mod command;
mod config;
mod error;
mod event;
mod keyring;
mod log;
mod proxy;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = args::Args::parse();

    if let Some(path) = args.log.as_ref().and_then(|path| path.parent()) {
        fs::create_dir_all(path)?;
    }
    if let Some(path) = args.access_log.as_ref().and_then(|path| path.parent()) {
        fs::create_dir_all(path)?;
    }

    let (log, _guard) = log::create_layer(args.log, "taxy.log", args.log_level, args.log_format);
    let (access_log, _guard) = log::create_layer(
        args.access_log,
        "access.log",
        args.access_log_level,
        args.log_format,
    );

    let access_log_filter =
        filter::filter_fn(|metadata| metadata.target().starts_with("taxy::access_log"));
    tracing_subscriber::registry()
        .with(log.with_filter(access_log_filter.clone().not()))
        .with(access_log.with_filter(access_log_filter))
        .init();

    let config_dir = if let Some(dir) = args.config_dir {
        dir
    } else {
        let dir = ProjectDirs::from("proxy", "taxy", "taxy").ok_or_else(|| {
            anyhow::anyhow!("failed to get project directories, try setting --config-dir")
        })?;
        dir.config_dir().to_owned()
    };

    fs::create_dir_all(&config_dir)?;
    let config = ConfigStorage::new(&config_dir);

    let (event_send, _) = broadcast::channel(16);
    let (command_send, command_recv) = mpsc::channel(1);
    let server_task = tokio::spawn(server::start_server(
        config,
        command_recv,
        event_send.clone(),
    ));

    let webui_enabled = !args.no_webui;
    tokio::select! {
        r = admin::start_admin(args.webui, command_send, event_send.clone()), if webui_enabled => {
            if let Err(err) = r {
                error!("admin error: {}", err);
            }
        }
        _ =  tokio::signal::ctrl_c() => {
            info!("received ctrl-c signal");
        }
    };

    let _ = event_send.send(event::ServerEvent::Shutdown);
    server_task.await??;

    Ok(())
}
