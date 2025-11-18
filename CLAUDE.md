# CLAUDE.md - LUMOS Core

**Repository:** https://github.com/getlumos/lumos
**Website:** https://lumos-lang.com
**Purpose:** Type-safe schema language bridging TypeScript ↔ Rust for Solana development

---

## What is LUMOS?

Write data structures once in `.lumos` syntax → Generate production-ready Rust + TypeScript with guaranteed Borsh serialization compatibility.

**Status:** Phase 3.3 Complete (Production Ready) | 64/64 tests passing | 0 warnings | 0 vulnerabilities

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
- `packages/cli/src/main.rs` - CLI commands (generate, validate, init, check)

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

---

## Development Commands

```bash
# Run tests (64 tests, ~150s with E2E)
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

## Test Suite (64 tests)

| Suite | Count | Location |
|-------|-------|----------|
| Unit tests | 39 | `packages/core/src/**/*.rs` |
| Parser integration | 5 | `packages/core/tests/integration_test.rs` |
| Rust generator | 8 | `packages/core/tests/test_rust_generator.rs` |
| TypeScript generator | 9 | `packages/core/tests/test_typescript_generator.rs` |
| E2E compilation | 9 | `packages/core/tests/test_e2e.rs` |

**E2E tests compile generated Rust code with `cargo check` (takes ~60s per test).**

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

```lumos
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
- [x] Homepage updated (lumos-lang.com)
- [x] Published to crates.io (lumos-core v0.1.0, lumos-cli v0.1.0)
- [ ] VSCode extension published

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

## Related Repositories

- **vscode-lumos** - VSCode extension (syntax highlighting, snippets, commands)
- **awesome-lumos** - Community examples and full-stack applications

---

**Last Updated:** 2025-11-18
**Status:** Published on crates.io ✅
