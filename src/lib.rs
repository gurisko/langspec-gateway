pub mod pipeline;
pub mod provider;
pub mod proxy;

// Stable public API re-exports
pub use pipeline::views::RequestView;
pub use provider::{ProviderKind, ProviderRegistry};
pub use proxy::GatewayProxy;
