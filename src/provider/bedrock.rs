use crate::pipeline::views::RequestView;
use crate::provider::{Provider, ProviderKind};

/// AWS Bedrock provider detection using conservative heuristics.
///
/// Matching Rules (any one match triggers detection):
/// - Host: Contains both `bedrock` and `.amazonaws.com` (e.g., `bedrock-runtime.us-east-1.amazonaws.com`)
/// - Path: Contains `/converse`, `/invoke`, or `/model/` (runtime API endpoints)
///
/// Conservative bias: Prefers false negatives over false positives.
/// Precedence: Second in registry order, defers to OpenAI for ambiguous cases.
pub struct BedrockProvider;

impl Provider for BedrockProvider {
    fn id(&self) -> &'static str {
        "bedrock"
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Bedrock
    }

    fn matches(&self, request_view: &RequestView) -> bool {
        // Conservative heuristics for Bedrock detection

        // Check host for bedrock patterns
        if let Some(host) = request_view.host()
            && host.contains("bedrock")
            && host.contains(".amazonaws.com")
        {
            return true;
        }

        // Check path patterns
        let path = request_view.path();
        if path.contains("/converse") || path.contains("/invoke") || path.contains("/model/") {
            return true;
        }

        false
    }
}
