use pingora::http::{RequestHeader, ResponseHeader};
use pingora::prelude::*;
use std::net::IpAddr;

/// Centralized header mutation policies for the langspec gateway.
///
/// This module encapsulates all header manipulation logic to:
/// - Prevent scattered header mutations across the codebase
/// - Provide consistent header policies
/// - Enable easy addition of new headers (X-Forwarded-For, X-Request-Id, etc.)
/// - Maintain observability and security best practices
pub struct HeaderPolicy {
    gateway_name: &'static str,
    proxy_name: &'static str,
}

impl HeaderPolicy {
    pub fn new() -> Self {
        Self {
            gateway_name: "langspec-gateway",
            proxy_name: "langspec",
        }
    }

    /// Apply all upstream request header mutations.
    /// This is called once per request in upstream_request_filter.
    ///
    /// To add new request headers in the future:
    /// 1. Add method to HeaderPolicy (see examples below)
    /// 2. Call it here - NO changes needed to ProxyHttp
    /// 3. Add test coverage
    pub fn apply_upstream_request_headers(&self, request: &mut RequestHeader) -> Result<()> {
        // Core forwarding headers
        self.add_forwarded_by_header(request)?;

        // Future headers will be added here:
        // self.add_forwarded_for_header(request, client_ip)?;
        // self.add_request_id_header(request)?;
        // self.add_trace_headers(request)?;

        Ok(())
    }

    /// Apply all response header mutations.
    /// This is called once per response in response_filter.
    ///
    /// To add new response headers in the future:
    /// 1. Add method to HeaderPolicy (see examples below)
    /// 2. Call it here - NO changes needed to ProxyHttp
    /// 3. Add test coverage
    pub fn apply_response_headers(&self, response: &mut ResponseHeader) -> Result<()> {
        // Core proxy identification
        self.add_proxy_header(response)?;

        // Future headers will be added here:
        // self.add_security_headers(response)?;
        // self.add_cache_headers(response)?;
        // self.add_cors_headers(response)?;

        Ok(())
    }

    /// Add X-Forwarded-By header to identify the gateway
    fn add_forwarded_by_header(&self, request: &mut RequestHeader) -> Result<()> {
        request.insert_header("X-Forwarded-By", self.gateway_name)?;
        Ok(())
    }

    /// Add X-Proxy header to identify the proxy software
    fn add_proxy_header(&self, response: &mut ResponseHeader) -> Result<()> {
        response.insert_header("X-Proxy", self.proxy_name)?;
        Ok(())
    }

    /// Future: Add X-Forwarded-For header with client IP
    #[allow(dead_code)]
    fn add_forwarded_for_header(
        &self,
        request: &mut RequestHeader,
        client_ip: IpAddr,
    ) -> Result<()> {
        // Check if X-Forwarded-For already exists and append, or create new
        if let Some(existing) = request.headers.get("X-Forwarded-For") {
            if let Ok(existing_str) = existing.to_str() {
                let new_value = format!("{}, {}", existing_str, client_ip);
                request.remove_header("X-Forwarded-For");
                request.insert_header("X-Forwarded-For", &new_value)?;
            }
        } else {
            request.insert_header("X-Forwarded-For", client_ip.to_string())?;
        }
        Ok(())
    }

    /// Future: Add X-Request-Id header for request tracing
    #[allow(dead_code)]
    fn add_request_id_header(&self, request: &mut RequestHeader) -> Result<()> {
        // Only add if not already present (preserve upstream request IDs)
        if request.headers.get("X-Request-Id").is_none() {
            // TODO: Generate UUID or use other request ID strategy
            let request_id = self.generate_request_id();
            request.insert_header("X-Request-Id", &request_id)?;
        }
        Ok(())
    }

    /// Future: Generate request ID (placeholder implementation)
    #[allow(dead_code)]
    fn generate_request_id(&self) -> String {
        // TODO: Implement proper request ID generation
        // Could use UUID, nanoid, or other strategy
        format!("req_{}", std::process::id())
    }

    /// Future: Add security headers to responses
    #[allow(dead_code)]
    fn add_security_headers(&self, response: &mut ResponseHeader) -> Result<()> {
        // Only add if not already present (don't override upstream policies)
        if response.headers.get("X-Content-Type-Options").is_none() {
            response.insert_header("X-Content-Type-Options", "nosniff")?;
        }
        if response.headers.get("X-Frame-Options").is_none() {
            response.insert_header("X-Frame-Options", "DENY")?;
        }
        Ok(())
    }
}

impl Default for HeaderPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy function for backward compatibility
/// TODO: Remove once all callers use HeaderPolicy
pub fn add_forwarded_headers(request: &mut RequestHeader) -> Result<()> {
    let policy = HeaderPolicy::new();
    policy.add_forwarded_by_header(request)
}
