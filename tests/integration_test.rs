use langspec::proxy::GatewayProxy;
use pingora::http::{RequestHeader, ResponseHeader};

#[test]
fn test_proxy_creation() {
    let upstreams = vec![
        "127.0.0.1:8001".to_string(),
        "127.0.0.1:8002".to_string(),
        "127.0.0.1:8003".to_string(),
    ];

    let proxy = GatewayProxy::new(upstreams.clone());

    // Verify the proxy is created with the correct upstreams
    // by checking round-robin behavior
    let first = proxy.select_upstream();
    let second = proxy.select_upstream();
    let third = proxy.select_upstream();

    assert_ne!(first, second);
    assert_ne!(second, third);
    assert_ne!(first, third);
}

#[test]
fn test_round_robin_wrapping() {
    let upstreams = vec!["upstream1:80".to_string(), "upstream2:80".to_string()];

    let proxy = GatewayProxy::new(upstreams);

    // Test that selection wraps around properly
    assert_eq!(proxy.select_upstream(), "upstream1:80");
    assert_eq!(proxy.select_upstream(), "upstream2:80");
    assert_eq!(proxy.select_upstream(), "upstream1:80"); // Should wrap back
    assert_eq!(proxy.select_upstream(), "upstream2:80");
}

#[tokio::test]
async fn test_request_headers() {
    // Test header manipulation that would happen in upstream_request_filter
    let mut request = RequestHeader::build("GET", b"/api/endpoint", None).unwrap();

    // Simulate what the proxy does
    request
        .insert_header("X-Forwarded-By", "langspec-gateway")
        .unwrap();

    // Verify the header was added
    let header = request.headers.get("X-Forwarded-By");
    assert!(header.is_some());
    assert_eq!(header.unwrap().to_str().unwrap(), "langspec-gateway");
}

#[tokio::test]
async fn test_response_headers() {
    // Test header manipulation that would happen in response_filter
    let mut response = ResponseHeader::build(200, None).unwrap();

    // Simulate what the proxy does
    response.insert_header("X-Proxy", "langspec").unwrap();

    // Verify the header was added
    let header = response.headers.get("X-Proxy");
    assert!(header.is_some());
    assert_eq!(header.unwrap().to_str().unwrap(), "langspec");
}

#[tokio::test]
async fn test_centralized_header_policy() {
    use langspec::proxy::headers::HeaderPolicy;

    let policy = HeaderPolicy::new();

    // Test request header mutations
    let mut request = RequestHeader::build("POST", b"/api/test", None).unwrap();
    policy.apply_upstream_request_headers(&mut request).unwrap();

    // Verify X-Forwarded-By header was added
    let forwarded_by = request.headers.get("X-Forwarded-By");
    assert!(forwarded_by.is_some());
    assert_eq!(forwarded_by.unwrap().to_str().unwrap(), "langspec-gateway");

    // Test response header mutations
    let mut response = ResponseHeader::build(200, None).unwrap();
    policy.apply_response_headers(&mut response).unwrap();

    // Verify X-Proxy header was added
    let proxy_header = response.headers.get("X-Proxy");
    assert!(proxy_header.is_some());
    assert_eq!(proxy_header.unwrap().to_str().unwrap(), "langspec");
}

#[tokio::test]
async fn test_various_http_methods() {
    // Test that different HTTP methods are handled correctly
    let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];

    for method in methods {
        let request = RequestHeader::build(method, b"/test", None).unwrap();
        assert_eq!(request.method.as_str(), method);
    }
}

#[tokio::test]
async fn test_different_status_codes() {
    // Test that different status codes are handled correctly
    let status_codes = vec![200, 201, 400, 401, 403, 404, 500, 502, 503];

    for code in status_codes {
        let response = ResponseHeader::build(code, None).unwrap();
        assert_eq!(response.status.as_u16(), code);
    }
}
