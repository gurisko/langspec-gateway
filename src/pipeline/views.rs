use pingora::http::RequestHeader;

/// A read-only wrapper around Pingora's RequestHeader to decouple provider code from Pingora types
pub struct RequestView<'a> {
    inner: &'a RequestHeader,
}

impl<'a> RequestView<'a> {
    pub fn new(request_header: &'a RequestHeader) -> Self {
        Self {
            inner: request_header,
        }
    }

    pub fn method(&self) -> &str {
        self.inner.method.as_str()
    }

    pub fn path(&self) -> &str {
        self.inner.uri.path()
    }

    /// Get the Host header value. Pingora's HeaderMap is case-insensitive.
    pub fn host(&self) -> Option<&str> {
        self.inner.headers.get("host").and_then(|h| h.to_str().ok())
    }

    /// Get a header value by key. Pingora's HeaderMap is case-insensitive.
    /// Use lowercase keys for consistency.
    pub fn header(&self, key: &str) -> Option<&str> {
        self.inner.headers.get(key).and_then(|h| h.to_str().ok())
    }

    /// Get Authorization header value
    pub fn authorization(&self) -> Option<&str> {
        self.header("authorization")
    }

    /// Check if this looks like AWS SigV4 authentication
    pub fn has_aws_sigv4(&self) -> bool {
        self.authorization()
            .map(|auth| auth.starts_with("AWS4-HMAC-SHA256"))
            .unwrap_or(false)
            || self.header("x-amz-date").is_some()
            || self.header("x-amz-security-token").is_some()
    }

    /// Check if this has Bearer token authentication
    pub fn has_bearer_auth(&self) -> bool {
        self.authorization()
            .map(|auth| auth.starts_with("Bearer "))
            .unwrap_or(false)
    }

    /// Check if host ends with a given suffix (for domain matching)
    pub fn host_ends_with(&self, suffix: &str) -> bool {
        self.host()
            .map(|host| host.ends_with(suffix))
            .unwrap_or(false)
    }
}
