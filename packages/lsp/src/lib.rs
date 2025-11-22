// Licensed under either of Apache License, Version 2.0 or MIT license at your option.
// Copyright 2025 RECTOR-LABS

//! LUMOS Language Server Protocol (LSP) implementation
//!
//! This crate provides a complete Language Server Protocol implementation for the LUMOS schema language,
//! enabling rich IDE features across all LSP-compatible editors.
//!
//! # Features
//!
//! - **Real-time diagnostics** - Instant feedback on syntax errors and undefined types
//! - **Auto-completion** - Context-aware suggestions for Solana types, primitives, and attributes
//! - **Hover information** - Type definitions and documentation on hover
//! - **Multi-editor support** - Works with VS Code, Neovim, Emacs, Sublime Text, and more
//!
//! # Usage as a Binary
//!
//! Install and run the LSP server as a standalone binary:
//!
//! ```bash
//! cargo install lumos-lsp
//! lumos-lsp
//! ```
//!
//! # Usage as a Library
//!
//! Embed the LSP server in your own application:
//!
//! ```rust,no_run
//! use lumos_lsp::LumosLanguageServer;
//! use tower_lsp::{LspService, Server};
//!
//! #[tokio::main]
//! async fn main() {
//!     let stdin = tokio::io::stdin();
//!     let stdout = tokio::io::stdout();
//!
//!     let (service, socket) = LspService::new(LumosLanguageServer::new);
//!     Server::new(stdin, stdout, socket).serve(service).await;
//! }
//! ```
//!
//! # Editor Configuration
//!
//! ## VS Code
//!
//! Install the [LUMOS VSCode extension](https://github.com/getlumos/vscode-lumos) which uses this LSP server automatically.
//!
//! ## Neovim
//!
//! Configure with `nvim-lspconfig`:
//!
//! ```lua
//! require'lspconfig'.lumos.setup{
//!   cmd = {"lumos-lsp"},
//!   filetypes = {"lumos"},
//!   root_dir = require'lspconfig'.util.root_pattern(".git", "Cargo.toml"),
//! }
//! ```
//!
//! ## Emacs
//!
//! Configure with `lsp-mode`:
//!
//! ```elisp
//! (add-to-list 'lsp-language-id-configuration '(lumos-mode . "lumos"))
//!
//! (lsp-register-client
//!  (make-lsp-client :new-connection (lsp-stdio-connection "lumos-lsp")
//!                   :major-modes '(lumos-mode)
//!                   :server-id 'lumos-lsp))
//! ```
//!
//! # Architecture
//!
//! The LSP server is built on [`tower-lsp`](https://docs.rs/tower-lsp) and consists of three main handlers:
//!
//! - **Diagnostics** - Validates `.lumos` files and reports errors
//! - **Completion** - Provides intelligent auto-completion
//! - **Hover** - Shows type information and documentation
//!
//! # Links
//!
//! - [GitHub Repository](https://github.com/getlumos/lumos)
//! - [LUMOS Core](https://docs.rs/lumos-core)
//! - [VSCode Extension](https://github.com/getlumos/vscode-lumos)

pub mod server;

// Re-export main types for convenience
pub use server::LumosLanguageServer;
