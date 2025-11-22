# Changelog

All notable changes to LUMOS will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Language Server Protocol (LSP) Implementation** (#45)
  - New `lumos-lsp` binary for IDE integration across all editors
  - Real-time diagnostics for syntax errors and undefined types
  - Auto-completion for Solana types, primitives, and attributes
  - Hover information with type definitions and documentation
  - Support for VS Code, Neovim, Emacs, and Sublime Text
  - 13 comprehensive tests for LSP handlers

### Changed

- Updated workspace to include `packages/lsp` package
- Increased total test suite from 129 to 142 tests

### Documentation

- Added LSP installation and setup guide in main README
- Created comprehensive `packages/lsp/README.md` with editor configurations
- Updated CLAUDE.md with LSP development commands and test coverage

---

## [0.1.1] - 2025-11-22

### Added

- Security analysis with static vulnerability detection
- Fuzzing support for generated code with comprehensive harness generation
- User-defined type validation during transformation
- Path traversal protection in CLI
- TypeScript precision warnings for u64/i64 fields
- 30 new error path tests
- 10 type validation tests

### Changed

- Enhanced error messages with source location tracking
- Improved CLI security with canonicalization and permission checks

### Documentation

- Added comprehensive migration guide at `docs/MIGRATION.md`
- Updated security documentation in `docs/security/`

---

## [0.1.0] - 2025-11-21

### Added

- Initial public release
- LUMOS schema language parser
- Rust code generator with Anchor support
- TypeScript code generator with Borsh schemas
- CLI with `generate`, `validate`, `init`, `check`, and `diff` commands
- Context-aware code generation (Anchor vs pure Borsh)
- Support for structs, enums, primitives, Solana types, and complex types
- Comprehensive test suite (129 tests)
- CI/CD pipeline with GitHub Actions
- VSCode extension (separate repository)

### Documentation

- Complete README with quick start guide
- Architecture documentation
- API documentation
- Example schemas

---

**Repository:** https://github.com/getlumos/lumos
**Website:** https://lumos-lang.org
