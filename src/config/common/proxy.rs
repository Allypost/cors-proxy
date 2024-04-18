use std::{
    collections::HashSet, convert::Into, net::ToSocketAddrs, string::ToString, time::Duration,
};

use super::timeframe::Timeframe;

#[derive(Debug, Clone, clap::Args)]
#[clap(next_help_heading = "Proxy options")]
pub struct ProxyArgs {
    /// The address to proxy requests to.
    ///
    /// For example, `127.0.0.1:80` or `my-computer.local:1234`
    #[clap(short, long, value_name = "ADDRESS", env="CORS_PROXY_PROXY_TO", value_parser = parse_socket_addr)]
    pub proxy_to: std::net::SocketAddr,

    /// Set which host names are allowed to be proxied.
    ///
    /// By default, all hosts are allowed.
    ///
    /// For example, `allypost.net`, `www.allypost.net` or `a.allypost.net,b.allypost.net`
    #[clap(
        short = 'H',
        long,
        value_name = "HOST",
        env = "CORS_PROXY_HOST_ALLOWLIST"
    )]
    pub host_allowlist: Vec<String>,

    /// Explicitly set whether to use TLS on first connection.
    ///
    /// By default, TLS is first tried and falls back to plain HTTP.
    #[clap(
        long,
        env = "CORS_PROXY_USE_TLS",
        num_args(0..=1),
        hide_possible_values = true,
        default_missing_value = "true"
    )]
    pub use_tls: Option<bool>,

    /// How long connect() call should be wait
    /// before it returns a timeout error.
    ///
    /// Eg. `300ms` or `5s``
    ///
    /// Defaults to `5s`
    #[clap(
        long,
        value_parser = Timeframe::parse_str,
        default_value = "5s",
        env = "CORS_PROXY_CONNECTION_TIMEOUT"
    )]
    pub connection_timeout: Timeframe,

    /// How long the overall connection establishment should take
    /// before a timeout error is returned.
    ///
    /// Eg. `300ms` or `5s`
    ///
    /// Defaults to `10s`
    #[clap(
        long,
        value_parser = Timeframe::parse_str,
        default_value = "10s",
        env = "CORS_PROXY_TOTAL_CONNECTION_TIMEOUT"
    )]
    pub total_connection_timeout: Timeframe,

    /// If the connection can be reused, how long the connection should wait
    /// to be reused before it shuts down.
    ///
    /// Eg. `300ms` or `5s`
    #[clap(
        long,
        value_parser = Timeframe::parse_str,
        env = "CORS_PROXY_IDLE_TIMEOUT"
    )]
    pub idle_timeout: Option<Timeframe>,
}
impl ProxyArgs {
    pub fn to_config(&self) -> ProxyConfig {
        ProxyConfig::from_args(self)
    }
}

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub proxy_to: std::net::SocketAddr,

    pub host_allowlist: HashSet<String>,

    pub use_tls: Option<bool>,

    pub connection_timeout: Duration,

    pub total_connection_timeout: Duration,

    pub idle_timeout: Option<Duration>,
}
impl ProxyConfig {
    fn from_args(args: &ProxyArgs) -> Self {
        Self {
            proxy_to: args.proxy_to,
            host_allowlist: args
                .host_allowlist
                .clone()
                .into_iter()
                .flat_map(|x| {
                    x.split_terminator(',')
                        .map(str::trim)
                        .map(str::to_lowercase)
                        .collect::<Vec<_>>()
                })
                .collect(),
            use_tls: args.use_tls,
            connection_timeout: args.connection_timeout.into(),
            total_connection_timeout: args.total_connection_timeout.into(),
            idle_timeout: args.idle_timeout.map(Into::into),
        }
    }
}

fn parse_socket_addr(s: &str) -> Result<std::net::SocketAddr, String> {
    if !s.contains(':') {
        return Err("Address must contain a port (eg. 127.0.0.1:80)".to_string());
    }

    s.to_socket_addrs()
        .map_err(|e| format!("{e:?}"))?
        .next()
        .ok_or_else(|| "Failed to parse address".to_string())
}
