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
fn test_bedrock_detection_by_path() {
    let request = create_test_request("POST", "/converse", Some("example.com"), &[]);
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
fn test_bedrock_path_patterns() {
    let test_cases = vec![
        ("/converse", true),
        ("/invoke", true),
        ("/model/anthropic.claude", true),
        ("/api/status", false),
        ("/health", false),
    ];

    let registry = ProviderRegistry::new();

    for (path, should_match) in test_cases {
        let request = create_test_request("POST", path, Some("example.com"), &[]);
        let request_view = RequestView::new(&request);
        let result = registry.detect(&request_view);

        if should_match {
            assert_eq!(
                result,
                ProviderKind::Bedrock,
                "Path {} should match Bedrock",
                path
            );
        } else {
            assert_eq!(
                result,
                ProviderKind::Unknown,
                "Path {} should not match Bedrock",
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
    // OpenAI has registry precedence and matches first on path pattern
    let request = create_test_request(
        "POST",
        "/v1/chat/completions",
        Some("bedrock-runtime.us-east-1.amazonaws.com"),
        &[],
    );
    let request_view = RequestView::new(&request);
    let registry = ProviderRegistry::new();

    // Should detect as OpenAI due to registry precedence (first match wins)
    // even though host suggests Bedrock
    assert_eq!(registry.detect(&request_view), ProviderKind::OpenAI);
}

#[test]
fn test_false_positive_guards() {
    let registry = ProviderRegistry::new();

    // Generic invoke path without Bedrock indicators should not match
    let request = create_test_request("POST", "/invoke", Some("generic-api.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(
        registry.detect(&request_view),
        ProviderKind::Bedrock,
        "Generic /invoke should match Bedrock by path pattern"
    );

    // OpenAI path on non-OpenAI host without OpenAI headers should still match by path
    let request = create_test_request("POST", "/v1/chat/completions", Some("other-api.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(
        registry.detect(&request_view),
        ProviderKind::OpenAI,
        "OpenAI path pattern should match regardless of host"
    );

    // Non-matching path and host should be Unknown
    let request = create_test_request("GET", "/api/health", Some("example.com"), &[]);
    let request_view = RequestView::new(&request);
    assert_eq!(registry.detect(&request_view), ProviderKind::Unknown);
}
