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
}
