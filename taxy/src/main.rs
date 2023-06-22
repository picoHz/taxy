#![forbid(unsafe_code)]

use crate::args::Command;
use crate::config::new_appinfo;
use crate::config::storage::ConfigStorage;
use crate::log::DatabaseLayer;
use crate::server::Server;
use args::StartArgs;
use clap::Parser;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::filter::{self, FilterExt};
use tracing_subscriber::prelude::*;

mod admin;
mod args;
mod auth;
mod certs;
mod command;
mod config;
mod log;
mod proxy;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = args::Cli::parse();

    match args.command {
        Command::Start(args) => start(args).await?,
        Command::AddUser(args) => add_user(args).await?,
    }

    Ok(())
}

async fn start(args: StartArgs) -> anyhow::Result<()> {
    let log_dir = get_log_dir(args.log_dir)?;
    fs::create_dir_all(&log_dir)?;

    let log = args.log.as_ref().map(|path| log_dir.join(path));
    if let Some(path) = log.as_ref().and_then(|path| path.parent()) {
        fs::create_dir_all(path)?;
    }

    let access_log = args.access_log.as_ref().map(|path| log_dir.join(path));
    if let Some(path) = access_log.as_ref().and_then(|path| path.parent()) {
        fs::create_dir_all(path)?;
    }

    let (log, _guard) = log::create_layer(log, "taxy.log", args.log_level, args.log_format);
    let (access_log, _guard) = log::create_layer(
        access_log,
        "access.log",
        args.access_log_level,
        args.log_format,
    );
    let db = DatabaseLayer::new(&log_dir.join("log.db"), args.log_level).await?;

    let access_log_filter =
        filter::filter_fn(|metadata| metadata.target().starts_with("taxy::access_log"));
    tracing_subscriber::registry()
        .with(log.with_filter(access_log_filter.clone().not()))
        .with(access_log.with_filter(access_log_filter))
        .with(db)
        .init();

    let config_dir = get_config_dir(args.config_dir)?;
    fs::create_dir_all(&config_dir)?;

    let config = ConfigStorage::new(&config_dir);
    let app_info = new_appinfo(&config_dir, &log_dir);

    let (server, channels) = Server::new(config);
    let server_task = tokio::spawn(server.start());
    let event_send = channels.event.clone();

    let webui_enabled = !args.no_webui;
    tokio::select! {
        r = admin::start_admin(app_info, args.webui, channels.command, channels.callback, channels.event), if webui_enabled => {
            if let Err(err) = r {
                error!("admin error: {}", err);
            }
        }
        _ =  tokio::signal::ctrl_c() => {
            info!("received ctrl-c signal");
        }
    };

    let _ = event_send.send(taxy_api::event::ServerEvent::Shutdown);
    server_task.await??;

    Ok(())
}

async fn add_user(args: args::AddUserArgs) -> anyhow::Result<()> {
    let config_dir = get_config_dir(args.config_dir)?;
    let password = if let Some(password) = args.password {
        password
    } else {
        rpassword::prompt_password("password?: ")?
    };
    auth::add_account(&config_dir, &args.name, &password).await?;
    Ok(())
}

fn get_config_dir(dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(dir) = dir {
        Ok(dir)
    } else {
        let dir = ProjectDirs::from("", "", "taxy").ok_or_else(|| {
            anyhow::anyhow!("failed to get project directories, try setting --config-dir")
        })?;
        Ok(dir.config_dir().to_owned())
    }
}

fn get_log_dir(dir: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(dir) = dir {
        Ok(dir)
    } else {
        let dir = ProjectDirs::from("", "", "taxy").ok_or_else(|| {
            anyhow::anyhow!("failed to get project directories, try setting --log-dir")
        })?;
        Ok(dir.data_dir().join("logs"))
    }
}
