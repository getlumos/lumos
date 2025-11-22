# CLAUDE.md - LUMOS Core

**Repository:** https://github.com/getlumos/lumos
**Website:** https://lumos-lang.org
**Purpose:** Type-safe schema language bridging TypeScript ↔ Rust for Solana development

---

## What is LUMOS?

Write data structures once in `.lumos` syntax → Generate production-ready Rust + TypeScript with guaranteed Borsh serialization compatibility.

**Status:** v0.1.1 Released | 108/108 tests passing | 0 warnings | 0 vulnerabilities

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
| CLI tool | ✅ | 4 commands (generate, validate, init, check) |
| VSCode extension | ✅ | Separate repo: getlumos/vscode-lumos |
| User-defined type validation | ✅ | Validates type references during transformation (v0.1.1) |
| Path traversal protection | ✅ | CLI validates output paths for security (v0.1.1) |
| u64 precision warnings | ✅ | JSDoc comments in generated TypeScript (v0.1.1) |

---

## Development Commands

```bash
# Run tests (108 tests, ~140s with E2E)
cargo test --all-features --workspace

# Check formatting
cargo fmt --all -- --check

# Lint (strict mode)
cargo clippy --all-targets --all-features -- -D warnings

# Build release
cargo build --release --all-features --workspace

# Generate from schema
cargo run --bin lumos -- generate examples/gaming/schema.lumos
```

---

## Test Suite (108 tests)

| Suite | Count | Location |
|-------|-------|----------|
| Unit tests | 47 | `packages/core/src/**/*.rs` |
| Parser integration | 5 | `packages/core/tests/integration_test.rs` |
| Error path tests | 30 | `packages/core/tests/test_error_paths.rs` |
| Rust generator | 5 | `packages/core/tests/test_rust_generator.rs` |
| TypeScript generator | 6 | `packages/core/tests/test_typescript_generator.rs` |
| E2E compilation | 9 | `packages/core/tests/test_e2e.rs` |
| Doc tests | 6 | Documentation examples (3 ignored) |

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

---

## Strategic Direction

**For long-term vision**: See `docs/VISION.md` - LUMOS evolution from schema DSL to programming language

---

## AI Assistant Guidelines

### ✅ DO:
- Run `cargo test` after any code changes
- Use `cargo fmt` before committing
- Check E2E tests pass (actual Rust compilation)
- Update this file when architecture changes
- Reference file:line when discussing code (e.g., `parser.rs:123`)

### ❌ DON'T:
- Add manual derives to `#[account]` structs (causes conflicts)
- Change IR structure without updating all generators
- Skip E2E tests (they catch real compilation issues)
- Use `echo` or bash for communication (output directly)
- Create .md files without surveying existing structure first

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

- [x] All tests passing (64/64)
- [x] Zero clippy warnings
- [x] Zero rustfmt violations
- [x] Security audit clean (0 vulnerabilities)
- [x] API documentation complete
- [x] Benchmarks added
- [x] CI/CD pipeline working
- [x] Organization migrated (getlumos)
- [x] Homepage updated (lumos-lang.org)
- [x] Published to crates.io (lumos-core v0.1.0, lumos-cli v0.1.0)
- [ ] VSCode extension published
- [ ] Publish v0.1.1 with security & validation improvements

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

---

## Community Resources

- **awesome-lumos** - 5 production-ready examples (NFT Marketplace, DeFi Staking, DAO Governance, Gaming Inventory, Token Vesting) - 53 types, 42 instructions, 4000+ LOC
- **docs-lumos** - Official documentation at https://lumos-lang.org
- **vscode-lumos** - VSCode extension (syntax highlighting, snippets, commands) - ready for VS Marketplace

---

## Related Repositories

- **vscode-lumos** - VSCode extension (syntax highlighting, snippets, commands)
- **awesome-lumos** - Community examples and full-stack applications
- **docs-lumos** - Official documentation site

---

**Last Updated:** 2025-11-21
**Status:** v0.1.1 - Security & Validation Improvements 🔒
