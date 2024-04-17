use clap::ArgAction;

use super::common::{pingora::PingoraConfig, proxy::ProxyArgs, server::ServerConfig};

#[derive(Debug, clap::Parser)]
#[clap(disable_help_flag = true)]
pub(super) struct Args {
    #[clap(flatten)]
    pub pingora: PingoraConfig,

    #[clap(flatten)]
    pub proxy: ProxyArgs,

    #[clap(flatten)]
    pub server: ServerConfig,

    /// Print help text
    #[clap(action = ArgAction::Help, long)]
    help: Option<bool>,
}
