use crate::pipeline::views::RequestView;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProviderKind {
    OpenAI,
    Bedrock,
    #[default]
    Unknown,
}

pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn kind(&self) -> ProviderKind;
    fn matches(&self, request_view: &RequestView) -> bool;
}

pub mod bedrock;
pub mod openai;
pub mod registry;

pub use registry::ProviderRegistry;
