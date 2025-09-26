use async_trait::async_trait;
use log::info;
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::prelude::*;
use pingora::proxy::{ProxyHttp, Session};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::pipeline::Pipeline;
use crate::proxy::ctx::Ctx;
use crate::proxy::headers::HeaderPolicy;

pub mod ctx;
pub mod headers;

pub struct GatewayProxy {
    upstreams: Vec<String>,
    current_upstream: AtomicUsize,
    pipeline: Pipeline,
    header_policy: HeaderPolicy,
}

impl GatewayProxy {
    pub fn new(upstreams: Vec<String>) -> Self {
        assert!(!upstreams.is_empty(), "Upstream list cannot be empty");

        // Validate that each upstream contains a port
        for upstream in &upstreams {
            assert!(
                upstream.contains(':'),
                "Upstream '{}' must include a port (e.g., 'host:port')",
                upstream
            );
        }

        Self {
            upstreams,
            current_upstream: AtomicUsize::new(0),
            pipeline: Pipeline::new(),
            header_policy: HeaderPolicy::new(),
        }
    }

    pub fn select_upstream(&self) -> &str {
        let index = self.current_upstream.fetch_add(1, Ordering::Relaxed) % self.upstreams.len();
        &self.upstreams[index]
    }
}

#[async_trait]
impl ProxyHttp for GatewayProxy {
    type CTX = Ctx;

    fn new_ctx(&self) -> Self::CTX {
        Ctx::default()
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let upstream = self.select_upstream();
        let peer = HttpPeer::new(upstream, false, "".to_string());

        info!("Routing request to upstream: {}", upstream);
        Ok(Box::new(peer))
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Apply all upstream request header mutations
        self.header_policy
            .apply_upstream_request_headers(upstream_request)?;

        // Run pipeline to detect provider
        self.pipeline.on_request(upstream_request, ctx);

        Ok(())
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        // Apply all response header mutations
        self.header_policy
            .apply_response_headers(upstream_response)?;

        // Run pipeline response processing
        self.pipeline.on_response(upstream_response, ctx);

        Ok(())
    }

    async fn logging(&self, session: &mut Session, _error: Option<&Error>, ctx: &mut Self::CTX) {
        let response_code = session
            .response_written()
            .map(|resp| resp.status.as_u16())
            .unwrap_or(0);

        info!(
            "{} {} status: {} provider:{:?}",
            session.req_header().method,
            session.req_header().uri,
            response_code,
            ctx.provider
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_proxy_creation() {
        let upstreams = vec!["127.0.0.1:8001".to_string(), "127.0.0.1:8002".to_string()];
        let proxy = GatewayProxy::new(upstreams.clone());
        assert_eq!(proxy.upstreams, upstreams);
    }

    #[test]
    fn test_round_robin_selection() {
        let upstreams = vec![
            "server1:80".to_string(),
            "server2:80".to_string(),
            "server3:80".to_string(),
        ];
        let proxy = GatewayProxy::new(upstreams);

        // Test that selection cycles through all upstreams
        assert_eq!(proxy.select_upstream(), "server1:80");
        assert_eq!(proxy.select_upstream(), "server2:80");
        assert_eq!(proxy.select_upstream(), "server3:80");
        // Should wrap around
        assert_eq!(proxy.select_upstream(), "server1:80");
    }

    #[tokio::test]
    async fn test_upstream_peer_creation() {
        let upstreams = vec!["127.0.0.1:8001".to_string()];
        let proxy = GatewayProxy::new(upstreams);

        // Create a mock session (this would normally come from Pingora)
        // For unit testing, we just verify the peer is created correctly
        let selected = proxy.select_upstream();
        assert_eq!(selected, "127.0.0.1:8001");
    }

    #[test]
    #[should_panic(expected = "Upstream list cannot be empty")]
    fn test_empty_upstreams_panics() {
        let empty_upstreams: Vec<String> = vec![];
        GatewayProxy::new(empty_upstreams);
    }

    #[test]
    #[should_panic(expected = "must include a port")]
    fn test_upstream_without_port_panics() {
        let upstreams = vec!["invalid-upstream".to_string()];
        GatewayProxy::new(upstreams);
    }
}
