# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-11-24

### Added

#### Type Aliases (#52)
- Rust-style type alias syntax: `type UserId = PublicKey;`
- Recursive type resolution with circular reference detection
- Generate Rust `pub type` and TypeScript `export type` aliases
- Support for primitive types, Solana types, collections, and user-defined types

#### Multi-File Imports (#52)
- JavaScript-style import syntax: `import { Type1, Type2 } from "./file.lumos";`
- Automatic import discovery and resolution
- Multi-line import support
- Circular import detection with clear error messages
- Three-pass validation architecture (collect → resolve → validate)
- Cross-file type validation

#### New File Resolver
- `FileResolver` module (340 lines) for multi-file schema management
- Automatic dependency discovery
- Import caching to avoid duplicate parsing
- Loading stack for circular import detection

#### Examples
- `examples/type_aliases.lumos` - Comprehensive type alias examples (200+ lines, 23 types)
- `examples/imports/` - Multi-file import examples with 7 files
  - `types.lumos` - Common type definitions (24 types + 2 enums)
  - `accounts.lumos` - Account structs using imported types (180+ lines)
  - `instructions.lumos` - Instruction enums using imported types (240+ lines)
  - `README.md` - Complete documentation

#### Documentation
- Multi-file import patterns and best practices
- Type alias usage examples
- Error handling and troubleshooting guide

### Changed

- Updated AST to include `Import` and `TypeAlias` structs
- Extended IR with `TypeAliasDefinition` and `TypeAlias` variant
- Parser now extracts imports using regex before parsing
- Transform module includes `TypeAliasResolver` for recursive resolution
- CLI `generate` command now uses `FileResolver` for multi-file support
- All generators updated to handle type aliases

### Fixed

- 15+ non-exhaustive pattern matches across codebase
- Enum import validation timing (deferred until all files loaded)
- Multi-line import parsing with regex support
- Integration test metadata access (use `.unwrap()`)

### Dependencies

- Added `regex = "1.10"` for import parsing

### Testing

- Added 4 new file_resolver tests (single file, circular imports, multiple files, validation)
- All 202 tests passing (100% success rate)
- E2E compilation tests verified with generated code

### Breaking Changes

None - fully backward compatible with v0.1.x schemas

---

## [0.1.1] - 2025-11-23

### Added

- Schema versioning with `#[version]` attribute
- Automatic migration code generation
- Backward compatibility validation
- Deprecation warnings for schema fields
- Schema diff tool: `lumos diff`
- Language Server Protocol (LSP) implementation
- Custom derive macros support
- Fixed-size arrays with const generics

### Security

- User-defined type validation during transformation
- Path traversal protection in CLI
- u64 precision warnings in TypeScript output

### Changed

- Enhanced error messages with source location tracking
- Expanded test suite to 202 tests (from 64)

---

## [0.1.0] - 2025-11-01

Initial release of LUMOS - Type-safe schema language for Solana development.

### Added

- Core schema language with structs and enums
- Rust and TypeScript code generation
- Borsh serialization support
- Anchor framework integration
- Context-aware code generation
- CLI tool with multiple commands
- VSCode extension
- Comprehensive documentation

[0.2.0]: https://github.com/getlumos/lumos/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/getlumos/lumos/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/getlumos/lumos/releases/tag/v0.1.0
