#![forbid(unsafe_code)]

use clap::Parser;
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use taxy::args::Command;
use taxy::args::StartArgs;
use taxy::config::file::FileStorage;
use taxy::config::new_appinfo;
use taxy::config::storage::Storage;
use taxy::log::DatabaseLayer;
use taxy::server::Server;
use tracing::{error, info};
use tracing_subscriber::filter::{self, FilterExt};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = taxy::args::Cli::parse();

    match args.command {
        Command::Start(args) => start(args).await?,
        Command::AddUser(args) => add_user(args).await?,
    }

    Ok(())
}

async fn start(args: StartArgs) -> anyhow::Result<()> {
    let log_dir = get_log_dir(args.log_dir)?;
    fs::create_dir_all(&log_dir)?;

    let (log, _guard) = taxy::log::create_layer(
        &log_dir,
        args.log,
        "taxy.log",
        args.log_level,
        args.log_format,
    )?;
    let (access_log, _guard) = taxy::log::create_layer(
        &log_dir,
        args.access_log,
        "access.log",
        args.access_log_level,
        args.log_format,
    )?;
    let db = DatabaseLayer::new(&log_dir.join("log.db"), args.log_level).await?;

    let access_log_filter =
        filter::filter_fn(|metadata| metadata.target().starts_with("taxy::access_log"));
    let is_span = filter::filter_fn(|metadata| metadata.is_span());
    tracing_subscriber::registry()
        .with(log.with_filter(access_log_filter.clone().not()))
        .with(access_log.with_filter(access_log_filter.or(is_span)))
        .with(db)
        .init();

    let config_dir = get_config_dir(args.config_dir)?;
    fs::create_dir_all(&config_dir)?;

    let config = FileStorage::new(&config_dir);
    let app_info = new_appinfo(&config_dir, &log_dir);

    let (server, channels) = Server::new(app_info.clone(), config).await;
    let server_task = tokio::spawn(server.start());
    let event_send = channels.event.clone();

    let webui_enabled = !args.no_webui;
    tokio::select! {
        r = taxy::admin::start_admin(app_info, args.webui, channels.command, channels.callback, channels.event), if webui_enabled => {
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

async fn add_user(args: taxy::args::AddUserArgs) -> anyhow::Result<()> {
    let config_dir = get_config_dir(args.config_dir)?;
    let config = FileStorage::new(&config_dir);
    let password = if let Some(password) = args.password {
        password
    } else {
        rpassword::prompt_password("password?: ")?
    };
    let account = config.add_account(&args.name, &password, args.totp).await?;
    if let Some(totp) = account.totp {
        println!("\nUse this code to setup your TOTP client:\n{totp}\n");
    }
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
