use crate::pipeline::views::RequestView;
use crate::provider::{DetectionResult, Provider, ProviderKind};

/// OpenAI API provider detection using Chain-of-Responsibility approach.
///
/// Detection Order (early exit on High confidence):
/// 1. Host match: `api.openai.com` (High confidence)
/// 2. Auth + corroboration: Bearer token + (host OR path) (High confidence)
/// 3. Path patterns: `/v1/(chat|completions|responses)` (Medium confidence)
/// 4. Headers: `OpenAI-Organization` (Low confidence)
///
/// Conservative bias: Prefers false negatives over false positives.
pub struct OpenAIProvider;

impl Provider for OpenAIProvider {
    fn id(&self) -> &'static str {
        "openai"
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAI
    }

    fn detect(&self, request_view: &RequestView) -> Option<DetectionResult> {
        // 1. Explicit override (handled at registry level)

        // 2. Host match (High confidence)
        if let Some(host) = request_view.host()
            && host == "api.openai.com"
        {
            return Some(DetectionResult::high_confidence(
                ProviderKind::OpenAI,
                "api.openai.com exact match",
                "host",
            ));
        }

        // 3. Auth scheme + corroboration (High confidence)
        if request_view.has_bearer_auth() {
            // Bearer alone is not unique - need corroboration
            let has_openai_host = request_view
                .host()
                .map(|h| h == "api.openai.com")
                .unwrap_or(false);

            let has_openai_path = request_view.path().starts_with("/v1/")
                && (request_view.path().contains("/chat")
                    || request_view.path().contains("/completions")
                    || request_view.path().contains("/responses"));

            if has_openai_host || has_openai_path {
                return Some(DetectionResult::high_confidence(
                    ProviderKind::OpenAI,
                    "bearer token with OpenAI context",
                    "auth",
                ));
            }
        }

        // 4. Path namespace (Medium confidence)
        let path = request_view.path();
        if path.starts_with("/v1/")
            && (path.contains("/chat")
                || path.contains("/completions")
                || path.contains("/responses"))
        {
            return Some(DetectionResult::medium_confidence(
                ProviderKind::OpenAI,
                "/v1/ API endpoints",
                "path",
            ));
        }

        // 5. Provider-specific headers (Low confidence)
        if request_view.header("OpenAI-Organization").is_some() {
            return Some(DetectionResult::low_confidence(
                ProviderKind::OpenAI,
                "OpenAI-Organization header present",
                "header",
            ));
        }

        None
    }
}
