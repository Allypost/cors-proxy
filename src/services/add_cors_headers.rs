use std::collections::HashSet;

use async_trait::async_trait;
use http::header;
use pingora::{http::ResponseHeader, prelude::*};
use tracing::{debug, field, info, trace, warn};

#[derive(Debug, Clone)]
pub struct AddCorsHeadersConfig {
    pub proxy_to: std::net::SocketAddr,
    pub host_whitelist: HashSet<String>,
}

#[derive(Debug)]
pub struct AddCorsHeaders {
    config: AddCorsHeadersConfig,
}
impl AddCorsHeaders {
    pub const fn new(config: AddCorsHeadersConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug)]
pub struct AddCorsHeadersCtx {
    request_id: uuid::fmt::Simple,
    request_start: std::time::Instant,
    tracing_span: tracing::Span,
}
impl AddCorsHeadersCtx {
    fn new(config: &AddCorsHeadersConfig) -> Self {
        let t = config.proxy_to;
        let m = field::Empty;
        let p = field::Empty;
        let id = uuid::Uuid::now_v7().simple();
        let dur = field::Empty;

        Self {
            request_id: id,
            request_start: std::time::Instant::now(),
            tracing_span: tracing::span!(tracing::Level::INFO, "req", %t, %id, m, p, dur),
        }
    }
}

#[async_trait]
impl ProxyHttp for AddCorsHeaders {
    type CTX = AddCorsHeadersCtx;

    fn new_ctx(&self) -> Self::CTX {
        Self::CTX::new(&self.config)
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        let _span = ctx.tracing_span.enter();
        let req_header = session.req_header();

        ctx.tracing_span
            .record("m", field::display(req_header.method.to_string()));
        ctx.tracing_span
            .record("p", field::display(req_header.uri.to_string()));

        info!("Incoming request");

        let whitelist = &self.config.host_whitelist;

        if whitelist.is_empty() {
            trace!("Host whitelist is empty");
            return Ok(false);
        }

        let request_host = session
            .get_header("Host")
            .ok_or_else(|| {
                pingora::Error::explain(ErrorType::UnknownError, "Missing Host header").into_down()
            })
            .and_then(|x| {
                x.to_str().map_err(|e| {
                    warn!(?e, "Failed to parse Host header");

                    pingora::Error::because(
                        ErrorType::UnknownError,
                        "Failed to parse Host header",
                        e,
                    )
                    .into_in()
                })
            })
            .map(str::to_lowercase);

        let request_host = match request_host {
            Ok(x) => x,
            Err(e) => {
                debug!(?e, "Failed to parse Host header");

                session
                    .respond_error(http::StatusCode::BAD_REQUEST.as_u16())
                    .await;

                return Ok(true);
            }
        };

        debug!(host = ?request_host, "Got host header");

        if !whitelist.contains(&request_host) {
            debug!(
                host = ?request_host,
                whitelist = ?whitelist,
                "Host header not in whitelist"
            );

            info!(host = ?request_host, "Host header not in whitelist");

            session
                .respond_error(http::StatusCode::BAD_REQUEST.as_u16())
                .await;

            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let _span = ctx.tracing_span.enter();

        let peer = HttpPeer::new(self.config.proxy_to, false, String::new());

        trace!(peer = ?peer, "Created peer");

        Ok(Box::new(peer))
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        let _span = ctx.tracing_span.enter();

        if let Some(host) = session.get_header("Host") {
            upstream_request.insert_header("Host", host)?;
        }

        trace!(headers = ?upstream_request.headers, "Modified upstream request");

        Ok(())
    }

    async fn response_filter(
        &self,
        session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        let _span = ctx.tracing_span.enter();

        trace!("Starting response filter");

        {
            let upstream_headers = upstream_response
                .headers
                .iter()
                .map(|x| x.0.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            trace!(
                headers = ?upstream_headers,
                "Allowing all upstream headers",
            );

            upstream_response
                .insert_header(header::ACCESS_CONTROL_EXPOSE_HEADERS, upstream_headers)?;
        }

        // upstream_response.insert_header(header::ACCESS_CONTROL_MAX_AGE, "1")?;

        upstream_response.append_header(header::VARY, "Origin")?;

        if let Some(origin) = session.get_header(header::ORIGIN) {
            debug!(origin = ?origin, "Adding origin-specific CORS headers");
            upstream_response.insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)?;
            upstream_response.insert_header(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true")?;
        } else {
            debug!("Adding generic CORS headers");
            upstream_response.insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")?;
            upstream_response.insert_header(header::ACCESS_CONTROL_ALLOW_HEADERS, "*")?;
            upstream_response.insert_header(header::ACCESS_CONTROL_ALLOW_METHODS, "*")?;
        }

        if let Some(header) = session.get_header(header::ACCESS_CONTROL_REQUEST_HEADERS) {
            trace!(?header, "Adding Access-Control-Request-Headers header");
            upstream_response.insert_header(header::ACCESS_CONTROL_ALLOW_HEADERS, header)?;
        }

        if let Some(header) = session.get_header(header::ACCESS_CONTROL_REQUEST_METHOD) {
            trace!(?header, "Adding Access-Control-Request-Method header");
            upstream_response.insert_header(header::ACCESS_CONTROL_ALLOW_METHODS, header)?;
        }

        upstream_response.append_header("X-CorsProxy-Request-Id", ctx.request_id.to_string())?;

        Ok(())
    }

    async fn logging(&self, _session: &mut Session, err: Option<&Error>, ctx: &mut Self::CTX) {
        let _span = ctx.tracing_span.enter();

        {
            let dur = std::time::Instant::now().duration_since(ctx.request_start);
            ctx.tracing_span.record("dur", field::debug(dur));
        }

        if let Some(err) = err {
            warn!(?err, "Done with error");
        } else {
            info!("Done");
        }
    }

    fn suppress_error_log(&self, _session: &Session, _ctx: &Self::CTX, _error: &Error) -> bool {
        true
    }
}
