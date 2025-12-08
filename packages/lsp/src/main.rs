// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! LUMOS Language Server Protocol (LSP) binary
//!
//! This binary runs the LUMOS LSP server, providing IDE features for `.lumos` files
//! across all LSP-compatible editors.

use lumos_lsp::LumosLanguageServer;
use tower_lsp::{LspService, Server};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    // Initialize Sentry error monitoring (optional, enabled via SENTRY_DSN env var)
    let _guard = sentry::init(sentry::ClientOptions {
        dsn: std::env::var("SENTRY_DSN").ok().and_then(|dsn| dsn.parse().ok()),
        release: Some(concat!(env!("CARGO_PKG_NAME"), "@", env!("CARGO_PKG_VERSION")).into()),
        traces_sample_rate: 0.0, // Disable performance monitoring for LSP
        ..Default::default()
    });

    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!(
        "Starting LUMOS Language Server v{} (sentry: {})",
        env!("CARGO_PKG_VERSION"),
        if sentry::Hub::current().client().is_some() {
            "enabled"
        } else {
            "disabled"
        }
    );

    // Create LSP service
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(LumosLanguageServer::new);

    // Start LSP server
    Server::new(stdin, stdout, socket).serve(service).await;

    tracing::info!("LUMOS Language Server shutdown");
}
