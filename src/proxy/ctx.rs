use crate::provider::ProviderKind;
use std::time::Instant;

#[derive(Debug)]
pub struct Ctx {
    pub provider: ProviderKind,
    pub start: Option<Instant>,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            provider: ProviderKind::Unknown,
            start: None,
        }
    }
}
