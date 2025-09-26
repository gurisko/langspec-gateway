use crate::pipeline::views::RequestView;
use crate::provider::{DetectionResult, Provider, ProviderKind};

/// AWS Bedrock provider detection using Chain-of-Responsibility approach.
///
/// Detection Order (early exit on High confidence):
/// 1. Host match: `*.bedrock*.amazonaws.com` (High confidence)
/// 2. Auth scheme: AWS SigV4 (High confidence)
/// 3. Path + AWS hints: `/converse|invoke|model/` + (AWS host OR SigV4) (Medium confidence)
/// 4. AWS headers: `x-amz-*` headers (Low confidence, requires corroboration)
///
/// Conservative bias: Requires AWS indicators to prevent false positives on generic paths.
pub struct BedrockProvider;

impl Provider for BedrockProvider {
    fn id(&self) -> &'static str {
        "bedrock"
    }

    fn kind(&self) -> ProviderKind {
        ProviderKind::Bedrock
    }

    fn detect(&self, request_view: &RequestView) -> Option<DetectionResult> {
        // 1. Explicit override (handled at registry level)

        // 2. Host match (High confidence)
        if let Some(host) = request_view.host()
            && host.contains("bedrock")
            && host.ends_with(".amazonaws.com")
        {
            return Some(DetectionResult::high_confidence(
                ProviderKind::Bedrock,
                "bedrock.amazonaws.com host",
                "host",
            ));
        }

        // 3. Auth scheme (High confidence)
        if request_view.has_aws_sigv4() {
            return Some(DetectionResult::high_confidence(
                ProviderKind::Bedrock,
                "AWS Signature Version 4",
                "auth",
            ));
        }

        // Check for AWS context for path-based detection
        let has_aws_host = request_view.host_ends_with(".amazonaws.com");
        let has_aws_headers = request_view.header("x-amz-date").is_some()
            || request_view.header("x-amzn-trace-id").is_some()
            || request_view.header("x-amz-security-token").is_some();

        // 4. Path + AWS hints (Medium confidence)
        let path = request_view.path();
        if (path.contains("/converse") || path.contains("/invoke") || path.contains("/model/"))
            && (has_aws_host || has_aws_headers)
        {
            let reason = if has_aws_host && has_aws_headers {
                "Bedrock paths with AWS host + headers"
            } else if has_aws_host {
                "Bedrock paths with AWS host"
            } else {
                "Bedrock paths with AWS headers"
            };

            return Some(DetectionResult::medium_confidence(
                ProviderKind::Bedrock,
                reason,
                "path",
            ));
        }

        // 5. AWS headers alone (Low confidence, requires multiple indicators)
        if has_aws_headers && has_aws_host {
            return Some(DetectionResult::low_confidence(
                ProviderKind::Bedrock,
                "AWS headers with AWS host",
                "header",
            ));
        }

        None
    }
}
