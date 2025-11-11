use async_trait::async_trait;
use pingora::{http::ResponseHeader, prelude::*, Error, ErrorType};
use pingora_proxy::{ProxyHttp, Session};

use crate::routing::{RoutingConfig, ServiceConfig};

pub struct BeatProxy {
    routing: RoutingConfig,
}

pub struct ProxyContext {
    pub request_id: String,
    pub service_name: String,
}

impl BeatProxy {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        const PATH: &str = "config/services.json";
        let routing = RoutingConfig::load(PATH)?;

        log::info!("Loaded services:");
        for (key, config) in &routing.services {
            log::info!(
                "  - {} → {}:{} (prefix: {})",
                key,
                config.host,
                config.port,
                config.prefix
            );
        }

        Ok(Self { routing })
    }

    fn format_route(&self, sc: &ServiceConfig, session: &mut Session, route: &str) -> String {
        let uri = session.req_header().uri.clone();

        let skip_prefix = sc
            .skip_prefix_for
            .iter()
            .any(|prefix| route.starts_with(prefix));

        return if skip_prefix {
            format!("/{}", route.trim_start_matches('/'))
        } else {
            format!(
                "/api/v{}{}{}",
                sc.api_version,
                route,
                uri.query().map(|q| format!("?{}", q)).unwrap_or_default()
            )
        };
    }

    pub fn forward_request(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut ProxyContext,
    ) -> Result<()> {
        let Some(service_config) = self.routing.services.get(&ctx.service_name) else {
            return Ok(());
        };

        if !service_config.strip_prefix {
            return Ok(());
        };

        let original_path = session.req_header().uri.path().to_string();
        let Some(stripped) = original_path.strip_prefix(&service_config.prefix) else {
            return Ok(());
        };

        let new_uri = self.format_route(service_config, session, stripped);

        upstream_request.set_uri(new_uri.parse().unwrap());

        Ok(())
    }
}

#[async_trait]
impl ProxyHttp for BeatProxy {
    type CTX = ProxyContext;

    fn new_ctx(&self) -> Self::CTX {
        ProxyContext {
            request_id: uuid::Uuid::new_v4().to_string(),
            service_name: String::new(),
        }
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let path = session.req_header().uri.path();

        let (service_name, service_config) = self.routing.find_service(path).ok_or_else(|| {
            Error::explain(
                ErrorType::HTTPStatus(404),
                format!("No route found for: {}", path),
            )
        })?;

        ctx.service_name = service_name.clone();

        let upstream = format!("{}:{}", service_config.host, service_config.port);
        log::debug!("Routing {} to {}", path, upstream);

        let peer = Box::new(HttpPeer::new(&upstream, false, "".to_string()));

        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        self.forward_request(session, upstream_request, ctx)?;
        upstream_request.insert_header("X-Request-ID", &ctx.request_id)?;

        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_response.insert_header("X-Content-Type-Options", "nosniff")?;
        upstream_response.insert_header("X-Frame-Options", "DENY")?;

        Ok(())
    }
}
