# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-12-09

### Added

#### Source Location Tracking (#121)
- Enhanced error messages with precise line:column information
- Propagated `proc_macro2::Span` through AST → IR → validation pipeline
- `SourceLocation::from_span()` method for error reporting
- Type validation errors now show exact location: `"Undefined type 'Player' (at 2:5)"`
- Enabled `span-locations` feature for proc-macro2

#### Enum Migration Generation (#122)
- Complete migration code generation for enum schema changes
- `generate_rust_enum_migration()` - Generates `From` impls for Rust (195 lines)
- `generate_typescript_enum_migration()` - Generates migration functions for TypeScript (193 lines)
- Handles variant additions, removals, and all variant types (Unit, Tuple, Struct)
- Maps removed variants to default variant with clear comments
- Added 4 comprehensive migration tests

#### Language Server Protocol Enhancements
- Context-aware completions (#120) - Smart suggestions based on cursor position
- Document formatting support (#129) - Format-on-save for `.lumos` files
- Improved diagnostics and hover information
- Enhanced LSP stability and performance

#### Multi-Language Code Generation
- **Seahorse Python Generator (#56)** - Generate Python programs for Solana
- Python code generation with proper Borsh serialization
- Go code generation support
- Ruby code generation support
- `--lang` flag for multi-language output (rust, typescript, python, go, ruby)

#### Solana Ecosystem Integration
- **Metaplex Token Metadata (#58)** - Full Token Metadata standard compatibility
- **Native Solana Support (#57)** - `--target` flag for non-Anchor programs
- Generate programs compatible with native Solana (without Anchor framework)
- Metaplex NFT metadata structures and validation

#### Testing & Quality
- Cross-language schema compatibility tests (#117)
- TypeScript derive equivalents (#107)
- Enhanced test suite to 322 tests (from 202)
- Comprehensive migration test coverage
- E2E compilation tests for all generators

### Changed
- Updated parser to capture and propagate source spans
- Enhanced transform module with span-aware type validation
- All generators updated for better error reporting
- Improved clippy compliance across all modules
- Better rustfmt formatting consistency

### Fixed
- Resolved 30+ clippy warnings across codebase
- Fixed TypeScript type safety issues in generators
- Improved error handling in library code (removed unwrap() calls)
- Fixed formatting inconsistencies in metaplex and CLI modules

### Dependencies
- Enabled `span-locations` feature for proc-macro2
- Updated workspace dependencies for better compatibility

### Breaking Changes
None - fully backward compatible with v0.2.x schemas

---

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

[0.3.0]: https://github.com/getlumos/lumos/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/getlumos/lumos/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/getlumos/lumos/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/getlumos/lumos/releases/tag/v0.1.0
