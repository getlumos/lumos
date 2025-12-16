# LUMOS Press Kit

*Last Updated: December 2025*

---

## Quick Facts

| | |
|---|---|
| **Name** | LUMOS |
| **Tagline** | The type-safe schema language for Solana |
| **Category** | Developer Tools / Code Generation |
| **License** | MIT + Apache 2.0 (dual-licensed) |
| **First Release** | November 2025 |
| **Current Version** | v0.2.0 |
| **Website** | [lumos-lang.org](https://lumos-lang.org) |
| **GitHub** | [github.com/getlumos/lumos](https://github.com/getlumos/lumos) |
| **Organization** | getlumos |

---

## One-Line Description

LUMOS is a schema language that generates synchronized Rust, TypeScript, Python, Go, and Ruby code from a single source file for Solana blockchain development.

---

## Elevator Pitch (50 words)

LUMOS eliminates type synchronization bugs in Solana development. Define your data structures once in `.lumos` syntax, then generate production-ready code for Rust, TypeScript, and three other languages. Full Anchor integration, IDE support, and security tools included. Open source and production-ready.

---

## The Problem

Full-stack Solana applications require identical type definitions in Rust (on-chain programs) and TypeScript (frontend clients). Manual synchronization is:

- **Error-prone**: Adding a field to Rust but forgetting TypeScript causes runtime failures
- **Time-consuming**: Every schema change requires updates in multiple files
- **Risky**: Borsh serialization requires exact field order—mismatches corrupt data

---

## The Solution

LUMOS provides a domain-specific language (DSL) for defining schemas once:

```rust
#[solana]
#[account]
struct UserAccount {
    wallet: PublicKey,
    balance: u64,
}
```

One command generates code for all target languages:

```bash
lumos generate schema.lumos --lang rust,typescript,python
```

---

## Key Features

### Multi-Language Generation
- Rust (with Anchor integration)
- TypeScript (with Borsh schemas)
- Python (with borsh-construct)
- Go (with struct tags)
- Ruby (with borsh-rb)

### IDE Support
Full Language Server Protocol (LSP) implementation:
- VS Code (dedicated extension)
- IntelliJ IDEA / Rust Rover
- Neovim (with Tree-sitter)
- Emacs
- Sublime Text

### Anchor Integration
- Detects `#[account]` structs automatically
- Generates correct imports and derives
- Complete Anchor program generation via `lumos anchor generate`

### Schema Evolution
- `lumos diff` — Visual schema comparison
- `lumos check-compat` — Breaking change detection
- `lumos migrate` — Migration code generation

### Security Tools
- Static vulnerability analysis
- Fuzz target generation
- Audit checklist generation

---

## Technical Stats

| Metric | Value |
|--------|-------|
| Test Count | 322 |
| Test Pass Rate | 100% |
| Languages Supported | 5 |
| IDE Integrations | 6 |
| Known Vulnerabilities | 0 |
| Generation Time | <100ms (typical) |

---

## Installation Options

```bash
# Rust developers
cargo install lumos-cli

# JavaScript/TypeScript developers
npm install -g @getlumos/cli

# Docker
docker pull ghcr.io/getlumos/lumos

# GitHub Action
uses: getlumos/lumos-action@v1
```

---

## Published Packages

| Package | Registry | Link |
|---------|----------|------|
| lumos-core | crates.io | [crates.io/crates/lumos-core](https://crates.io/crates/lumos-core) |
| lumos-cli | crates.io | [crates.io/crates/lumos-cli](https://crates.io/crates/lumos-cli) |
| lumos-lsp | crates.io | [crates.io/crates/lumos-lsp](https://crates.io/crates/lumos-lsp) |
| @getlumos/cli | npm | [npmjs.com/package/@getlumos/cli](https://www.npmjs.com/package/@getlumos/cli) |
| lumos | Docker | [ghcr.io/getlumos/lumos](https://github.com/getlumos/lumos/pkgs/container/lumos) |

---

## Repositories

| Repository | Purpose |
|------------|---------|
| [getlumos/lumos](https://github.com/getlumos/lumos) | Core compiler, CLI, LSP |
| [getlumos/vscode-lumos](https://github.com/getlumos/vscode-lumos) | VS Code extension |
| [getlumos/intellij-lumos](https://github.com/getlumos/intellij-lumos) | IntelliJ plugin |
| [getlumos/nvim-lumos](https://github.com/getlumos/nvim-lumos) | Neovim plugin |
| [getlumos/lumos-mode](https://github.com/getlumos/lumos-mode) | Emacs mode |
| [getlumos/sublime-lumos](https://github.com/getlumos/sublime-lumos) | Sublime Text package |
| [getlumos/tree-sitter-lumos](https://github.com/getlumos/tree-sitter-lumos) | Tree-sitter grammar |
| [getlumos/lumos-action](https://github.com/getlumos/lumos-action) | GitHub Action |
| [getlumos/awesome-lumos](https://github.com/getlumos/awesome-lumos) | Example projects |
| [getlumos/docs-lumos](https://github.com/getlumos/docs-lumos) | Documentation site |

---

## Brand Assets

### Logo Files
Available in `assets/logo/`:
- `logo.png` — Full size
- `logo-512.png` — 512x512
- `logo-256.png` — 256x256
- `logo-128.png` — 128x128
- `logo-64.png` — 64x64
- `logo-32.png` — 32x32

### Colors
- Primary: Purple (#8B5CF6)
- Secondary: Gold (#F59E0B)
- Background: Dark (#1F2937)

### Usage Guidelines
- Use logo on dark or light backgrounds
- Maintain aspect ratio
- Minimum clear space: 20% of logo width
- Do not modify colors or add effects

---

## Contact

- **GitHub Issues**: [github.com/getlumos/lumos/issues](https://github.com/getlumos/lumos/issues)
- **Twitter/X**: [@getlumos](https://twitter.com/getlumos)
- **Discord**: [Join server]
- **Email**: [contact email]

---

## Boilerplate

**Short (25 words):**
LUMOS is a schema language for Solana that generates synchronized Rust and TypeScript code, eliminating type drift in full-stack blockchain applications.

**Medium (50 words):**
LUMOS is an open-source schema language designed for Solana development. Define your data structures once, then generate production-ready code for Rust, TypeScript, Python, Go, and Ruby. With full Anchor integration, IDE support, and security tools, LUMOS eliminates type synchronization bugs and accelerates development.

**Long (100 words):**
LUMOS is an open-source schema language and code generator purpose-built for Solana blockchain development. It solves the critical problem of maintaining identical type definitions across Rust on-chain programs and TypeScript frontend clients. Developers define schemas once using a Rust-like DSL, then generate synchronized, production-ready code for five languages: Rust (with Anchor integration), TypeScript (with Borsh schemas), Python, Go, and Ruby. LUMOS includes a full Language Server Protocol implementation for IDE integration, schema versioning tools, and security analysis features. Published to crates.io and npm, LUMOS is production-ready and actively maintained by the getlumos organization.

---

## FAQ

**Q: Is LUMOS free?**
A: Yes. LUMOS is open source under MIT and Apache 2.0 licenses.

**Q: Does LUMOS work without Solana?**
A: LUMOS is designed for Solana but the generated Borsh code works anywhere Borsh serialization is used.

**Q: How does LUMOS compare to Anchor IDL?**
A: Anchor IDL is generated from Rust code. LUMOS works in the opposite direction—you write schemas first, then generate Rust (and other languages). They're complementary tools.

**Q: Can I use LUMOS in production?**
A: Yes. LUMOS has 322 tests, zero known vulnerabilities, and is published to crates.io.

---

*For press inquiries or additional information, please open a GitHub issue or contact us via Discord.*
