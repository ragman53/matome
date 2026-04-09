//! matome - CLI entry point
//!
//! A Rust CLI tool that collects articles from specified document domains,
//! applies automatic Japanese translation, and builds a local integrated web portal.

use clap::Parser;
use std::process::ExitCode;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli;
mod config;
mod pipeline;
mod db;
mod web;
mod modes;  // v0.2.0: Agent mode module

use cli::Cli;

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn main() -> ExitCode {
    // Initialize tracing/logging
    init_tracing();

    // Set up panic hook for better error reporting
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        let location = if let Some(loc) = panic_info.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "unknown location".to_string()
        };

        error!("PANIC at {}: {}", location, payload);
        eprintln!("PANIC at {}: {}", location, payload);
    }));

    // Parse CLI arguments and execute
    let cli = Cli::parse();

    if let Err(e) = cli.run() {
        error!("{}", e);
        eprintln!("Error: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
