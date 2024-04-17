use clap::Parser;
use once_cell::sync::Lazy;

use self::{
    args::Args,
    common::{pingora::PingoraConfig, proxy::ProxyConfig, server::ServerConfig},
};

pub mod args;
pub mod common;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::new);

#[derive(Debug, Clone)]
pub struct Config {
    pub pingora: PingoraConfig,
    pub proxy: ProxyConfig,
    pub server: ServerConfig,
}
impl Config {
    fn new() -> Self {
        Self::from_args(Args::parse())
    }

    fn from_args(args: Args) -> Self {
        Self {
            pingora: args.pingora,
            proxy: args.proxy.to_config(),
            server: args.server,
        }
    }
}
