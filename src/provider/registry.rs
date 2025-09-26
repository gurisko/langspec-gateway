use crate::pipeline::views::RequestView;
use crate::provider::bedrock::BedrockProvider;
use crate::provider::openai::OpenAIProvider;
use crate::provider::{Provider, ProviderKind};

pub struct ProviderRegistry {
    providers: &'static [&'static dyn Provider],
}

impl ProviderRegistry {
    pub fn new() -> Self {
        // Provider precedence: First match wins. Order is important for overlapping patterns.
        // OpenAI has priority over Bedrock for ambiguous cases.
        static PROVIDERS: &[&dyn Provider] = &[&OpenAIProvider, &BedrockProvider];

        Self {
            providers: PROVIDERS,
        }
    }

    pub fn detect(&self, request_view: &RequestView) -> ProviderKind {
        for provider in self.providers {
            if provider.matches(request_view) {
                return provider.kind();
            }
        }
        ProviderKind::Unknown
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
