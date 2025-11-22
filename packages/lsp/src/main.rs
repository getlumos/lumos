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
    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting LUMOS Language Server");

    // Create LSP service
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(LumosLanguageServer::new);

    // Start LSP server
    Server::new(stdin, stdout, socket).serve(service).await;

    tracing::info!("LUMOS Language Server shutdown");
}
