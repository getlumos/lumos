# LUMOS Roadmap

**Vision**: Transform Solana development with the first type-safe workflow language - from schemas to complete automation

**For detailed vision**: See [docs/VISION.md](docs/VISION.md) (vertical expansion) and [docs/FUTURE.md](docs/FUTURE.md) (horizontal expansion)

**Last Updated**: November 25, 2025

---

## Current Status

**Phase 5 In Progress - Advanced Features** 🚀

LUMOS continues rapid evolution with IDE integration and schema versioning:

- ✅ **v0.2.0 LSP released** - Multi-editor support via Language Server Protocol
- ✅ **IntelliJ plugin** - Full LSP integration for IntelliJ IDEA, Rust Rover, CLion
- ✅ **v0.1.1 released** - 146 tests, zero warnings, zero vulnerabilities
- ✅ **Schema evolution complete** - Versioning, migration, compatibility, deprecation, diff tool
- ✅ **Security hardened** - Type validation, path protection, enhanced errors
- ✅ **VSCode extension** - v0.5.0 published to marketplace
- ✅ **5 community examples** - NFT, DeFi, DAO, Gaming, Vesting
- ✅ **Complete documentation** - Migration guide, API reference, quickstart
- ✅ **Interactive playground** - Live code generation at docs.lumos-lang.org/playground
- ✅ **Performance benchmarks** - Comprehensive Borsh comparison suite

**Completed**: Phase 5.1 (Schema Evolution - 100%), Phase 5.2 (IDE Integration - 100%), Phase 5.3 (Advanced Type System - 100%), Phase 6.2 (Tooling Ecosystem - 100%), Phase 6.3 (Security & Validation - 100%)
**Active**: Phase 5.4 (Multi-Language Generation), Phase 6.1 (Framework Integration)
**Next**: Era 2 - Language Transformation (Phase 7+)

---

## 📍 The LUMOS Evolution

LUMOS is evolving in two major eras:

### **Era 1: DSL Completion** (Phases 5-6, Q1 2026)
Become the definitive schema language for Solana - complete type system, IDE integration, ecosystem tools

**Timeline**: 3 months (Dec 2025 - March 2026)
**Milestone**: When Phase 6 closes → **LUMOS DSL Feature Complete** ✅

### **Era 2: Language Transformation** (Phases 7-9, Q2 2026 - Q1 2027)
Transform from schema DSL → full programming language for type-safe Solana workflows and automation

**Timeline**: 12 months (Apr 2026 - March 2027)
**Milestone**: **LUMOS becomes a real programming language** with parser, runtime, and execution engine

---

# Era 1: DSL Completion (Q1 2026)

## Phase 5: Advanced Features (Q1 2026)

### 5.1 Schema Evolution

**Goal**: Support schema changes without breaking deployed programs

**Status**: 5/5 complete (100%) ✅

**Issues:**
- [x] Add schema versioning syntax with `#[version = "1.0.0"]` attribute [#40] ✅ **COMPLETE**
- [x] Generate automatic migration code between schema versions [#41] ✅ **COMPLETE**
- [x] Validate backward compatibility between schema versions [#42] ✅ **COMPLETE**
- [x] Add deprecation warnings for old schema fields [#43] ✅ **COMPLETE**
- [x] Create schema diff tool: `lumos diff v1.lumos v2.lumos` [#44] ✅ **COMPLETE**

**Completed**:
- #40 (Nov 23, 2025) - Schema versioning with semantic versioning support
  - Added `#[version]` attribute parser and validation
  - Auto-generated version constants in Rust and TypeScript
  - Created comprehensive examples and documentation
  - 6 new unit tests, 2 integration tests

- #41 (Nov 23, 2025) - Automatic migration code generation
  - New migration module with SchemaDiff and SchemaChange types (724 lines)
  - CLI command `lumos migrate` with full option support
  - Generates both Rust and TypeScript migration code
  - Migration safety classification (Safe vs Unsafe)
  - Smart default values for all primitive and complex types
  - Comprehensive examples in examples/migration/
  - Full documentation at docs/schema-evolution/migrations.md
  - 7 new unit tests for diff detection and code generation

- #42 (Nov 23, 2025) - Backward compatibility validation
  - New compat module with CompatibilityChecker (434 lines)
  - CLI command `lumos check-compat` with text/JSON output
  - Breaking vs compatible change classification
  - SemVer version bump validation
  - Verbose mode with detailed explanations
  - Strict mode (fail on warnings)
  - Exit codes: 0 (compatible), 1 (breaking), 2 (warnings)
  - Complete documentation at docs/schema-evolution/compatibility.md
  - CLI reference at docs/cli/check-compat.md
  - 13 new comprehensive tests (168 total tests passing)

- #43 (Nov 23, 2025) - Deprecation warnings for schema fields
  - Added `#[deprecated]` attribute parser with message support
  - Generates Rust `#[deprecated]` attributes in output
  - Generates TypeScript `@deprecated` JSDoc comments
  - Compile-time warnings for deprecated field usage
  - IDE support (strikethrough, warnings in VS Code)
  - Documentation at docs/schema-evolution/deprecation.md

- #44 (Nov 23, 2025) - Schema diff tool
  - New CLI command `lumos diff` for comparing schemas
  - Detects added, removed, and modified fields
  - Color-coded terminal output with visual indicators
  - JSON and Markdown output formats
  - Breaking change detection
  - Filter by specific types, statistics mode
  - Git integration support for commit comparisons

**Success Metric**: Zero-downtime schema upgrades ✅ **ACHIEVED**

**Milestone**: 🎯 **PHASE 5.1 COMPLETE** - Full schema evolution toolkit with versioning, migration, compatibility checking, deprecation warnings, and visual diff tool

### 5.2 IDE Integration

**Goal**: Multi-editor support beyond VSCode

**Status**: 5/5 complete (100%) ✅

**Issues:**
- [x] Implement Language Server Protocol (LSP) for LUMOS [#45] ✅ **COMPLETE**
- [x] Create IntelliJ IDEA / Rust Rover plugin for LUMOS [#46] ✅ **COMPLETE**
- [x] Create Neovim plugin with Tree-sitter grammar for LUMOS [#47] ✅ **COMPLETE**
- [x] Create Emacs mode for LUMOS [#48] ✅ **COMPLETE**
- [x] Create Sublime Text package for LUMOS [#49] ✅ **COMPLETE**

**Completed**:
- #45 (Nov 22, 2025) - Language Server Protocol implementation
  - Full LSP server with diagnostics, completion, and hover
  - Published lumos-lsp v0.1.1 to crates.io
  - Multi-editor support (VS Code, Neovim, Emacs, Sublime, etc.)
  - 13 new LSP-specific tests

- #46 (Nov 23, 2025) - IntelliJ IDEA / Rust Rover plugin
  - New repository: getlumos/intellij-lumos
  - LSP client integration with lumos-lsp server
  - File type recognition for .lumos files
  - Comprehensive test suite (39 tests across 5 test files)
  - Documentation at docs-lumos/src/content/docs/editors/intellij.md
  - Supports IntelliJ IDEA, Rust Rover, CLion (2024.1+)

- #47 (Nov 23, 2025) - Neovim plugin with Tree-sitter grammar
  - New repositories: getlumos/tree-sitter-lumos, getlumos/nvim-lumos
  - Complete Tree-sitter grammar for .lumos syntax highlighting
  - Full LSP integration with lumos-lsp server
  - 6 comprehensive grammar test cases (all passing)
  - Pre-configured keybindings (gd, K, gr, <leader>rn, <leader>ca, <leader>f)
  - Support for lazy.nvim, packer.nvim, manual installation
  - Comprehensive documentation with troubleshooting guide

- #48 (Nov 24, 2025) - Emacs mode for LUMOS
  - New repository: getlumos/lumos-mode
  - Complete Emacs major mode with syntax highlighting and smart indentation
  - Full LSP integration via lsp-mode and lumos-lsp server
  - 14 comprehensive test cases (all passing)
  - GitHub Actions CI testing across Emacs 27.2, 28.2, 29.1, snapshot
  - MELPA recipe ready for package distribution
  - Auto-completion, diagnostics, hover, go-to-definition support
  - Customizable variables (indent-offset, lsp-server-command)
  - Comprehensive README with installation and configuration guide

- #49 (Nov 24, 2025) - Sublime Text package for LUMOS
  - New repository: https://github.com/getlumos/sublime-lumos
  - YAML-based .sublime-syntax file with full syntax highlighting (120 lines)
  - LSP integration via LSP-lumos.sublime-settings and LSP package
  - 6 snippets for common patterns (struct, enum variants, account, deprecated)
  - Package settings with 2-space indentation and auto-formatting
  - Comment toggling, bracket matching, and auto-indentation
  - Support for Sublime Text 4 and 3 (build 3103+)
  - Comprehensive README with 3 installation methods (270 lines)
  - Dual-licensed (MIT + Apache 2.0)
  - Submitted to Package Control (PR #9251)

**Success Metric**: LSP used by 3+ editors ✅ **ACHIEVED** (VS Code, IntelliJ IDEA/Rust Rover, Neovim, Emacs, Sublime Text)

### 5.3 Advanced Type System

**Goal**: Express complex Solana program constraints

**Status**: 5/5 complete (100%) ✅

**Issues:**
- [x] Add custom derive macros support to LUMOS [#50] ✅ **COMPLETE**
- [x] Add const generics support for fixed-size arrays in LUMOS [#51] ✅ **COMPLETE**
- [x] Add type aliases and imports to LUMOS [#52] ✅ **COMPLETE**
- [x] Add nested module support to LUMOS [#53] ✅ **COMPLETE** [#113]
  - [x] #53a: AST foundations with visibility support ✅ **COMPLETE**
  - [x] #53b: Module resolution & hierarchical loading ✅ **COMPLETE**
  - [x] #53c: Use statement validation & CLI integration ✅ **COMPLETE**
- [x] Add generic struct/enum definitions to LUMOS [#54] ✅ **COMPLETE**

**Completed**:
- #54 (Nov 25, 2025) - Generic type parameters support
  - Added `type_params: Vec<String>` to AST (StructDef, EnumDef)
  - Added `TypeSpec::Generic(String)` variant for generic parameter types
  - Implemented `parse_generic_params()` to extract from `syn::Generics`
  - Added validation for unsupported features (bounds, where clauses, lifetimes, const generics)
  - Updated IR with `generic_params` field in StructDefinition and EnumDefinition
  - Added `TypeInfo::Generic(String)` variant in IR layer
  - Updated transform layer to pass generic context through type parsing
  - Modified Rust generator to output `struct Foo<T>` and `enum Result<T, E>` syntax
  - Modified TypeScript generator to output `interface Foo<T>` and `type Result<T, E>` syntax
  - Fixed 13+ non-exhaustive pattern matches across all modules
  - Added 10 comprehensive parser tests for generic functionality
  - Created examples/generics.lumos demonstrating various generic patterns
  - All 139 tests passing with full generic support
- #50 (Nov 24, 2025) - Custom derive macros support
  - Extended `AttributeValue` enum with `List(Vec<String>)` variant
  - Added `#[derive(...)]` parser with comma-separated list support
  - Added `custom_derives` field to IR `Metadata` struct
  - Implemented intelligent derive deduplication with `merge_derives()`
  - Context-aware generation: Anchor accounts get only custom derives
  - Created comprehensive example at examples/custom_derives.lumos
  - 17 new tests (parser, transform, generator, end-to-end)
  - All 196 tests passing
  - Future enhancement: TypeScript derive equivalents (#107)

- #51 (Nov 24, 2025) - Fixed-size arrays (const generics)
  - Added `FixedArray { element, size }` variant to AST and IR
  - Parser extracts size from `[T; N]` syntax with validation (1-1024)
  - Generates Rust `[T; N]` format correctly
  - Generates TypeScript `T[]` with `borsh.array(element, size)` (no length prefix!)
  - Size calculation: `element_size * count` (no 4-byte Vec prefix)
  - Supports nested arrays: `[[u8; 10]; 10]`
  - Example schema at examples/fixed_arrays.lumos
  - Updated 10+ files across codebase (parser, transform, generators, CLI, etc.)
  - All 116 tests passing
  - Common Solana patterns now supported: `[u8; 32]` for hashes, `[PublicKey; N]` for authority lists

- #52 (Nov 24, 2025) - Type aliases and imports
  - Added `Import` and `TypeAlias` structs to AST with span tracking
  - Implemented JavaScript-style import parser: `import { Type1, Type2 } from "./file.lumos"`
  - Implemented Rust-style type alias parser: `type UserId = PublicKey`
  - Added `TypeAliasDefinition` to IR and `TypeAlias` variant to `TypeDefinition`
  - Created `TypeAliasResolver` with recursive alias resolution and circular reference detection
  - Implemented multi-file `FileResolver` with automatic import discovery and circular import detection
  - Three-pass validation: collect aliases → resolve recursively → transform → validate across all files
  - Updated all generators to handle type aliases (Rust `pub type`, TypeScript `export type`)
  - Fixed 15+ non-exhaustive pattern matches across codebase
  - Updated CLI to use FileResolver for multi-file schema generation
  - Added regex dependency for import parsing (multi-line support)
  - Created comprehensive examples: `examples/type_aliases.lumos` (200+ lines, 23 types)
  - Created multi-file import examples: `examples/imports/{types,accounts,instructions}.lumos`
  - Added 4 new file_resolver tests (single file, circular imports, multiple files, validation)
  - All 202 tests passing (including E2E compilation tests)
  - **Feature complete**: Single-file type aliases + multi-file imports with full validation

- #53a (Nov 24, 2025) - AST foundations for module system
  - Added `Module`, `UseStatement`, `ModulePath`, `PathSegment` structs to AST
  - Added `Visibility` enum (Public/Private) for type definitions
  - Extended `StructDef`, `EnumDef`, `TypeAlias` with `visibility` field
  - Updated `Item` enum with `Module` and `Use` variants
  - Implemented `parse_visibility()` to extract pub keyword from syn
  - Added helper methods for `ModulePath` (is_absolute, final_ident, to_string)
  - Updated Transform and FileResolver to handle new item types
  - Fixed all compilation errors and non-exhaustive pattern matches
  - Updated all test fixtures with visibility field
  - All 120 tests passing (107 core + 13 LSP)
  - **Sub-issues created**: #113 (resolution), #114 (generators), #115 (tests)
  - **Foundation laid**: Ready for module resolution implementation

- #53 (Nov 25, 2025) - Complete hierarchical module system [#113]
  - **Parser support**:
    - Parse `mod name;` declarations (external modules only)
    - Parse `use path::Type;` and `use path::Type as Alias;` statements
    - Support visibility modifiers (`pub`) on all items
    - Handle path keywords (crate::, super::, self::)
    - Error handling for unsupported syntax (glob imports, grouped imports)
    - Added 11 new parser tests (mod/use declarations, visibility, error cases)
  - **Module resolution**:
    - Created `ModuleResolver` with dual file resolution strategies
    - Sibling file: `current_dir/name.lumos`
    - Directory module: `current_dir/name/mod.lumos`
    - Hierarchical module tree construction with parent-child tracking
    - Circular dependency detection with clear error chains
    - Recursive loading with caching to avoid duplicate parsing
    - Integration with TypeAliasResolver for cross-module types
    - Added 6 integration tests (single, sibling, directory, nested, circular, missing)
  - **Use statement validation**:
    - Path resolution for crate::, super::, self:: keywords
    - Type existence checking in target modules
    - Visibility enforcement (private types cannot be imported)
    - Module tree traversal for nested paths (crate::models::User)
    - Parent/child module tracking for super resolution
    - Clear error messages with file paths
    - Added 6 comprehensive validation tests
  - **CLI integration**:
    - Auto-detect module vs import resolution strategy
    - ModuleResolver for `mod name;` declarations
    - FileResolver for `import` statements (backward compatible)
    - Single-file fallback for schemas without dependencies
    - Unified `resolve_schema()` function
    - Updated messaging to show file count
    - Tested with multi-module project (verified Rust + TypeScript output)
  - **Test coverage**: All 130 tests passing (+23 new tests)
  - **Feature complete**: Full Rust-style module system with hierarchical organization

**Success Metric**: Support 95% of Anchor program patterns

### 5.4 Multi-Language Code Generation

**Goal**: Generate schemas in Python, Go, and Ruby alongside Rust and TypeScript

**Issues to create:**
- [ ] Design multi-language code generation architecture [#67]
- [ ] Implement Python schema generator with Borsh serialization [#68]
- [ ] Implement Go schema generator with Borsh serialization [#69]
- [ ] Implement Ruby schema generator with Borsh serialization [#70]
- [ ] Add language-specific type mapping documentation [#71]
- [ ] Create cross-language schema compatibility tests [#72]
- [ ] Add `--lang` flag to `lumos generate` command [#73]

**Success Metric**: One `.lumos` file generates type-safe schemas in 5 languages (Rust, TypeScript, Python, Go, Ruby)

**Example:**
```bash
# Generate for all languages
lumos generate schema.lumos --lang rust,typescript,python,go,ruby

# Output:
# - schema.rs (Rust with Borsh)
# - schema.ts (TypeScript with Borsh)
# - schema.py (Python dataclass with Borsh)
# - schema.go (Go struct with Borsh)
# - schema.rb (Ruby class with Borsh)
```

**Benefits:**
- Polyglot codebases with guaranteed serialization compatibility
- Backend in Rust/Go, frontend in TypeScript, scripts in Python/Ruby
- Single source of truth for data structures across entire stack

---

## Phase 6: Ecosystem Integration (Q1 2026)

### 6.1 Framework Integration

**Issues to create:**
- [ ] Create Anchor framework plugin for LUMOS [#55] **HIGH PRIORITY**
- [ ] Add Seahorse integration for Python-based Solana development [#56]
- [ ] Add native Solana program support (non-Anchor) [#57]
- [ ] Add Metaplex standard compatibility for NFT schemas [#58]

### 6.2 Tooling Ecosystem

**Status**: 4/4 complete (100%) ✅

**Issues:**
- [x] Create `cargo lumos` subcommand for Rust workflows [#59] ✅ **COMPLETE**
- [x] Create GitHub Action for CI/CD auto-generation [#60] ✅ **COMPLETE**
- [x] Create pre-commit hook for schema validation [#61] ✅ **COMPLETE**
- [x] Create npm package for JavaScript/TypeScript projects [#62] ✅ **COMPLETE**

**Completed**:
- #59 (Nov 25, 2025) - cargo lumos subcommand for Rust workflows
  - New package `cargo-lumos` in workspace
  - Supports all lumos commands via `cargo lumos <cmd>`
  - Reads config from `[package.metadata.lumos]` in Cargo.toml
  - Auto-detects local debug/release builds for development
  - External command passthrough for full CLI compatibility
  - 5 unit tests

- #60 (Nov 2025) - GitHub Action for automated validation and generation
  - Published to GitHub Marketplace as `getlumos/lumos-action@v1`
  - Auto-install CLI, validate schemas, generate code, drift detection
  - Comprehensive CI/CD integration with PR comments
  - Available at https://github.com/marketplace/actions/lumos-generate

- #61 (Nov 2025) - Pre-commit hook for schema validation
  - Repository: https://github.com/getlumos/lumos-pre-commit
  - Three hooks: lumos-validate, lumos-format, lumos-check-generated
  - Works with pre-commit framework
  - Fast validation (< 1s)

- #62 (Nov 23, 2025) - npm package for JavaScript/TypeScript projects
  - Published `@getlumos/cli` to npm registry
  - WebAssembly-based (~750 KB optimized binary)
  - No Rust toolchain required for JS/TS developers
  - CLI commands: generate, validate
  - Programmatic API for Node.js/TypeScript integration
  - Comprehensive documentation with examples
  - Build tool integration guides (Vite, Webpack, CI/CD)

### 6.3 Security & Validation

**Status**: 4/4 complete (100%) ✅

**Issues:**
- [x] Add static analysis for common vulnerabilities ✅ **COMPLETE**
- [x] Add account size overflow detection ✅ **COMPLETE**
- [x] Create security audit checklist generator [#65] ✅ **COMPLETE**
- [x] Add fuzzing support for generated code testing [#66] ✅ **COMPLETE**

**Completed**:
- Static Security Analysis (Nov 2025)
  - Implemented security_analyzer.rs with 8 vulnerability patterns
  - Detects missing signer checks, unchecked arithmetic, missing discriminators
  - Strict mode for comprehensive analysis
  - 5 unit tests for security patterns

- Account Size Calculator (Nov 2025)
  - Implemented size_calculator.rs with precise Borsh size calculation
  - Detects potential account size overflows
  - Includes discriminator in size calculations
  - 5 unit tests for size validation

- Security Audit Generator (Nov 2025)
  - Implemented audit_generator.rs for automated checklist generation
  - Priority-based sorting (High → Medium → Low)
  - Covers account validation, arithmetic safety, signer checks
  - 4 unit tests for audit generation

- Fuzzing Support (Nov 2025)
  - Implemented fuzz_generator.rs for cargo-fuzz integration
  - Generates fuzz targets for round-trip testing
  - Comprehensive corpus generation with edge cases
  - 8 unit tests for fuzz target generation

**Milestone**: 🎯 **DSL FEATURE COMPLETE** - LUMOS becomes the most comprehensive schema language for Solana

---

# Era 2: Language Transformation (Q2 2026 - Q1 2027)

**The Big Vision**: LUMOS evolves from "schema generator" to "programmable workflow language"

## Why a Language?

Solana development today requires juggling multiple tools:
- Write schemas (LUMOS/manual)
- Script deployments (bash, JavaScript, Python)
- Build automation (Makefiles, npm scripts)
- Manage workflows (CI/CD configs, YAML)

**What if one language handled all of it?**

LUMOS becomes:
- **The TypeScript of workflows** - Type-safe automation for Solana
- **The Terraform of Solana** - Declarative + executable infrastructure
- **The Hardhat for Solana** - Unified developer experience

---

## Phase 7: Core Language Foundation (Q2-Q3 2026)

**Goal**: Transform `.lumos` from schema DSL to real programming language

**Timeline**: 6 months (Apr - Sep 2026)
**Estimated**: 285-480 commits, 40-60 issues

### 7.1 Parser & AST

**Issues to create:**
- [ ] Design extended LUMOS grammar (variables, functions, control flow)
- [ ] Research parser library comparison (chumsky vs nom vs lalrpop)
- [ ] Implement lexer/tokenizer for LUMOS language
- [ ] Define AST data structures (expressions, statements, declarations)
- [ ] Implement parser for variable declarations and assignments
- [ ] Implement parser for function definitions
- [ ] Implement parser for control flow (if/else, loops)
- [ ] Implement parser for module system (import/export)
- [ ] Add error recovery and diagnostic system
- [ ] Add source location tracking for debugging
- [ ] Create syntax error messages with suggestions

### 7.2 Evaluator & Runtime Foundation

**Issues to create:**
- [ ] Design AST evaluation engine architecture
- [ ] Implement variable binding and scoping system
- [ ] Implement function definitions and call evaluation
- [ ] Implement expression evaluation (arithmetic, logical, comparison)
- [ ] Implement control flow evaluation (if/else, while, for)
- [ ] Add module system (import/export resolution)
- [ ] Create basic REPL for interactive development
- [ ] Add runtime error handling and stack traces
- [ ] Implement closure support for functions

### 7.3 Standard Library (Core)

**Issues to create:**
- [ ] Design standard library architecture and organization
- [ ] Implement string manipulation primitives (concat, split, trim, etc.)
- [ ] Implement collection types (List, Map, Set)
- [ ] Add file I/O operations (read, write, exists, etc.)
- [ ] Add HTTP client basics (GET, POST requests)
- [ ] Add JSON parsing and serialization
- [ ] Add TOML parsing and serialization
- [ ] Implement environment variable access
- [ ] Add path manipulation utilities
- [ ] Create standard library documentation

**Success Metric**: Execute simple LUMOS programs locally

**Example LUMOS code (future):**

```lumos
// Variables and functions
let cluster = "devnet"
let wallet_path = env("WALLET_PATH")

fn deploy_program(path: String, cluster: String) {
  let config = parse_toml("Anchor.toml")
  let program = build_anchor_program(path, config)

  deploy(program, {
    cluster: cluster,
    wallet: wallet_path
  })
}

deploy_program(".", cluster)
```

---

## Phase 8: Type System Layer (Q3-Q4 2026)

**Goal**: Bring TypeScript-like gradual typing to workflows

**Timeline**: 6 months (Jul 2026 - Dec 2026, overlaps with Phase 7)
**Estimated**: 220-440 commits, 35-50 issues

### 8.1 Gradual Type System

**Issues to create:**
- [ ] Design type annotation syntax for LUMOS
- [ ] Implement type representation and type environment
- [ ] Add type annotation parser (function signatures, variable types)
- [ ] Implement type inference for variable bindings
- [ ] Implement type inference for function return types
- [ ] Add type checking pass for expressions
- [ ] Add type checking for function calls and arguments
- [ ] Implement generic type parameters (List<T>, Option<T>)
- [ ] Add union types (T | U)
- [ ] Add intersection types (T & U)
- [ ] Create type error messages with suggestions
- [ ] Add type diagnostics in LSP

### 8.2 Solana-Native Types

**Issues to create:**
- [ ] Design Solana primitive types (Pubkey, Signature, Lamports)
- [ ] Implement `Pubkey` type with validation
- [ ] Implement `Signature` type
- [ ] Implement `Lamports` type with u64 precision handling
- [ ] Add `Account<T>` type for program accounts
- [ ] Add `Instruction` type with validation
- [ ] Add `Transaction` builder types
- [ ] Implement Solana type conversions and serialization
- [ ] Add type-safe RPC client types

### 8.3 Anchor IDL Integration

**Issues to create:**
- [ ] Design IDL-to-types architecture
- [ ] Implement Anchor IDL parser
- [ ] Generate LUMOS types from Anchor IDL
- [ ] Auto-generate type-safe instruction builders from IDL
- [ ] Add account struct mapping from IDL
- [ ] Create IDL-based autocomplete in LSP
- [ ] Add runtime IDL validation

**Success Metric**: Write type-safe Solana scripts in LUMOS

**Example LUMOS code (future):**

```lumos
// Type-safe Solana operations
fn airdrop(recipients: List<Pubkey>, amount: Lamports) -> Transaction {
  recipients.map(|addr| {
    transfer_instruction(addr, amount)
  }).build_transaction()
}

// Load types from Anchor IDL
import { UserAccount, initialize } from "anchor:my-program"

let user: UserAccount = {
  wallet: pubkey("..."),
  balance: lamports(1_000_000)
}
```

---

## Phase 9: Compiler & Runtime (Q4 2026 - Q1 2027)

**Goal**: Execute LUMOS workflows natively and compile to other formats

**Timeline**: 6 months (Oct 2026 - March 2027)
**Estimated**: 330-640 commits, 50-80 issues

### 9.1 IR (Intermediate Representation) & Lowering

**Issues to create:**
- [ ] Design LUMOS IR (Intermediate Representation) architecture
- [ ] Implement AST → IR transformation
- [ ] Create IR optimization passes
- [ ] Implement LUMOS → Solana CLI command lowering
- [ ] Implement LUMOS → Anchor CLI command lowering
- [ ] Implement LUMOS → bash script generation
- [ ] Implement LUMOS → Python script generation (solana-py integration)
- [ ] Implement LUMOS → Go script generation (solana-go-sdk integration)
- [ ] Implement LUMOS → Ruby script generation (solana.rb integration)
- [ ] Implement LUMOS → TOML config generation
- [ ] Implement LUMOS → YAML config generation
- [ ] Add LUMOS → GitHub Actions workflow generation
- [ ] Add LUMOS → Docker Compose generation
- [ ] Create `lumos compile` command with multiple targets

### 9.2 LUMOS Runtime Engine

**Issues to create:**
- [ ] Design native runtime execution engine
- [ ] Implement `lumos run` command for native execution
- [ ] Add Solana RPC client integration
- [ ] Add Jito RPC client integration
- [ ] Implement transaction builder and simulator
- [ ] Add parallel execution engine for workflows
- [ ] Implement workflow orchestration (dependencies, retries)
- [ ] Add cron-like scheduling support
- [ ] Create execution logs and debugging
- [ ] Add runtime performance profiling
- [ ] Implement sandboxed execution environment
- [ ] Add transaction simulation and dry-run mode

### 9.3 Package Ecosystem

**Issues to create:**
- [ ] Design package manager architecture (LPM - LUMOS Package Manager)
- [ ] Implement package manifest format (lumos.toml)
- [ ] Create package registry backend
- [ ] Implement `lumos install` command
- [ ] Implement `lumos publish` command
- [ ] Add dependency resolution algorithm
- [ ] Create standard library package: `lumos-solana`
- [ ] Create standard library package: `lumos-anchor`
- [ ] Create standard library package: `lumos-jito`
- [ ] Create standard library package: `lumos-metaplex`
- [ ] Create standard library package: `lumos-http`
- [ ] Add private package support
- [ ] Add package versioning and semver
- [ ] Create package documentation generation
- [ ] Build package search and discovery (lumos-lang.org/packages)

### 9.4 Cloud Platform (Optional SaaS - Future)

**Note**: Cloud platform details remain exploratory. Core language stays open source.

**Potential features (not committed):**
- [ ] Design LUMOS Cloud architecture
- [ ] Hosted workflow execution runner
- [ ] Secrets management integration
- [ ] Execution logs and monitoring dashboard
- [ ] Scheduled jobs (cron-like in cloud)
- [ ] Webhook triggers for workflows
- [ ] RPC batching and optimization

**Success Metric**: Execute production workflows with `lumos run`, publish packages to registry

**Example LUMOS code (future):**

```lumos
// Import packages
import { deploy, airdrop } from "lumos-solana"
import { send_bundle } from "lumos-jito"

// Execute workflow
let recipients = load_csv("recipients.csv")
let tx = airdrop(recipients, lamports(1_000_000))

send_bundle([tx], { tip: lamports(10_000) })
```

---

## Phase 10: Horizontal Expansion (2027+)

**Goal**: Expand beyond Solana

**Timeline**: 12+ months (2027+)
**Estimated**: 200+ commits, 40+ issues

### 10.1 Multichain Support

**Potential issues:**
- [ ] Add EVM chain support (Ethereum, Polygon, Base)
- [ ] Add Cosmos SDK integration
- [ ] Add Sui blockchain support
- [ ] Add Aptos blockchain support
- [ ] Create cross-chain transaction builders
- [ ] Add multichain wallet abstraction

### 10.2 DevOps Automation

**Potential issues:**
- [ ] Add Docker integration and container management
- [ ] Add Kubernetes deployment support
- [ ] Create Terraform-like infrastructure-as-code features
- [ ] Add GitHub Actions native integration
- [ ] Create CI/CD pipeline templates
- [ ] Add AWS/GCP/Azure cloud provider support

### 10.3 General Purpose Scripting

**Potential issues:**
- [ ] Replace Makefile/Justfile use cases
- [ ] Add system automation capabilities
- [ ] Create data processing pipelines
- [ ] Add API testing framework
- [ ] Create database migration tools
- [ ] Add web scraping utilities

**Success Metric**: LUMOS used beyond Solana ecosystem

---

## Development Velocity Targets

### To DSL Complete (Phase 5-6)
- **Timeline**: 3 months (Dec 2025 - Mar 2026)
- **Commits**: 150-250 total
- **Issues**: 25-35 total
- **Daily**: 2-3 commits/day, 1 issue every 2-3 days

### To ENDGAME (Phase 7-9)
- **Timeline**: 12 months (Apr 2026 - Mar 2027)
- **Commits**: 900-1,400 total
- **Issues**: 180-260 total
- **Daily**: 3-5 commits/day, 1 issue every 1-2 days

### Aggressive Pace (Target)
- **3-5 commits/day** minimum
- **1 issue completed every 1-2 days**
- **Weekly demos** of new features
- **Monthly milestone** reviews

**This pace scares copycats** - execution speed creates an unbeatable moat.

---

## Why LUMOS Will Win

### 1. Category Creation
No direct competitor exists in "type-safe workflow language for Solana" - we define the category

### 2. Vertical Technical Moat
Type system + macro system + IR compiler + LSP + runtime = years to replicate

### 3. Solana-Native Design
Built specifically for Solana's constraints (accounts, transactions, Borsh, Anchor)

### 4. Familiar Yet Better
TypeScript-like syntax + Terraform-like declarative model = easy adoption

### 5. Open Source + Ecosystem
Core language free forever, monetize via cloud platform and premium extensions

### 6. Fast Execution
3-5 commits/day = competitors can't catch up even if they copy the idea

---

## Completed Phases

### Phase 5: Advanced Features 🚧 (In Progress - Nov 2025)

**Overall Progress**: 13/23 features complete (57%)

**5.1 Schema Evolution (100% complete) ✅:**
- [x] Schema versioning with #[version] attribute (#40)
- [x] Automatic migration code generation (#41)
- [x] Backward compatibility validation (#42)
- [x] Deprecation warnings for schema fields (#43)
- [x] Schema diff tool: `lumos diff` (#44)

**5.2 IDE Integration (100% complete) ✅:**
- [x] Language Server Protocol implementation (#45)
- [x] IntelliJ IDEA / Rust Rover plugin (#46)
- [x] Neovim plugin with Tree-sitter grammar (#47)
- [x] Emacs mode (#48)
- [x] Sublime Text package (#49)

**5.3 Advanced Type System (100% complete) ✅:**
- [x] Custom derive macros support (#50)
- [x] Fixed-size arrays (const generics) (#51)
- [x] Type aliases and imports (#52)
- [x] Nested module support (#53)
- [x] Generic struct/enum definitions (#54)

**5.4 Multi-Language Code Generation (0% complete):**
- No issues started yet

**6.2 Tooling Ecosystem (100% complete) ✅:**
- [x] cargo lumos subcommand (#59)
- [x] GitHub Action for CI/CD (#60)
- [x] Pre-commit hook for validation (#61)
- [x] npm package for JS/TS (#62)

**6.3 Security & Validation (100% complete):**
- [x] Static security analysis
- [x] Account size overflow detection
- [x] Security audit checklist generator
- [x] Fuzzing support

**Result**: Strong foundation for schema evolution and IDE support, complete security validation toolkit

---

### Phase 4.3: Developer Experience ✅ (Completed Nov 2025)

- [x] Migration guide from manual Borsh serialization (docs/MIGRATION.md - 295 lines)
- [x] Performance benchmarks vs manual implementations (packages/core/benches/)
- [x] API reference documentation with examples (docs-lumos API section)
- [x] "LUMOS in 5 minutes" quickstart guide (docs-lumos quick-start)
- [x] Interactive playground on lumos-lang.org (https://docs.lumos-lang.org/playground/)
- [x] Video tutorial series (deferred - documentation sufficient for v1.0)

**Result**: Complete documentation ecosystem with interactive playground - 6/6 items complete (100%)

### Phase 4.2: Community Examples ✅ (Completed Nov 2025)

- [x] NFT marketplace schema (#7 - awesome-lumos/examples/nft-marketplace)
- [x] DeFi staking program (#8 - awesome-lumos/examples/defi-staking)
- [x] DAO governance structure (#9 - awesome-lumos/examples/dao-governance)
- [x] Gaming inventory system (#10 - awesome-lumos/examples/gaming-inventory)
- [x] Token vesting contract (#11 - awesome-lumos/examples/token-vesting)

**Result**: 5 complete full-stack examples with schemas, Anchor programs, and TypeScript clients

### Phase 4.1: VSCode Extension Polish ✅ (Completed Nov 2025)

- [x] Published extension to VS Marketplace (v0.1.0 - v0.5.0)
- [x] Added error diagnostics with red squiggles
- [x] Implemented auto-completion for Solana types (PublicKey, Signature, etc.)
- [x] Added format-on-save support
- [x] Created quick fix suggestions for common errors
- [x] Deployed documentation site at docs.lumos-lang.org with SSL

**Result**: Full-featured VSCode extension with professional DX

### Phase 4.0: Security & Validation Improvements ✅ (Completed Nov 2025)

- [x] User-defined type validation during transformation (#26)
- [x] Path traversal protection in CLI (#25)
- [x] u64 precision warnings in TypeScript output (#24)
- [x] Enhanced error messages with source location tracking (#27)
- [x] 30 new error path tests for edge cases (#28)
- [x] Comprehensive migration guide created (#29)
- [x] Test suite expanded to 129 tests (from 64)
- [x] Published v0.1.1 to crates.io

**Result**: Enhanced security, better error messages, and comprehensive test coverage

### Phase 3.3: Production Polish ✅ (Completed Nov 2025)

- [x] All 64 tests passing (later expanded to 129 in v0.1.1)
- [x] Zero clippy warnings, zero rustfmt violations
- [x] Security audit clean (0 vulnerabilities)
- [x] Published to crates.io (lumos-core, lumos-cli)
- [x] Organization migrated to getlumos
- [x] Homepage updated to lumos-lang.org
- [x] Comprehensive documentation
- [x] CI/CD pipeline with GitHub Actions
- [x] VSCode extension created (syntax highlighting, snippets)

### Phase 3.2: Enum Support ✅ (Completed Nov 2025)

- [x] Unit, Tuple, and Struct enum variants
- [x] Rust enum generation with sequential discriminants
- [x] TypeScript discriminated unions with `kind` field
- [x] Borsh `rustEnum()` integration

### Phase 3.1: Context-Aware Generation ✅ (Completed Nov 2025)

- [x] Anchor vs pure Borsh detection
- [x] Automatic import management
- [x] Smart derive macro handling

---

## Contributing

See an opportunity to help? Check our [Contributing Guide](CONTRIBUTING.md) or:

1. **Developers**: Claim an issue, submit a PR
2. **Language Designers**: Help design LUMOS syntax and semantics (Phase 7+)
3. **Content Creators**: Write tutorials, create videos
4. **Example Authors**: Build real-world schemas for awesome-lumos
5. **Community**: Test features, report bugs, suggest improvements

---

## How to Provide Feedback

- **Feature Requests**: Open a GitHub issue with label `enhancement`
- **Bug Reports**: Open a GitHub issue with label `bug`
- **Discussions**: Use GitHub Discussions for questions and ideas
- **Direct Contact**: Twitter [@getlumos](https://twitter.com/getlumos)

---

**This roadmap is a living document** - priorities may shift based on community feedback and ecosystem needs.

**Last Updated**: November 25, 2025

**Recent Updates**:
- Nov 25, 2025: **PHASE 6.2 TOOLING ECOSYSTEM COMPLETE** 🎉🎉🎉 - All 4 issues closed (#59-#62)
- Nov 25, 2025: **cargo lumos subcommand COMPLETE** (#59) - Full cargo integration with Cargo.toml config support
- Nov 25, 2025: **PHASE 5.3 ADVANCED TYPE SYSTEM COMPLETE** 🎉🎉🎉 - All 5 issues closed (#50-#54)
- Nov 25, 2025: **Generic type parameters COMPLETE** (#54) - Full `<T>`, `<K, V>` support in structs/enums with Rust + TypeScript generation
- Nov 24, 2025: **Module system foundations COMPLETE** (#53a) - Phase 5.3 at 65% 🚧 - AST & visibility support, split into sub-issues (#113-115)
- Nov 24, 2025: **Type aliases and imports COMPLETE** (#52) - Phase 5.3 at 60% 🎉 - Multi-file schemas with automatic import discovery
- Nov 24, 2025: **Fixed-size arrays COMPLETE** (#51) - Phase 5.3 at 40% 🎉 - Const generics support for `[T; N]` syntax
- Nov 24, 2025: **Custom derive macros support COMPLETE** (#50) - Phase 5.3 begins (20% complete) 🎉
- Nov 24, 2025: **PHASE 5.2 IDE INTEGRATION COMPLETE** 🎉🎉🎉 - All 5 editors supported!
- Nov 24, 2025: **Sublime Text package COMPLETE** (#49) - Full syntax + LSP + snippets 🎉
- Nov 24, 2025: **Emacs mode COMPLETE** (#48) - Phase 5.2 at 80% 🎉
- Nov 23, 2025: **Neovim plugin with Tree-sitter grammar COMPLETE** (#47) - Phase 5.2 at 60% 🎉
- Nov 23, 2025: **Published @getlumos/cli v0.1.0 to npm** (#62) - Phase 6.2 at 50% 🎉
- Nov 23, 2025: **Phase 5.1 Schema Evolution COMPLETE** 🎉 - All 5 issues closed (#40-#44)
- Nov 23, 2025: Added schema diff tool (#44) - `lumos diff` CLI command
- Nov 23, 2025: Added deprecation warnings (#43) - `#[deprecated]` attribute support
- Nov 23, 2025: Added backward compatibility validation (#42)
- Nov 23, 2025: Added automatic migration code generation (#41)
- Nov 23, 2025: Added schema versioning (#40)
- Nov 22, 2025: Published LSP v0.1.1 (#45) - Phase 5.2 at 20%
- Nov 2025: Completed Phase 6.3 Security & Validation (100%)
- Nov 2025: Published GitHub Action v1.0.0 (#60)
