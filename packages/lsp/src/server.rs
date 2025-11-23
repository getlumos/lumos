// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! Core LSP server implementation

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

mod completion;
mod diagnostics;
mod hover;

use completion::CompletionHandler;
use diagnostics::DiagnosticsHandler;
use hover::HoverHandler;

/// LUMOS Language Server state
#[derive(Debug)]
pub struct LumosLanguageServer {
    /// LSP client for sending notifications/requests
    client: Client,

    /// Document cache (URI -> content)
    documents: DashMap<String, String>,

    /// Diagnostics handler
    diagnostics: DiagnosticsHandler,

    /// Completion handler
    completion: CompletionHandler,

    /// Hover handler
    hover: HoverHandler,
}

impl LumosLanguageServer {
    /// Create a new LUMOS Language Server
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            diagnostics: DiagnosticsHandler::new(),
            completion: CompletionHandler::new(),
            hover: HoverHandler::new(),
        }
    }

    /// Handle document changes (validate and publish diagnostics)
    async fn on_change(&self, uri: Url, text: String) {
        // Cache document content
        self.documents.insert(uri.to_string(), text.clone());

        // Run diagnostics
        let diagnostics = self.diagnostics.analyze(&text);

        // Publish diagnostics to client
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for LumosLanguageServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("Initializing LUMOS Language Server");

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Text document synchronization (full sync)
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),

                // Completion support
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "#".to_string(), // Attributes: #[solana]
                        "[".to_string(), // After #[
                        ":".to_string(), // After field name
                        " ".to_string(), // After keywords
                    ]),
                    ..Default::default()
                }),

                // Hover support
                hover_provider: Some(HoverProviderCapability::Simple(true)),

                // Document formatting
                document_formatting_provider: Some(OneOf::Left(true)),

                // Diagnostic support
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("lumos".to_string()),
                        inter_file_dependencies: false,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),

                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "LUMOS Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("LUMOS Language Server initialized successfully");

        self.client
            .log_message(MessageType::INFO, "LUMOS Language Server is ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down LUMOS Language Server");
        Ok(())
    }

    // ========== Document Synchronization ==========

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        tracing::debug!("Document opened: {}", params.text_document.uri);

        self.on_change(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        tracing::debug!("Document changed: {}", params.text_document.uri);

        if let Some(change) = params.content_changes.into_iter().next() {
            self.on_change(params.text_document.uri, change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        tracing::debug!("Document closed: {}", params.text_document.uri);

        // Remove from cache
        self.documents.remove(&params.text_document.uri.to_string());
    }

    // ========== Language Features ==========

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        tracing::debug!(
            "Completion request at {}:{}",
            params.text_document_position.text_document.uri,
            params.text_document_position.position.line
        );

        // Get document content
        let uri = params.text_document_position.text_document.uri.to_string();
        let document = self.documents.get(&uri);

        if let Some(doc) = document {
            let items = self
                .completion
                .get_completions(&doc, params.text_document_position.position);
            Ok(Some(CompletionResponse::Array(items)))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        tracing::debug!(
            "Hover request at {}:{}",
            params.text_document_position_params.text_document.uri,
            params.text_document_position_params.position.line
        );

        // Get document content
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let document = self.documents.get(&uri);

        if let Some(doc) = document {
            Ok(self
                .hover
                .get_hover(&doc, params.text_document_position_params.position))
        } else {
            Ok(None)
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        tracing::debug!("Formatting request for {}", params.text_document.uri);

        // Get document content
        let uri = params.text_document.uri.to_string();
        let document = self.documents.get(&uri);

        if let Some(_doc) = document {
            // For now, return no edits (formatting will be implemented later)
            // TODO: Implement rustfmt-style formatting for .lumos files
            Ok(Some(vec![]))
        } else {
            Ok(None)
        }
    }
}
