use crate::pipeline::views::RequestView;
use crate::provider::bedrock::BedrockProvider;
use crate::provider::openai::OpenAIProvider;
use crate::provider::{DetectionResult, Provider, ProviderKind};
use log::info;

pub struct ProviderRegistry {
    providers: &'static [&'static dyn Provider],
}

impl ProviderRegistry {
    pub fn new() -> Self {
        // Chain-of-Responsibility order: Override > Host > Auth > Path > Headers
        // Each provider implements this chain internally
        static PROVIDERS: &[&dyn Provider] = &[&OpenAIProvider, &BedrockProvider];

        Self {
            providers: PROVIDERS,
        }
    }

    pub fn detect(&self, request_view: &RequestView) -> ProviderKind {
        // 1. Explicit override (highest confidence)
        if let Some(override_provider) = request_view.header("x-langspec-provider") {
            match override_provider.to_lowercase().as_str() {
                "openai" => {
                    info!("Provider override: OpenAI (X-Langspec-Provider header)");
                    return ProviderKind::OpenAI;
                }
                "bedrock" => {
                    info!("Provider override: Bedrock (X-Langspec-Provider header)");
                    return ProviderKind::Bedrock;
                }
                "unknown" => {
                    info!("Provider override: Unknown (X-Langspec-Provider header)");
                    return ProviderKind::Unknown;
                }
                _ => {
                    info!(
                        "Invalid provider override '{}', continuing with detection",
                        override_provider
                    );
                }
            }
        }

        // 2. Chain-of-Responsibility detection with confidence-based accumulation
        let mut all_results: Vec<DetectionResult> = Vec::new();
        let mut best_result: Option<DetectionResult> = None;

        for provider in self.providers {
            if let Some(result) = provider.detect(request_view) {
                // Log all detections for observability
                info!(
                    "Provider candidate: {} detected {:?} (confidence: {:?}, signal: {}, reason: {})",
                    provider.id(),
                    result.kind,
                    result.confidence,
                    result.signal,
                    result.reason
                );

                // Early exit on High confidence (decisive)
                if result.is_decisive() {
                    info!(
                        "Decisive detection: {:?} via {} ({})",
                        result.kind, result.signal, result.reason
                    );
                    return result.kind;
                }

                // Accumulate for conflict detection
                all_results.push(result.clone());

                // Track best result for fallback
                match &best_result {
                    None => best_result = Some(result),
                    Some(current_best) => {
                        if result.is_better_than(current_best) {
                            best_result = Some(result);
                        }
                    }
                }
            }
        }

        // Detect and log conflicts between providers
        if all_results.len() > 1 {
            let mut conflicts: Vec<(&DetectionResult, &DetectionResult)> = Vec::new();
            for (i, result1) in all_results.iter().enumerate() {
                for result2 in all_results.iter().skip(i + 1) {
                    if result1.kind != result2.kind {
                        conflicts.push((result1, result2));
                    }
                }
            }

            if !conflicts.is_empty() {
                info!("Provider conflicts detected:");
                for (r1, r2) in conflicts {
                    info!(
                        "  Conflict: {:?} ({:?} via {}) vs {:?} ({:?} via {})",
                        r1.kind, r1.confidence, r1.signal, r2.kind, r2.confidence, r2.signal
                    );
                }
            }
        }

        // Return best result found, or Unknown
        match best_result {
            Some(result) => {
                info!(
                    "Final detection: {:?} (confidence: {:?}, signal: {}, reason: {})",
                    result.kind, result.confidence, result.signal, result.reason
                );
                result.kind
            }
            None => {
                info!("No provider detected, defaulting to Unknown");
                ProviderKind::Unknown
            }
        }
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
