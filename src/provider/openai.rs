use crate::pipeline::views::RequestView;
use crate::provider::{Provider, ProviderKind};

/// OpenAI API provider detection using conservative heuristics.
///
/// Matching Rules (any one match triggers detection):
/// - Host: `api.openai.com` (exact match)
/// - Header: `OpenAI-Organization` header present (any value)
/// - Path: `/v1/` prefix with `/chat`, `/completions`, or `/responses` in path
///
/// Conservative bias: Prefers false negatives over false positives.
/// Precedence: First in registry order, takes priority over Bedrock for ambiguous cases.
pub struct OpenAIProvider;

impl Provider for OpenAIProvider {
    fn id(&self) -> &'static str {
        "openai"
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAI
    }

    fn matches(&self, request_view: &RequestView) -> bool {
        // Conservative heuristics for OpenAI detection

        // Check host
        if let Some(host) = request_view.host()
            && host == "api.openai.com"
        {
            return true;
        }

        // Check for OpenAI-Organization header
        if request_view.header("OpenAI-Organization").is_some() {
            return true;
        }

        // Check path patterns
        let path = request_view.path();
        if path.starts_with("/v1/")
            && (path.contains("/chat")
                || path.contains("/completions")
                || path.contains("/responses"))
        {
            return true;
        }

        false
    }
}
