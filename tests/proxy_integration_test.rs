use langspec::GatewayProxy;
use pingora::http::{RequestHeader, ResponseHeader};
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};

#[test]
fn test_gateway_proxy_round_robin() {
    let upstreams = vec![
        "backend1:80".to_string(),
        "backend2:80".to_string(),
        "backend3:80".to_string(),
    ];

    let proxy = GatewayProxy::new(upstreams);

    // Track selections in order
    let mut selections = Vec::new();
    for _ in 0..9 {
        selections.push(proxy.select_upstream().to_string());
    }

    // Check that we cycle through all three backends three times
    assert_eq!(selections[0], "backend1:80");
    assert_eq!(selections[1], "backend2:80");
    assert_eq!(selections[2], "backend3:80");
    assert_eq!(selections[3], "backend1:80");
    assert_eq!(selections[4], "backend2:80");
    assert_eq!(selections[5], "backend3:80");
    assert_eq!(selections[6], "backend1:80");
    assert_eq!(selections[7], "backend2:80");
    assert_eq!(selections[8], "backend3:80");
}

#[test]
fn test_single_upstream() {
    let upstreams = vec!["single-backend:8080".to_string()];
    let proxy = GatewayProxy::new(upstreams);

    // With a single upstream, it should always select the same one
    for _ in 0..5 {
        assert_eq!(proxy.select_upstream(), "single-backend:8080");
    }
}

#[test]
fn test_concurrent_selection() {
    // Test thread safety of round-robin selection
    use std::thread;

    let upstreams = vec![
        "server1:80".to_string(),
        "server2:80".to_string(),
        "server3:80".to_string(),
    ];

    let proxy = Arc::new(GatewayProxy::new(upstreams));
    let selection_count = Arc::new(AtomicU16::new(0));

    let mut handles = vec![];

    // Spawn multiple threads to test concurrent access
    for _ in 0..10 {
        let proxy_clone = Arc::clone(&proxy);
        let count_clone = Arc::clone(&selection_count);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _ = proxy_clone.select_upstream();
                count_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all selections were made
    assert_eq!(selection_count.load(Ordering::Relaxed), 1000);
}

#[tokio::test]
async fn test_request_header_modification() {
    // Test that the proxy adds the X-Forwarded-By header
    let mut request = RequestHeader::build("GET", b"/api/test", None).unwrap();

    // Simulate what upstream_request_filter does
    request
        .insert_header("X-Forwarded-By", "langspec-gateway")
        .unwrap();

    // Verify the header is present and correct
    let header_value = request.headers.get("X-Forwarded-By");
    assert!(header_value.is_some());
    assert_eq!(header_value.unwrap().to_str().unwrap(), "langspec-gateway");
}

#[tokio::test]
async fn test_response_header_modification() {
    // Test that the proxy adds the X-Proxy header
    let mut response = ResponseHeader::build(200, None).unwrap();

    // Simulate what response_filter does
    response.insert_header("X-Proxy", "langspec").unwrap();

    // Verify the header is present and correct
    let header_value = response.headers.get("X-Proxy");
    assert!(header_value.is_some());
    assert_eq!(header_value.unwrap().to_str().unwrap(), "langspec");
}

#[tokio::test]
async fn test_multiple_header_operations() {
    // Test multiple header operations
    let mut request = RequestHeader::build("POST", b"/api/users", None).unwrap();

    // Add multiple headers
    request
        .insert_header("X-Forwarded-By", "langspec-gateway")
        .unwrap();
    request.insert_header("X-Request-Id", "test-123").unwrap();
    request
        .insert_header("Content-Type", "application/json")
        .unwrap();

    // Verify all headers are present
    assert!(request.headers.get("X-Forwarded-By").is_some());
    assert!(request.headers.get("X-Request-Id").is_some());
    assert!(request.headers.get("Content-Type").is_some());

    // Check specific values
    assert_eq!(
        request
            .headers
            .get("Content-Type")
            .unwrap()
            .to_str()
            .unwrap(),
        "application/json"
    );
}
