use std::sync::Arc;

use config::CONFIG;
use pingora::{prelude::*, server::configuration::ServerConf, services::Service};
use services::add_cors_headers::AddCorsHeaders;
use tracing::{debug, info};

mod config;
mod services;

fn main() {
    init_log();

    info!("Starting server");
    debug!(config=?*CONFIG, "Starting server");

    let mut server = {
        let opt = CONFIG.pingora.as_pingora_opt();

        Server::new(Some(opt)).expect("Failed to create server")
    };

    server.bootstrap();

    let services = vec![proxy_service(&server.configuration)];

    server.add_services(services);
    server.run_forever();
}

fn proxy_service(conf: &Arc<ServerConf>) -> Box<dyn Service> {
    let config = CONFIG.proxy.clone();
    let threads = num_cpus::get();

    debug!(?config, ?threads, "Creating proxy service");
    let mut service = pingora::proxy::http_proxy_service(conf, AddCorsHeaders::new(config));
    service.threads = Some(threads);

    let addr = CONFIG.server.addr_string();
    info!(?addr, "Adding listener for proxy service");
    service.add_tcp(&addr);

    Box::new(service)
}

fn init_log() {
    use tracing::Level;
    use tracing_subscriber::{
        filter::Directive, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
    };

    let mut base_level = EnvFilter::builder()
        .with_default_directive(Level::WARN.into())
        .parse_lossy("cors_proxy=info");

    let env_directives = std::env::var("CORS_PROXY_LOG_LEVEL")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| match s.parse() {
            Ok(d) => Some(d),
            Err(e) => {
                eprintln!("Failed to parse log level directive {s:?}: {e:?}");
                None
            }
        })
        .collect::<Vec<Directive>>();

    for d in env_directives {
        base_level = base_level.add_directive(d);
    }

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(base_level)
        .try_init()
        .expect("setting default subscriber failed");
}
