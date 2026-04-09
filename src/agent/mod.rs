//! Agent mode module
//!
//! Provides token counting, workspace generation, and AI agent integrations.

pub mod templates;
pub mod token_counter;

pub use templates::AgentTemplates;
pub use token_counter::TokenCounter;
