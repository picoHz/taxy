use crate::log::LogFormat;
use clap::{Args, Parser, Subcommand};
use std::{net::SocketAddr, path::PathBuf};
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start server
    Start(StartArgs),
    /// Add user
    AddUser(AddUserArgs),
}

#[derive(Args)]
pub struct StartArgs {
    #[clap(long, value_name = "FILE", env = "TAXY_LOG")]
    pub log: Option<PathBuf>,

    #[clap(long, value_name = "FILE", env = "TAXY_ACCESS_LOG")]
    pub access_log: Option<PathBuf>,

    #[clap(
        long,
        short,
        value_name = "LEVEL",
        default_value = "info",
        env = "TAXY_LOG_LEVEL"
    )]
    pub log_level: LevelFilter,

    #[clap(
        long,
        short,
        value_name = "LEVEL",
        default_value = "info",
        env = "TAXY_ACCESS_LOG_LEVEL"
    )]
    pub access_log_level: LevelFilter,

    #[clap(
        long,
        value_enum,
        value_name = "FORMAT",
        default_value = "text",
        env = "TAXY_LOG_FORMAT"
    )]
    pub log_format: LogFormat,

    #[clap(
        long,
        short,
        value_name = "ADDR",
        default_value = "127.0.0.1:46492",
        env = "TAXY_WEBUI"
    )]
    pub webui: SocketAddr,

    #[clap(long, short, env = "TAXY_NO_WEBUI")]
    pub no_webui: bool,

    #[clap(long, short, value_name = "DIR", env = "TAXY_CONFIG_DIR")]
    pub config_dir: Option<PathBuf>,

    #[clap(long, short = 'd', value_name = "DIR", env = "TAXY_LOG_DIR")]
    pub log_dir: Option<PathBuf>,

    #[clap(long)]
    pub use_keystore: bool,
}

#[derive(Args)]
pub struct AddUserArgs {
    pub name: String,

    #[clap(long, short, value_name = "PASSWORD")]
    pub password: Option<String>,

    #[clap(long, short, value_name = "DIR", env = "TAXY_CONFIG_DIR")]
    pub config_dir: Option<PathBuf>,
}
