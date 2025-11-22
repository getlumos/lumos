# lumos-lsp

Language Server Protocol (LSP) implementation for LUMOS schema language.

## Overview

`lumos-lsp` provides IDE features for `.lumos` files across all LSP-compatible editors:

- **Diagnostics** - Real-time syntax errors and type validation
- **Auto-completion** - Solana types, primitives, attributes, keywords
- **Hover information** - Type definitions and documentation
- **Formatting** - Consistent code style (coming soon)

## Installation

### From crates.io

```bash
cargo install lumos-lsp
```

### From source

```bash
git clone https://github.com/getlumos/lumos.git
cd lumos
cargo install --path packages/lsp
```

## Editor Setup

### VS Code

The [LUMOS VSCode extension](https://github.com/getlumos/vscode-lumos) uses this LSP server automatically. Just install the extension.

### Neovim

Add to your `init.lua`:

```lua
require'lspconfig'.lumos.setup{
  cmd = {"lumos-lsp"},
  filetypes = {"lumos"},
  root_dir = require'lspconfig'.util.root_pattern(".git", "Cargo.toml"),
}
```

### Emacs (lsp-mode)

Add to your Emacs config:

```elisp
(add-to-list 'lsp-language-id-configuration '(lumos-mode . "lumos"))

(lsp-register-client
 (make-lsp-client :new-connection (lsp-stdio-connection "lumos-lsp")
                  :major-modes '(lumos-mode)
                  :server-id 'lumos-lsp))
```

### Sublime Text

1. Install [LSP package](https://packagecontrol.io/packages/LSP)
2. Add to LSP settings:

```json
{
  "clients": {
    "lumos": {
      "enabled": true,
      "command": ["lumos-lsp"],
      "selector": "source.lumos"
    }
  }
}
```

## Features

### Diagnostics

Real-time error detection:

```rust
// ❌ Syntax error
struct Account {
    invalid syntax
}

// ❌ Undefined type
struct Player {
    inventory: UndefinedType,  // Error: Type 'UndefinedType' is not defined
}

// ✅ Valid
#[solana]
struct Account {
    owner: PublicKey,
    balance: u64,
}
```

### Auto-completion

Smart completions for:

- **Solana types**: `PublicKey`, `Signature`, `Keypair`
- **Primitives**: `u8`-`u128`, `i8`-`i128`, `bool`, `String`, `f32`, `f64`
- **Complex types**: `Vec<T>`, `Option<T>`
- **Attributes**: `#[solana]`, `#[account]`, `#[key]`, `#[max(n)]`, `#[deprecated]`
- **Keywords**: `struct`, `enum`

### Hover Information

Hover over types to see documentation:

```rust
owner: PublicKey  // Hover shows: "Solana public key (32 bytes)"
balance: u64      // Hover shows: "64-bit unsigned integer, Range: 0 to 18,446,744,073,709,551,615"
```

## Architecture

```
Editor (VS Code, Neovim, etc.)
    ↕ JSON-RPC (LSP protocol)
LUMOS Language Server
    ↓ Uses
LUMOS Parser (lumos-core)
    ↓ Produces
Abstract Syntax Tree (AST)
```

## Development

### Build

```bash
cargo build --package lumos-lsp
```

### Test

```bash
cargo test --package lumos-lsp
```

### Run locally

```bash
cargo run --package lumos-lsp
```

The server communicates via stdin/stdout using JSON-RPC.

## Logging

Set `RUST_LOG` environment variable to enable logging:

```bash
# Debug level
RUST_LOG=debug lumos-lsp

# Info level
RUST_LOG=info lumos-lsp

# Trace level (very verbose)
RUST_LOG=trace lumos-lsp
```

Logs are written to stderr (not stdout, which is used for LSP communication).

## Project Structure

```
packages/lsp/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs              # Binary entry point
    ├── server.rs            # Core LSP server
    └── server/
        ├── diagnostics.rs   # Diagnostics handler
        ├── completion.rs    # Auto-completion
        └── hover.rs         # Hover information
```

## Contributing

Contributions welcome! See the main [LUMOS repository](https://github.com/getlumos/lumos) for guidelines.

## License

Dual-licensed under MIT or Apache 2.0, at your option.

## Links

- [LUMOS Core](https://github.com/getlumos/lumos)
- [VSCode Extension](https://github.com/getlumos/vscode-lumos)
- [Documentation](https://lumos-lang.org)
- [LSP Specification](https://microsoft.github.io/language-server-protocol/)

---

**Version**: 0.1.0
**Status**: Alpha - Core features implemented, additional features coming soon
