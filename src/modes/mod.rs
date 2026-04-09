//! Modes module
//!
//! Contains mode-specific functionality:
//! - Agent: Workspace export for AI coding assistants

pub mod agent;

pub use agent::{AgentError, AgentExporter, ExportResult};
