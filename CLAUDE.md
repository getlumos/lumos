# CLAUDE.md - LUMOS Ecosystem

**Organization:** https://github.com/getlumos
**Website:** https://lumos-lang.org
**Purpose:** This file contains ecosystem-wide context for AI assistants working across all LUMOS repositories

---

## ECOSYSTEM OVERVIEW

**LUMOS** is a type-safe schema language for Solana development that bridges TypeScript ↔ Rust with guaranteed Borsh serialization compatibility.

**Write once in `.lumos`** → Generate production-ready Rust + TypeScript

### Related Repositories

| Repo | Purpose | Tech Stack | Version |
|------|---------|------------|---------|
| `getlumos/lumos` | **Core** - Schema compiler + CLI + LSP server | Rust | v0.1.1 |
| `getlumos/vscode-lumos` | VSCode extension (syntax, IntelliSense, commands) | TypeScript | v0.5.0 |
| `getlumos/intellij-lumos` | IntelliJ IDEA / Rust Rover plugin via LSP | Kotlin | v0.1.0 |
| `getlumos/nvim-lumos` | Neovim plugin + Tree-sitter + LSP | Lua | v0.1.0 |
| `getlumos/tree-sitter-lumos` | Tree-sitter grammar (used by nvim-lumos) | JavaScript | v0.1.0 |
| `getlumos/lumos-mode` | Emacs major mode + LSP | Emacs Lisp | v0.1.0 |
| `getlumos/sublime-lumos` | Sublime Text package + LSP + snippets | YAML | v0.1.0 |
| `getlumos/awesome-lumos` | 5 production examples (NFT, DeFi, DAO, Gaming, Vesting) | Anchor, TS | - |
| `getlumos/docs-lumos` | Official docs & website (VitePress) | Vue, MD | - |
| `getlumos/lumos-action` | GitHub Action for CI/CD validation & generation | Bash | v1.0.0 |
| `@getlumos/cli` | npm package - WASM CLI for JS/TS devs (no Rust required) | WASM | v0.1.0 |

**Future:** cargo subcommand integration

**Organization Mission:** Become the standard schema language for type-safe Solana development

---

## CROSS-REPO STANDARDS

These standards apply to ALL repositories under getlumos organization.

### Shared Coding Standards

**Formatting:**
- Rust: `cargo fmt --all` (rustfmt)
- TypeScript/JavaScript: 2-space indentation
- Markdown: Consistent headers, code blocks with language tags

**Quality Gates:**
- Zero warnings in CI/CD
- All tests passing before merge
- No security vulnerabilities (cargo audit, npm audit)

**Git Workflow:**
- Feature branches from `dev` or `main`
- Descriptive commit messages (no AI attribution)
- Squash commits on PR merge when sensible

**Documentation:**
- Keep README.md synchronized with code
- Update CHANGELOG.md for user-facing changes
- Maintain repo-specific CLAUDE.md (references this file)

### Shared AI Assistant Guidelines

**✅ ALWAYS DO:**
- Read repo-specific CLAUDE.md for unique details
- Run tests after code changes
- Use proper formatting tools before committing
- Reference file:line when discussing code
- Update documentation when changing behavior
- Check for redundant or duplicate information

**❌ NEVER DO:**
- Include AI attribution in commits, docs, or code
- Skip running tests to save time
- Create files without checking existing structure
- Use bash echo for communication (output directly)
- Proceed with ambiguous or unclear instructions

### Licenses

All LUMOS projects are dual-licensed:
- **MIT License** - Permissive for commercial use
- **Apache License 2.0** - Patent protection

---

## REPOSITORY INDEX

Quick reference for navigating between LUMOS repositories.

### 1. lumos (Core) - **YOU ARE HERE**

**Purpose:** Schema language compiler and CLI
**Tech Stack:** Rust (syn, proc-macro2, borsh)
**Key Commands:**
```bash
cargo test --all-features --workspace    # Run 129 tests
cargo clippy -- -D warnings              # Lint
lumos generate schema.lumos              # Generate code
```
**Maintainers:** Core team
**Full details:** See "LUMOS CORE REPOSITORY" section below

---

### 2. vscode-lumos

**Purpose:** VSCode extension for `.lumos` files
**Tech Stack:** TypeScript, VSCode API, TextMate grammar
**Key Commands:**
```bash
npm run compile      # Build extension
npm run package      # Create .vsix
vsce publish         # Publish to marketplace
```
**Key Files:**
- `src/extension.ts` - Extension activation
- `syntaxes/lumos.tmLanguage.json` - Grammar (26 rules)
- `snippets/lumos.json` - 13 snippets

**Features:** Syntax highlighting, IntelliSense, diagnostics, quick fixes, format-on-save
**CLAUDE.md:** [vscode-lumos/CLAUDE.md](https://github.com/getlumos/vscode-lumos/blob/main/CLAUDE.md)

---

### 3. intellij-lumos

**Purpose:** IntelliJ IDEA and Rust Rover plugin for `.lumos` files
**Tech Stack:** Kotlin, IntelliJ Platform SDK, LSP client
**Key Commands:**
```bash
./gradlew buildPlugin    # Build plugin
./gradlew runIde        # Run in IDE sandbox
./gradlew publishPlugin  # Publish to marketplace
```
**Key Files:**
- `src/main/kotlin/com/lumos/LumosLspServerDescriptor.kt` - LSP integration
- `src/main/kotlin/com/lumos/LumosFileType.kt` - File type registration
- `src/main/resources/META-INF/plugin.xml` - Plugin manifest
- `build.gradle.kts` - Gradle build configuration

**Features:** File type recognition, LSP client for lumos-lsp server, syntax highlighting, auto-completion, diagnostics
**Target IDEs:** IntelliJ IDEA, Rust Rover, CLion (2024.1+)
**Dependencies:** lumos-lsp v0.1.1+, lsp4ij plugin
**CLAUDE.md:** [intellij-lumos/CLAUDE.md](https://github.com/getlumos/intellij-lumos/blob/main/CLAUDE.md)

---

### 4. tree-sitter-lumos

**Purpose:** Tree-sitter grammar for LUMOS syntax highlighting
**Tech Stack:** JavaScript, Tree-sitter, npm
**Key Commands:**
```bash
npm install              # Install dependencies
npm test                # Run grammar tests
tree-sitter generate    # Generate parser from grammar.js
tree-sitter parse file.lumos  # Test parsing
```
**Key Files:**
- `grammar.js` - Tree-sitter grammar definition (120 lines)
- `queries/highlights.scm` - Syntax highlighting queries
- `test/corpus/struct.txt` - Struct test cases
- `test/corpus/enum.txt` - Enum test cases

**Grammar Coverage:**
- Structs with fields and attributes
- Enums (unit, tuple, struct variants)
- All LUMOS types (primitives, Solana types, Vec, Option, arrays)
- Attributes (`#[solana]`, `#[account]`, `#[deprecated]`)
- Comments (line and block)

**Test Results:** 6/6 tests passing
**Repository:** https://github.com/getlumos/tree-sitter-lumos

---

### 5. nvim-lumos

**Purpose:** Neovim plugin for LUMOS with LSP and Tree-sitter support
**Tech Stack:** Lua, Neovim API, nvim-lspconfig, nvim-treesitter
**Installation:**
```lua
-- Using lazy.nvim
{
  "getlumos/nvim-lumos",
  dependencies = {
    "nvim-treesitter/nvim-treesitter",
    "neovim/nvim-lspconfig",
  },
  config = function()
    require("lumos").setup()
  end,
}
```
**Key Files:**
- `ftdetect/lumos.lua` - File type detection
- `lua/lumos/init.lua` - Plugin entry point
- `lua/lumos/lsp.lua` - LSP configuration with keybindings
- `queries/lumos/highlights.scm` - Syntax highlighting queries

**Features:**
- Automatic file type detection for `.lumos` files
- Full LSP integration with lumos-lsp server
- Tree-sitter syntax highlighting via tree-sitter-lumos
- Pre-configured keybindings:
  - `gd` - Go to definition
  - `K` - Hover documentation
  - `gr` - Find references
  - `<leader>rn` - Rename symbol
  - `<leader>ca` - Code actions
  - `<leader>f` - Format document

**Dependencies:** Neovim 0.9+, lumos-lsp, nvim-treesitter, nvim-lspconfig
**Repository:** https://github.com/getlumos/nvim-lumos

---

### 6. lumos-mode

**Purpose:** Emacs major mode for LUMOS with syntax highlighting and LSP integration
**Tech Stack:** Emacs Lisp, lsp-mode
**Installation:**
```elisp
;; Using MELPA
(use-package lumos-mode
  :ensure t
  :hook (lumos-mode . lsp-deferred))

;; Using straight.el
(use-package lumos-mode
  :straight (lumos-mode :type git :host github :repo "getlumos/lumos-mode")
  :hook (lumos-mode . lsp-deferred))
```
**Key Files:**
- `lumos-mode.el` - Main mode implementation (350 lines)
- `lumos-mode-test.el` - Test suite (14 tests)
- `.github/workflows/ci.yml` - CI testing across Emacs 27.2, 28.2, 29.1, snapshot
- `Makefile` - Test and compile commands
- `lumos-mode-recipe.el` - MELPA recipe file

**Features:**
- Syntax highlighting (keywords, types, attributes, comments, fields)
- Smart indentation with configurable offset
- LSP integration via lsp-mode and lumos-lsp server
- Auto-completion, diagnostics, hover, go-to-definition
- Comment support (line and block)
- File type auto-detection for `.lumos` files
- Customizable variables (indent-offset, lsp-server-command)

**Test Coverage:** 14 tests (mode loading, file association, syntax highlighting, indentation, comments, custom variables)
**Dependencies:** Emacs 26.1+, lsp-mode, lumos-lsp server
**Repository:** https://github.com/getlumos/lumos-mode
**CLAUDE.md:** [lumos-mode/CLAUDE.md](https://github.com/getlumos/lumos-mode/blob/main/CLAUDE.md)

---

### 7. sublime-lumos

**Purpose:** Sublime Text package for `.lumos` files
**Tech Stack:** YAML syntax definitions, Sublime Text API, LSP integration
**Installation:**
```bash
# Manual installation
cd ~/Library/Application\ Support/Sublime\ Text/Packages  # macOS
# or ~/.config/sublime-text/Packages  # Linux
# or %APPDATA%\Sublime Text\Packages  # Windows
git clone https://github.com/getlumos/sublime-lumos.git LUMOS
```
**Key Files:**
- `LUMOS.sublime-syntax` - Syntax definition (YAML-based, ~130 lines)
- `LUMOS.sublime-settings` - Package settings
- `LSP-lumos.sublime-settings` - LSP client configuration
- `snippets/*.sublime-snippet` - 6 snippets for common patterns

**Features:**
- Syntax highlighting (keywords, types, attributes, comments, numbers)
- LSP integration via LSP package and lumos-lsp server
- 6 snippets (struct, enum variants, account, deprecated)
- Auto-indentation (2 spaces, smart indent)
- Comment toggling (line and block comments)
- Bracket matching and auto-pairing

**Dependencies:** Sublime Text 4 (or 3 build 3103+), LSP package (optional), lumos-lsp server
**Repository:** https://github.com/getlumos/sublime-lumos
**Package Control PR:** https://github.com/wbond/package_control_channel/pull/9251
**CLAUDE.md:** [sublime-lumos/CLAUDE.md](https://github.com/getlumos/sublime-lumos/blob/main/CLAUDE.md)

---

### 8. awesome-lumos

**Purpose:** Production-ready examples and community projects
**Tech Stack:** Anchor, TypeScript, Solana web3.js
**Examples:** 5 complete (NFT Marketplace, DeFi Staking, DAO Governance, Gaming Inventory, Token Vesting)
**Structure:**
```
examples/[project-name]/
├── schema.lumos          # Source of truth
├── generated.rs/.ts      # Generated code
├── programs/             # Anchor program
└── client/               # TypeScript client
```
**Total:** 53 types, 42 instructions, 4000+ LOC
**CLAUDE.md:** [awesome-lumos/CLAUDE.md](https://github.com/getlumos/awesome-lumos/blob/main/CLAUDE.md)

---

### 9. docs-lumos

**Purpose:** Official documentation website
**Tech Stack:** VitePress, Markdown, Vue
**Deployment:** Cloudflare Pages → lumos-lang.org
**Key Commands:**
```bash
npm run docs:dev     # Dev server (localhost:5173)
npm run docs:build   # Build for production
```
**Content Sections:** guide/, reference/, examples/, api/
**Auto-deploy:** Push to `main` → live
**CLAUDE.md:** [docs-lumos/CLAUDE.md](https://github.com/getlumos/docs-lumos/blob/main/CLAUDE.md)

---

### 10. lumos-action

**Purpose:** GitHub Action for automated schema validation and code generation
**Tech Stack:** Composite Action (Bash, GitHub Actions)
**Marketplace:** https://github.com/marketplace/actions/lumos-generate
**Repository:** https://github.com/getlumos/lumos-action
**Usage:**
```yaml
- uses: getlumos/lumos-action@v1
  with:
    schema: 'schemas/**/*.lumos'
```
**Features:** Auto-install CLI, validate schemas, generate code, drift detection, PR comments
**Docs:** [lumos-action/README.md](https://github.com/getlumos/lumos-action#readme)
**CLAUDE.md:** [lumos-action/CLAUDE.md](https://github.com/getlumos/lumos-action/blob/main/CLAUDE.md)

---

## CURRENT FOCUS

See [ROADMAP.md](ROADMAP.md) for detailed phase tracking and priorities.

**Cross-Repo Coordination:**
- Changes to `lumos-core` may require updates to editor extensions
- All repos follow semantic versioning
- Check repo-specific CLAUDE.md files for individual development notes

---

# LUMOS CORE REPOSITORY

> **Note:** The sections below are specific to the `getlumos/lumos` repository (core compiler and CLI). For other repos, see their respective CLAUDE.md files linked above.

---

## What is LUMOS?

Write data structures once in `.lumos` syntax → Generate production-ready Rust + TypeScript with guaranteed Borsh serialization compatibility.

**Status:** v0.1.1 published to crates.io | 142/142 tests passing | 0 warnings | 0 vulnerabilities

---

## Architecture

```
.lumos → Parser (syn) → AST → Transform → IR → Generators → .rs + .ts
```

**Key Files:**
- `packages/core/src/parser.rs` - Parse .lumos syntax to AST
- `packages/core/src/transform.rs` - AST → IR conversion
- `packages/core/src/generators/rust.rs` - Rust code generation (context-aware)
- `packages/core/src/generators/typescript.rs` - TypeScript + Borsh schemas
- `packages/cli/src/main.rs` - CLI commands (generate, validate, init, check, diff)

---

## Current Features

| Feature | Status | Notes |
|---------|--------|-------|
| Struct support | ✅ | Full support with #[account] |
| Enum support | ✅ | Unit, Tuple, Struct variants |
| Primitive types | ✅ | u8-u128, i8-i128, bool, String |
| Solana types | ✅ | PublicKey, Signature |
| Complex types | ✅ | Vec, Option |
| Context-aware generation | ✅ | Anchor vs pure Borsh detection |
| CLI tool | ✅ | 5 commands (generate, validate, init, check, diff) |
| VSCode extension | ✅ | Separate repo: getlumos/vscode-lumos |
| User-defined type validation | ✅ | Validates type references during transformation (v0.1.1) |
| Path traversal protection | ✅ | CLI validates output paths for security (v0.1.1) |
| u64 precision warnings | ✅ | JSDoc comments in generated TypeScript (v0.1.1) |
| Deprecation warnings | ✅ | #[deprecated] attribute for schema evolution (v0.1.2) |
| Security analysis | ✅ | Static analysis for vulnerabilities (v0.1.2) |
| Fuzzing support | ✅ | Generate fuzzing harnesses for code (v0.1.2) |
| Language Server (LSP) | ✅ | Diagnostics, completion, hover for all editors (v0.2.0) |
| Multi-language generation | ✅ | Rust, TypeScript, Python, Go, Ruby via --lang flag |

---

## Development Commands

```bash
# Run tests (322 tests, ~180s with E2E)
cargo test --all-features --workspace

# Check formatting
cargo fmt --all -- --check

# Lint (strict mode)
cargo clippy --all-targets --all-features -- -D warnings

# Build release
cargo build --release --all-features --workspace

# Generate from schema
cargo run --bin lumos -- generate examples/gaming/schema.lumos

# Run LSP server (for editor integration)
cargo run --bin lumos-lsp

# Test LSP package
cargo test --package lumos-lsp
```

---

## Test Suite (322 tests)

| Suite | Count | Location |
|-------|-------|----------|
| Unit tests | 211 | `packages/core/src/**/*.rs` |
| Parser integration | 5 | `packages/core/tests/integration_test.rs` |
| Error path tests | 30 | `packages/core/tests/test_error_paths.rs` |
| Rust generator | 8 | `packages/core/tests/test_rust_generator.rs` |
| TypeScript generator | 7 | `packages/core/tests/test_typescript_generator.rs` |
| Python generator | 11 | `packages/core/src/generators/python.rs` |
| Go generator | 11 | `packages/core/src/generators/go.rs` |
| Ruby generator | 11 | `packages/core/src/generators/ruby.rs` |
| LSP tests | 13 | `packages/lsp/src/server/**/*.rs` (diagnostics, completion, hover) |
| E2E compilation | 11 | `packages/core/tests/test_e2e.rs` (actual compilation) |
| Doc tests | 18 | Documentation examples (13 ignored) |

**E2E tests compile generated Rust code with `cargo check` (takes ~60s per test).**

**Quality Improvements (v0.1.1):**
- 30 new error path tests (parser, transform, generator, edge cases)
- 10 type validation tests (included in unit tests)
- Enhanced error messages with source location tracking
- Comprehensive migration guide at `docs/MIGRATION.md`

---

## Critical Design Decisions

### 1. Context-Aware Rust Generation
- **With #[account]:** Use `anchor_lang::prelude::*`, no manual derives
- **Without #[account]:** Use `borsh::{BorshSerialize, BorshDeserialize}`
- **Mixed modules:** Use Anchor imports for entire module if ANY struct has #[account]

### 2. Enum Support (Phase 3.1)
- **Rust:** Native `enum` with sequential discriminants (0, 1, 2...)
- **TypeScript:** Discriminated unions with `kind` field for type narrowing
- **Borsh:** `borsh.rustEnum()` with matching discriminants

### 3. Type Mapping
```
LUMOS      → Rust        → TypeScript
u64        → u64         → number
u128       → u128        → bigint
PublicKey  → Pubkey      → PublicKey
[T]        → Vec<T>      → T[]
Option<T>  → Option<T>   → T | undefined
```

### 4. User-Defined Type Validation (v0.1.1)
- **Early Detection:** Undefined types caught during transformation, not at compile time
- **Recursive Checking:** Validates types inside arrays and options
- **Clear Errors:** Shows exact location of undefined type reference (e.g., `Player.inventory`)
- **Implementation:** `packages/core/src/transform.rs:353-448`

### 5. CLI Security (v0.1.1)
- **Path Validation:** Prevents path traversal attacks (e.g., `../../etc/passwd`)
- **Canonicalization:** Resolves `..`, `.`, and symlinks before file operations
- **Write Permission Check:** Tests directory writability before generating files
- **Implementation:** `packages/cli/src/main.rs:745-785`

### 6. TypeScript Precision Warnings (v0.1.1)
- **JSDoc Comments:** Auto-generated warnings for u64/i64 fields
- **Precision Limit:** Documents 2^53-1 safe range for TypeScript `number`
- **Solana Context:** Specifically mentions lamports and large values
- **Implementation:** `packages/core/src/generators/typescript.rs:327-368`

### 7. Deprecation Warnings (v0.1.2)
- **Field-Level Deprecation:** Mark fields as deprecated with `#[deprecated]` or `#[deprecated("message")]`
- **Compile-Time Warnings:** Warnings emitted during `lumos validate` and `lumos generate`
- **Custom Messages:** Optional deprecation message for migration guidance
- **Enum Support:** Works for struct fields and enum variant fields
- **Implementation:**
  - Attribute parsing: `packages/core/src/parser.rs:252-298`
  - Transformation: `packages/core/src/transform.rs:338-404`
  - IR storage: `packages/core/src/ir.rs:73-74`

**Example:**
```rust
struct Account {
    balance: u64,
    #[deprecated("Use new_email instead")]
    email: String,
    new_email: Option<String>,
}
```
**Output:**
```
warning: Account.email: Use new_email instead
```

---

## Strategic Direction

**For long-term vision**: See `docs/VISION.md` - LUMOS evolution from schema DSL to programming language

---

## Core-Specific AI Guidelines

### ✅ DO (Core Repository):
- Run `cargo test` after any code changes
- Use `cargo fmt` before committing
- Check E2E tests pass (actual Rust compilation)
- Update this file when architecture changes
- Reference file:line when discussing code (e.g., `parser.rs:123`)

### ❌ DON'T (Core Repository):
- Add manual derives to `#[account]` structs (causes conflicts)
- Change IR structure without updating all generators
- Skip E2E tests (they catch real compilation issues)

---

## Example Schema

```rust
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    level: u16,
    experience: u64,
}

#[solana]
enum GameState {
    Active,
    Paused,
    Finished,
}
```

**Generates:**
- `generated.rs` - Rust structs with Anchor integration
- `generated.ts` - TypeScript interfaces + Borsh schemas

---

## Publishing Checklist

- [x] All tests passing (142/142)
- [x] Zero clippy warnings
- [x] Zero rustfmt violations
- [x] Security audit clean (0 vulnerabilities)
- [x] API documentation complete
- [x] Benchmarks added
- [x] CI/CD pipeline working
- [x] Organization migrated (getlumos)
- [x] Homepage updated (lumos-lang.org)
- [x] Published to crates.io (lumos-core v0.1.0, lumos-cli v0.1.0)
- [x] VSCode extension published (v0.5.0)
- [x] Published lumos-core v0.1.1 with security improvements
- [x] Published lumos-lsp v0.1.0 to crates.io
- [x] Published lumos-lsp v0.1.1 with library component (docs.rs support)

---

## Installation

```bash
# Install CLI
cargo install lumos-cli

# Verify installation
lumos --version

# Or use as library
cargo add lumos-core
```

**crates.io URLs:**
- https://crates.io/crates/lumos-core
- https://crates.io/crates/lumos-cli
- https://crates.io/crates/lumos-lsp

---

**Last Updated:** 2025-11-28
**Status:** v0.1.1 published | lumos-lsp v0.1.0 published | All packages live on crates.io
