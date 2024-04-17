use std::{collections::HashSet, net::ToSocketAddrs, string::ToString};

use crate::services::add_cors_headers::AddCorsHeadersConfig;

#[derive(Debug, Clone, clap::Args)]
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
    pub host_allowlist: Option<Vec<String>>,
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
}
impl ProxyConfig {
    fn from_args(args: &ProxyArgs) -> Self {
        Self {
            proxy_to: args.proxy_to,
            host_allowlist: args
                .host_allowlist
                .clone()
                .unwrap_or_default()
                .into_iter()
                .flat_map(|x| {
                    x.split_terminator(',')
                        .map(str::trim)
                        .map(str::to_lowercase)
                        .collect::<Vec<_>>()
                })
                .collect(),
        }
    }

    pub fn as_add_cors_headers_config(&self) -> AddCorsHeadersConfig {
        AddCorsHeadersConfig {
            proxy_to: self.proxy_to,
            host_allowlist: self.host_allowlist.clone().into_iter().collect(),
        }
    }
}

impl From<ProxyConfig> for AddCorsHeadersConfig {
    fn from(config: ProxyConfig) -> Self {
        config.as_add_cors_headers_config()
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
