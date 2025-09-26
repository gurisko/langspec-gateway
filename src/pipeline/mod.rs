use crate::provider::ProviderRegistry;
use crate::proxy::ctx::Ctx;
use pingora::http::{RequestHeader, ResponseHeader};
use std::time::Instant;

pub mod views;

use views::RequestView;

pub struct Pipeline {
    provider_registry: ProviderRegistry,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            provider_registry: ProviderRegistry::new(),
        }
    }

    pub fn on_request(&self, request_header: &RequestHeader, ctx: &mut Ctx) {
        let request_view = RequestView::new(request_header);
        ctx.provider = self.provider_registry.detect(&request_view);
        ctx.start = Some(Instant::now());
    }

    pub fn on_response(&self, _response_header: &ResponseHeader, _ctx: &mut Ctx) {
        // Placeholder for future usage parsing
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
