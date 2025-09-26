use langspec::pipeline::views::RequestView;
use langspec::provider::{ProviderKind, ProviderRegistry};
use langspec::proxy::ctx::Ctx;
use pingora::http::RequestHeader;

fn create_test_request(
    method: &str,
    uri: &str,
    host: Option<&str>,
    headers: &[(&str, &str)],
) -> RequestHeader {
    let mut request = RequestHeader::build(method, uri.as_bytes(), None).unwrap();

    if let Some(host_value) = host {
        request.insert_header("host", host_value).unwrap();
    }

    for (key, value) in headers {
        request
            .insert_header(key.to_string(), value.to_string())
            .unwrap();
    }

    request
}

#[test]
fn test_openai_detection_by_host() {
    let request = create_test_request("POST", "/v1/chat/completions", Some("api.openai.com"), &[]);
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);
}

#[test]
fn test_openai_detection_by_path() {
    let request = create_test_request("POST", "/v1/chat/completions", Some("example.com"), &[]);
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);
}

#[test]
fn test_openai_detection_by_header() {
    let request = create_test_request(
        "POST",
        "/api/chat",
        Some("example.com"),
        &[("OpenAI-Organization", "org-123")],
    );
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);
}

#[test]
fn test_bedrock_detection_by_host() {
    let request = create_test_request(
        "POST",
        "/model/anthropic.claude-3",
        Some("bedrock-runtime.us-east-1.amazonaws.com"),
        &[],
    );
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    assert_eq!(registry.detect(&request_view), ProviderKind::Bedrock);
}

#[test]
fn test_bedrock_detection_by_path_with_aws_context() {
    // Path pattern with AWS host context
    let request = create_test_request("POST", "/converse", Some("api.amazonaws.com"), &[]);
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    assert_eq!(registry.detect(&request_view), ProviderKind::Bedrock);
}

#[test]
fn test_unknown_detection() {
    let request = create_test_request("GET", "/api/status", Some("example.com"), &[]);
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    assert_eq!(registry.detect(&request_view), ProviderKind::Unknown);
}

#[test]
fn test_ctx_defaults() {
    let ctx = Ctx::default();
    assert_eq!(ctx.provider, ProviderKind::Unknown);
    assert!(ctx.start.is_none());
}

#[test]
fn test_provider_detection_with_pipeline() {
    use langspec::pipeline::Pipeline;

    let request = create_test_request("POST", "/v1/chat/completions", Some("api.openai.com"), &[]);
    let pipeline = Pipeline::new();
    let mut ctx = Ctx::default();

    pipeline.on_request(&request, &mut ctx);

    assert_eq!(ctx.provider, ProviderKind::OpenAI);
    assert!(ctx.start.is_some());
}

#[test]
fn test_safe_handling_missing_host() {
    let request = create_test_request("POST", "/v1/chat/completions", None, &[]);
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    // Should not panic and detect OpenAI via path pattern even without host
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);
}

#[test]
fn test_openai_path_patterns() {
    let test_cases = vec![
        ("/v1/chat/completions", true),
        ("/v1/completions", true),
        ("/v1/responses", true),
        ("/v1/models", false),
        ("/api/chat", false),
        ("/v2/chat/completions", false),
    ];

    let registry = ProviderRegistry::new();

    for (path, should_match) in test_cases {
        let request = create_test_request("POST", path, Some("example.com"), &[]);
        let request_view = RequestView::new(&request);
        let result = registry.detect(&request_view);

        if should_match {
            assert_eq!(
                result,
                ProviderKind::OpenAI,
                "Path {} should match OpenAI",
                path
            );
        } else {
            assert_eq!(
                result,
                ProviderKind::Unknown,
                "Path {} should not match OpenAI",
                path
            );
        }
    }
}

#[test]
fn test_bedrock_path_patterns_with_aws_context() {
    let test_cases = vec![
        ("/converse", true),
        ("/invoke", true),
        ("/model/anthropic.claude", true),
        ("/api/status", false),
        ("/health", false),
    ];

    let registry = ProviderRegistry::new();

    for (path, should_match) in test_cases {
        // Test with AWS host context
        let request = create_test_request("POST", path, Some("api.amazonaws.com"), &[]);
        let request_view = RequestView::new(&request);
        let result = registry.detect(&request_view);

        if should_match {
            assert_eq!(
                result,
                ProviderKind::Bedrock,
                "Path {} with AWS host should match Bedrock",
                path
            );
        } else {
            assert_eq!(
                result,
                ProviderKind::Unknown,
                "Path {} with AWS host should not match Bedrock",
                path
            );
        }
    }
}

#[test]
fn test_precedence_openai_host_with_bedrock_path() {
    // OpenAI host should take precedence even with Bedrock-looking path
    let request = create_test_request("POST", "/converse", Some("api.openai.com"), &[]);
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    // Should detect as OpenAI due to host precedence
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);
}

#[test]
fn test_precedence_bedrock_host_with_openai_path() {
    // Bedrock host should win due to High confidence in Chain-of-Responsibility
    let request = create_test_request(
        "POST",
        "/v1/chat/completions",
        Some("bedrock-runtime.us-east-1.amazonaws.com"),
        &[],
    );
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    // Should detect as Bedrock due to High confidence host match (early exit)
    // Host beats path pattern in Chain-of-Responsibility order
    assert_eq!(registry.detect(&request_view), ProviderKind::Bedrock);
}

#[test]
fn test_false_positive_guards() {
    let registry = ProviderRegistry::new();

    // Generic invoke path without AWS context should NOT match (conservative behavior)
    let request = create_test_request("POST", "/invoke", Some("generic-api.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(
        registry.detect(&request_view),
        ProviderKind::Unknown,
        "Generic /invoke without AWS context should not match Bedrock"
    );

    // OpenAI path on non-OpenAI host should still match by path pattern
    let request = create_test_request("POST", "/v1/chat/completions", Some("other-api.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(
        registry.detect(&request_view),
        ProviderKind::OpenAI,
        "OpenAI path pattern should match regardless of host"
    );

    // Bedrock path WITH AWS context should match
    let request = create_test_request("POST", "/invoke", Some("api.amazonaws.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(
        registry.detect(&request_view),
        ProviderKind::Bedrock,
        "Bedrock path with AWS context should match"
    );

    // Non-matching path and host should be Unknown
    let request = create_test_request("GET", "/api/health", Some("example.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::Unknown);
}

#[test]
fn test_explicit_override_header() {
    let registry = ProviderRegistry::new();

    // Override to OpenAI even with Bedrock-looking request
    let request = create_test_request(
        "POST",
        "/converse",
        Some("bedrock-runtime.amazonaws.com"),
        &[("X-Langspec-Provider", "openai")],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);

    // Override to Bedrock even with OpenAI-looking request
    let request = create_test_request(
        "POST",
        "/v1/chat/completions",
        Some("api.openai.com"),
        &[("X-Langspec-Provider", "bedrock")],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::Bedrock);

    // Override to Unknown
    let request = create_test_request(
        "POST",
        "/v1/chat/completions",
        Some("api.openai.com"),
        &[("X-Langspec-Provider", "unknown")],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::Unknown);

    // Invalid override should continue with normal detection
    let request = create_test_request(
        "POST",
        "/v1/chat/completions",
        Some("api.openai.com"),
        &[("X-Langspec-Provider", "invalid")],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);
}

#[test]
fn test_auth_based_detection() {
    let registry = ProviderRegistry::new();

    // AWS SigV4 auth should detect as Bedrock
    let request = create_test_request(
        "POST",
        "/some/api",
        Some("example.com"),
        &[
            ("Authorization", "AWS4-HMAC-SHA256 Credential=..."),
            ("X-Amz-Date", "20231201T120000Z"),
        ],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::Bedrock);

    // Bearer token with OpenAI path should detect as OpenAI
    let request = create_test_request(
        "POST",
        "/v1/chat/completions",
        Some("custom-api.com"),
        &[("Authorization", "Bearer sk-...")],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);

    // Bearer token alone without corroboration should not match OpenAI
    let request = create_test_request(
        "POST",
        "/api/chat",
        Some("generic-api.com"),
        &[("Authorization", "Bearer token123")],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::Unknown);
}

#[test]
fn test_confidence_based_precedence() {
    let registry = ProviderRegistry::new();

    // High confidence should always win over medium confidence
    // OpenAI host (High) should beat any medium confidence Bedrock detection
    let request = create_test_request(
        "POST",
        "/converse", // This could match Bedrock at medium confidence with AWS context
        Some("api.openai.com"), // But OpenAI host is High confidence
        &[("X-Amz-Date", "20231201T120000Z")], // AWS header that could support Bedrock
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);

    // Medium confidence path should beat low confidence header-only
    let request = create_test_request(
        "POST",
        "/v1/chat/completions", // OpenAI path (Medium confidence)
        Some("example.com"),
        &[("OpenAI-Organization", "org-123")], // OpenAI header (Low confidence)
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);
}

#[test]
fn test_enhanced_logging_signals() {
    // This test verifies that the enhanced detection system works
    // but doesn't check logs (would require capturing log output)
    let registry = ProviderRegistry::new();

    // Test host signal (High confidence)
    let request = create_test_request("POST", "/v1/chat", Some("api.openai.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);

    // Test auth signal with corroboration (High confidence)
    let request = create_test_request(
        "POST",
        "/v1/chat/completions",
        Some("custom.com"),
        &[("Authorization", "Bearer sk-123")],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);

    // Test path signal (Medium confidence)
    let request = create_test_request("POST", "/v1/completions", Some("example.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);

    // Test AWS SigV4 auth signal (High confidence)
    let request = create_test_request(
        "POST",
        "/some/api",
        Some("example.com"),
        &[("Authorization", "AWS4-HMAC-SHA256 Credential=...")],
    );
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::Bedrock);
}
