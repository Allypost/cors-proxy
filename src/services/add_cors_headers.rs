use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use http::header;
use pingora::{http::ResponseHeader, prelude::*};
use tracing::{debug, field, info, trace, warn};

use crate::config::common::proxy::ProxyConfig;

#[derive(Debug)]
pub struct AddCorsHeaders {
    config: ProxyConfig,
    use_tls: AtomicBool,
}
impl AddCorsHeaders {
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config,
            use_tls: AtomicBool::new(true),
        }
    }

    fn using_tls(&self) -> bool {
        if let Some(use_tls) = self.config.use_tls {
            return use_tls;
        }

        self.use_tls.load(Ordering::Relaxed)
    }

    fn set_use_tls(&self, use_tls: bool) -> bool {
        if self.config.use_tls.unwrap_or_default() {
            debug!("Force TLS is enabled, not setting use_tls to {}", use_tls);
            return true;
        }

        debug!("Setting use_tls to {}", use_tls);
        self.use_tls.store(use_tls, Ordering::Relaxed);

        use_tls
    }
}

#[derive(Debug)]
pub struct AddCorsHeadersCtx {
    request_id: uuid::fmt::Simple,
    request_start: std::time::Instant,
    tracing_span: tracing::Span,
}
impl AddCorsHeadersCtx {
    fn new(config: &ProxyConfig) -> Self {
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

        let allowlist = &self.config.host_allowlist;

        if allowlist.is_empty() {
            trace!("Host allowlist is empty");
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

        if !allowlist.contains(&request_host) {
            debug!(
                host = ?request_host,
                ?allowlist,
                "Host header not in allowlist"
            );

            info!(host = ?request_host, "Host header not in allowlist");

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

        let peer = {
            let mut peer = if self.using_tls() {
                HttpPeer::new(self.config.proxy_to, true, String::new())
            } else {
                HttpPeer::new(self.config.proxy_to, false, String::new())
            };

            peer.options.connection_timeout = Some(self.config.connection_timeout);
            peer.options.total_connection_timeout = Some(self.config.total_connection_timeout);
            peer.options.idle_timeout = self.config.idle_timeout;

            peer
        };

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
        let origin = session.get_header(header::ORIGIN);

        trace!(?origin, "Starting response filter");

        if !self.config.origin_allowlist.is_empty()
            && !origin
                .and_then(|x| x.to_str().ok())
                .map(str::to_lowercase)
                .is_some_and(|x| self.config.origin_allowlist.contains(&x))
        {
            debug!(origin = ?origin, "Origin not in allowlist, not adding CORS headers");

            return Ok(());
        }

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

        if let Some(origin) = origin {
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

    fn fail_to_connect(
        &self,
        _session: &mut Session,
        _peer: &HttpPeer,
        ctx: &mut Self::CTX,
        e: Box<Error>,
    ) -> Box<Error> {
        let use_tls = self.using_tls();
        debug!(?ctx, ?e, ?use_tls, "Failed to connect to upstream");

        if use_tls && !self.set_use_tls(false) {
            let mut e = e.into_down();
            e.set_retry(true);

            return e;
        }

        warn!(
            ctx = ?ctx,
            ?e,
            "Failed to connect to upstream. Aborting request."
        );

        e
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
