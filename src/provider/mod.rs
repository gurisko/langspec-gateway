use crate::pipeline::views::RequestView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProviderKind {
    OpenAI,
    Bedrock,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub kind: ProviderKind,
    pub confidence: Confidence,
    pub reason: &'static str,
    pub signal: &'static str, // What triggered the detection (host, auth, path, header)
}

impl DetectionResult {
    pub fn high_confidence(kind: ProviderKind, reason: &'static str, signal: &'static str) -> Self {
        Self {
            kind,
            confidence: Confidence::High,
            reason,
            signal,
        }
    }

    pub fn medium_confidence(
        kind: ProviderKind,
        reason: &'static str,
        signal: &'static str,
    ) -> Self {
        Self {
            kind,
            confidence: Confidence::Medium,
            reason,
            signal,
        }
    }

    pub fn low_confidence(kind: ProviderKind, reason: &'static str, signal: &'static str) -> Self {
        Self {
            kind,
            confidence: Confidence::Low,
            reason,
            signal,
        }
    }

    pub fn no_match() -> Option<Self> {
        None
    }

    /// Returns true if this result should trigger early exit in CoR
    pub fn is_decisive(&self) -> bool {
        self.confidence == Confidence::High
    }

    /// Returns true if this result is better than another for accumulation
    pub fn is_better_than(&self, other: &DetectionResult) -> bool {
        // Prefer higher confidence, then provider-specific tiebreaker logic could be added here
        self.confidence > other.confidence
    }
}

pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn kind(&self) -> ProviderKind;
    fn detect(&self, request_view: &RequestView) -> Option<DetectionResult>;
}

pub mod bedrock;
pub mod openai;
pub mod registry;

pub use registry::ProviderRegistry;
