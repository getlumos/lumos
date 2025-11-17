# CLAUDE.md - LUMOS Project Context

**Project:** LUMOS - Type-Safe Schema Language for Solana
**Owner:** RECTOR (rz1989s)
**Repository:** https://github.com/RECTOR-LABS/lumos
**Purpose:** AI assistant context and project knowledge base

---

## Project Overview

LUMOS is a **domain-specific language (DSL)** and **code generator** that enables type-safe, cross-language development for Solana blockchain applications. It bridges TypeScript (client-side) and Rust (on-chain programs) with guaranteed type safety and Borsh serialization compatibility.

### The Problem LUMOS Solves

When building Solana applications:
- **Manual duplication:** Developers write account structures twice (Rust for programs, TypeScript for clients)
- **Type drift:** Changes in Rust don't automatically reflect in TypeScript
- **Serialization errors:** Borsh schema mismatches cause runtime failures
- **Boilerplate:** Repetitive code for derives, imports, and serialization

### LUMOS Solution

Write schemas **once** in `.lumos` format, generate production-ready code for **both** languages:

```lumos
#[solana]
#[account]
struct UserAccount {
    wallet: PublicKey,
    balance: u64,
}
```

**Generates:**
- ✅ Rust with proper Anchor/Borsh integration
- ✅ TypeScript interfaces with Borsh schemas
- ✅ Guaranteed serialization compatibility

---

## Project Status

### Phase 1: Core Parser & Generators ✅ COMPLETED (2025-01-17)

**All 50 tests passing (100% success rate)**

- ✅ Parser: syn-based .lumos parser
- ✅ Rust Generator: Context-aware Anchor/Borsh code generation
- ✅ TypeScript Generator: Interface + Borsh schema generation
- ✅ E2E Testing: Actual compilation verification

### Phase 2: CLI & Developer Tools ✅ COMPLETED (2025-01-17)

**All 50 tests passing after CLI implementation**

- ✅ `lumos generate` - Generate Rust/TypeScript from schemas
- ✅ `lumos validate` - Validate .lumos schema syntax
- ✅ `lumos init` - Initialize new LUMOS project
- ✅ `lumos check` - Health check and diagnostics
- ✅ File I/O with proper error handling
- ✅ Colorized terminal output (colored crate)

### Phase 3.1: Enum Support ✅ COMPLETE (2025-11-17)

**64 tests passing (100% success rate) after Week 3**

**Week 1: AST & Parser ✅ COMPLETE**
- ✅ Enum syntax design (`examples/enums/schema.lumos` with 8 patterns)
- ✅ Complete design documentation (`docs/enum-design.md` 500+ lines)
- ✅ AST support for 3 enum variant types (Unit, Tuple, Struct)
- ✅ Parser implementation with full enum parsing
- ✅ 5 new parser tests for enum functionality

**Week 2: IR & Transform ✅ COMPLETE**
- ✅ Refactored IR to enum-based TypeDefinition (Struct|Enum)
- ✅ EnumDefinition and EnumVariantDefinition types
- ✅ Complete AST→IR transform for all variant types
- ✅ 3 new transform tests (unit/tuple/struct enums)
- ✅ Updated all generators for new IR structure
- ✅ All 57 tests passing

**Week 3: Code Generation ✅ COMPLETE**
- ✅ Rust enum generator implementation (all 3 variant types)
- ✅ TypeScript discriminated union generator with `kind` field
- ✅ Context-aware derives (Anchor vs Borsh detection)
- ✅ Borsh schema generation for enums (`borsh.rustEnum`)
- ✅ 6 new unit tests (3 Rust + 3 TypeScript)
- ✅ E2E test with enum compilation verification
- ✅ All 64 tests passing

**Week 4: Documentation & Polish ⏳ IN PROGRESS**
- ✅ Update CLAUDE.md with Phase 3.1 completion
- [ ] Update execution-plan.md with detailed enum implementation
- [ ] Real-world Solana instruction pattern testing
- [ ] Performance optimization (if needed)

### Phase 3.2: Advanced Features 📋 FUTURE

- VSCode extension
- Validation constraints
- Migration tooling
- Package publishing (crates.io, npm)

---

## Architecture

### Pipeline

```
.lumos file
    ↓
Parser (syn-based)
    ↓
AST (Abstract Syntax Tree)
    ↓
Transformer
    ↓
IR (Intermediate Representation)
    ↓
    ├─→ Rust Generator → .rs files
    └─→ TypeScript Generator → .ts files
```

### Key Design Decisions

#### 1. Rust-Style Syntax
- Uses `#[attribute]` syntax familiar to Solana developers
- Natural for target audience (primarily Rust developers)
- Leverages `syn` crate for parsing

#### 2. Context-Aware Generation
- **Smart Import Management:** Detects Anchor usage, uses appropriate imports
- **Intelligent Derives:** No manual derives for `#[account]`, context-aware for mixed modules
- **Prevents Conflicts:** Avoids Borsh import ambiguity in Anchor projects

#### 3. IR-Based Design
- Language-agnostic intermediate representation
- Easy to add new target languages (C++, Python, etc.)
- Separation of concerns: parsing ≠ generation

---

## Codebase Structure

```
lumos/
├── packages/
│   ├── core/                    # Core parser & generators (Rust)
│   │   ├── src/
│   │   │   ├── lib.rs           # Public API
│   │   │   ├── parser.rs        # .lumos → AST parser (enum support ✅)
│   │   │   ├── ast.rs           # AST definitions (Item enum: Struct|Enum ✅)
│   │   │   ├── transform.rs     # AST → IR transformer (enum transform ✅)
│   │   │   ├── ir.rs            # IR (TypeDefinition enum ✅)
│   │   │   ├── schema.rs        # Legacy schema parser
│   │   │   ├── error.rs         # Error types
│   │   │   └── generators/
│   │   │       ├── rust.rs      # Rust generator (enum gen pending)
│   │   │       └── typescript.rs # TS generator (enum gen pending)
│   │   └── tests/
│   │       ├── integration_test.rs        # Parser integration (5 tests)
│   │       ├── test_rust_generator.rs     # Rust gen tests (5 tests)
│   │       ├── test_typescript_generator.rs # TS gen tests (6 tests)
│   │       └── test_e2e.rs                # E2E compilation (8 tests)
│   └── cli/                     # CLI implementation (Phase 2 ✅)
│       ├── src/
│       │   ├── main.rs          # CLI entry point
│       │   └── commands/        # Command implementations
│       │       ├── generate.rs  # Code generation
│       │       ├── validate.rs  # Schema validation
│       │       ├── init.rs      # Project initialization
│       │       └── check.rs     # Health check
│       └── tests/               # CLI integration tests
├── examples/
│   ├── gaming/schema.lumos              # Gaming example
│   ├── nft-marketplace/schema.lumos     # NFT marketplace
│   ├── defi-staking/schema.lumos        # DeFi staking
│   ├── dao-governance/schema.lumos      # DAO governance
│   ├── token-vesting/schema.lumos       # Token vesting
│   └── enums/schema.lumos               # ✅ NEW: 8 enum patterns (200+ lines)
├── docs/
│   ├── execution-plan.md        # Development roadmap
│   └── enum-design.md           # ✅ NEW: Enum support design (500+ lines)
└── CLAUDE.md                    # This file (updated 2025-11-17)
```

---

## Test Suite

**Total:** 64/64 passing (100%)

| Suite | Count | Purpose |
|-------|-------|---------|
| Unit Tests | 39 | Core functionality (parser, generators, transform, AST) |
| Parser Integration | 5 | Real-world schema parsing |
| Rust Generator Integration | 8 | Rust code generation (structs + enums) |
| TypeScript Generator Integration | 9 | TypeScript code generation (structs + enums) |
| E2E Compilation | 9 | Actual Rust compilation with cargo check |

**New Tests (Phase 3.1 Weeks 1-3):**
- **Week 1:** 5 AST enum tests (unit/tuple/struct variants)
- **Week 2:** 3 Transform enum tests (full AST→IR pipeline)
- **Week 3:** 7 code generation tests (3 Rust + 3 TypeScript + 1 E2E enum)

**Run tests:** `cd packages/core && cargo test`

---

## Key Technical Achievements

### 1. Context-Aware Rust Generation

**Challenge:** Modules with mixed `#[account]` and non-`#[account]` structs caused import conflicts.

**Solution:**
- If **any** struct uses `#[account]` → use `anchor_lang::prelude::*` for entire module
- Non-account structs in Anchor modules use `AnchorSerialize/AnchorDeserialize`
- Pure Borsh modules use `borsh::{BorshSerialize, BorshDeserialize}`

### 2. Smart Derive Generation

**Insight:** Anchor's `#[account]` macro provides derives automatically.

**Implementation:**
- `#[account]` structs: NO manual derives
- Non-account in Anchor modules: `AnchorSerialize/AnchorDeserialize, Debug, Clone`
- Pure Borsh structs: `BorshSerialize/BorshDeserialize, Debug, Clone`

### 3. Type Mapping Excellence

Complete bidirectional type mapping:

| LUMOS | Rust | TypeScript | Borsh (Rust) | Borsh (TS) |
|-------|------|------------|--------------|------------|
| `u64` | `u64` | `number` | - | `borsh.u64` |
| `u128` | `u128` | `bigint` | - | `borsh.u128` |
| `Pubkey` | `Pubkey` | `PublicKey` | - | `borsh.publicKey` |
| `Signature` | `String` | `string` | - | `borsh.string` |
| `[T]` | `Vec<T>` | `T[]` | - | `borsh.vec(...)` |
| `Option<T>` | `Option<T>` | `T \| undefined` | - | `borsh.option(...)` |

### 4. Enum Support Architecture (Phase 3.1 Weeks 1-2)

**Challenge:** Support 3 different enum variant types with proper Rust/TypeScript mapping.

**Solution:**
- **AST Layer:** `Item` enum wrapping both `StructDef` and `EnumDef`
- **Parser:** Handles all 3 variant types:
  - Unit variants: `Active`, `Paused` (state machines)
  - Tuple variants: `PlayerJoined(PublicKey, u64)` (data-carrying)
  - Struct variants: `Initialize { authority: PublicKey }` (Solana instructions)
- **IR Layer:** Refactored to enum-based `TypeDefinition`:
  ```rust
  pub enum TypeDefinition {
      Struct(StructDefinition),
      Enum(EnumDefinition),
  }

  pub enum EnumVariantDefinition {
      Unit { name: String },
      Tuple { name: String, types: Vec<TypeInfo> },
      Struct { name: String, fields: Vec<FieldDefinition> },
  }
  ```
- **Transform:** Complete AST→IR pipeline for all variant types
- **Type Mapping Strategy:**
  - Rust: Native `enum` with derives
  - TypeScript: Discriminated unions with `kind` field for type narrowing
  - Borsh: Sequential discriminants (0, 1, 2...) matching Borsh defaults

**Status:** AST ✅ | Parser ✅ | IR ✅ | Transform ✅ | Code Generation ⏳

---

## Development Workflow

### Running Tests

```bash
cd packages/core
cargo test                 # Run all tests
cargo test --lib           # Unit tests only
cargo test --test test_e2e # E2E tests only
```

### Building

```bash
cd packages/core
cargo build                # Debug build
cargo build --release      # Release build
```

### Adding New Features

1. **Parser Changes:**
   - Modify `src/parser.rs` and `src/ast.rs`
   - Update `src/transform.rs` to handle new AST nodes
   - Add tests to `tests/integration_test.rs`

2. **Generator Changes:**
   - Modify `src/generators/rust.rs` or `typescript.rs`
   - Add unit tests in the same file (`#[cfg(test)] mod tests`)
   - Add integration tests in `tests/test_*_generator.rs`

3. **E2E Validation:**
   - Add test to `tests/test_e2e.rs`
   - Ensure generated code actually compiles

---

## Example Schemas

### 1. Gaming (`examples/gaming/schema.lumos`)
**Features:** Mixed structs (3 `#[account]` + 1 non-account)
```lumos
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    level: u16,
    experience: u64,
    equipped_items: [PublicKey],
}

#[solana]
struct MatchResult {
    player: PublicKey,
    opponent: Option<PublicKey>,
    score: u64,
}
```

### 2. NFT Marketplace (`examples/nft-marketplace/schema.lumos`)
**Features:** Signature type, optional fields
```lumos
#[solana]
struct PurchaseReceipt {
    buyer: PublicKey,
    nft_mint: PublicKey,
    price: u64,
    transaction_signature: Signature,  // Maps to String
}
```

### 3. Enum Patterns (`examples/enums/schema.lumos`)
**Features:** Comprehensive enum variant showcase (200+ lines, 8 patterns)

```lumos
// Unit enum (state machines)
#[solana]
enum GameState {
    Active,
    Paused,
    Finished,
    Cancelled,
}

// Tuple enum (data-carrying variants)
#[solana]
enum GameEvent {
    PlayerJoined(PublicKey),
    ScoreUpdated(PublicKey, u64),
}

// Struct enum (Solana instruction pattern)
#[solana]
enum GameInstruction {
    Initialize {
        authority: PublicKey,
        max_players: u32,
    },
    UpdateScore {
        player: PublicKey,
        new_score: u64,
    },
}

// Enums in structs
#[solana]
#[account]
struct GameAccount {
    authority: PublicKey,
    state: GameState,  // Enum as field
    current_round: u32,
}
```

**See:** `docs/enum-design.md` for complete design specification (500+ lines)

---

## Common Issues & Solutions

### Issue: Import Conflicts in Mixed Modules
**Symptom:** `error[E0659]: borsh is ambiguous`
**Solution:** Context-aware generation automatically handles this

### Issue: Derive Conflicts with #[account]
**Symptom:** `error[E0119]: conflicting implementations`
**Solution:** No derives for `#[account]` structs

### Issue: Signature Type Import Error
**Symptom:** `could not find signature in solana_program`
**Solution:** Map `Signature` → `String` (base58 representation)

---

## Dependencies

### Core (`packages/core/Cargo.toml`)
```toml
[dependencies]
syn = "2.0"           # Rust parser
quote = "1.0"         # Code generation
proc-macro2 = "1.0"   # Token manipulation
serde = "1.0"         # Serialization
serde_json = "1.0"    # JSON support
toml = "0.8"          # Config files
anyhow = "1.0"        # Error handling
thiserror = "1.0"     # Error macros

[dev-dependencies]
tempfile = "3.8"      # E2E test infrastructure
```

---

## Conventions & Standards

### Code Style
- **Rust:** Follow `rustfmt` defaults, 2-space indentation
- **Generated code:** Auto-formatted, idiomatic for target language
- **Comments:** Only for complex logic, avoid obvious explanations

### Naming
- **Functions:** `snake_case` (Rust convention)
- **Structs/Types:** `PascalCase` (both Rust and TypeScript)
- **Files:** `snake_case.rs`, `kebab-case.ts`

### Testing
- **Unit tests:** In same file as implementation (`#[cfg(test)] mod tests`)
- **Integration tests:** In `tests/` directory
- **E2E tests:** Must verify actual compilation, not just generation

### Git Workflow
- **Branch:** `dev` for development, `main` for releases
- **Commits:** Descriptive messages following conventional commits style
- **No pushing to main directly** - merge via PRs

---

## AI Assistant Guidelines

### When Working on LUMOS:

1. **Always run tests after changes:**
   ```bash
   cd packages/core && cargo test
   ```

2. **Check generated code compiles (E2E tests):**
   ```bash
   cargo test --test test_e2e
   ```

3. **Update this file** when:
   - Architecture changes
   - New features added
   - Test structure changes
   - New conventions established

4. **Update `docs/execution-plan.md`** when:
   - Phase milestones completed
   - New phases planned
   - Major technical decisions made

5. **Commit message format:**
   ```
   feat: Add enum support to parser
   fix: Resolve derive conflict in mixed modules
   test: Add E2E test for DAO governance
   docs: Update CLAUDE.md with new architecture
   ```

### What AI Should Know:

- **Context-aware generation is critical** - don't suggest reverting to simple Borsh imports
- **Always consider Anchor compatibility** - Solana developers use Anchor heavily
- **E2E tests must compile** - generation alone isn't enough
- **Type safety is paramount** - Rust ↔ TypeScript types must match exactly

---

## Future Considerations

### Phase 3.1 Week 3-4 (IN PROGRESS):
- ✅ Enum AST & Parser (Week 1 complete)
- ✅ Enum IR & Transform (Week 2 complete)
- ⏳ **Rust enum generator:** Native enum with derives (Week 3)
- ⏳ **TypeScript discriminated unions:** Type-safe unions with `kind` field (Week 3)
- ⏳ E2E tests with actual enum compilation (Week 3)
- ⏳ Real-world Solana instruction pattern testing (Week 4)
- ⏳ Performance optimization and polish (Week 4)

### Phase 3.2+ (FUTURE):
- **Custom derives:** User-specified derive macros beyond defaults
- **Validation constraints:** Min/max values, regex patterns, custom validators
- **VSCode extension:** Syntax highlighting, IntelliSense, schema validation
- **Watch mode:** Auto-regeneration on file changes
- **Configuration file:** `.lumosrc` for project-wide settings
- **Package publishing:** crates.io (Rust) and npm (TypeScript)

### Resolved Challenges:
- ✅ **Enum representation:** Solved via discriminated unions in TypeScript
- ✅ **IR flexibility:** Enum-based TypeDefinition supports extensibility
- ✅ **Borsh compatibility:** Sequential discriminants match Borsh defaults

### Remaining Challenges:
- **Generics:** May require significant IR changes for type parameters
- **Macros:** Complex to parse and generate custom proc macros
- **Breaking changes:** Need migration tooling for schema evolution

---

## Metrics & Success Criteria

### Phase 1 Success Criteria ✅ (2025-01-17)
- [x] Parse all 5 example schemas
- [x] Generate valid Rust code (verified via compilation)
- [x] Generate valid TypeScript code (syntax validation)
- [x] 100% test pass rate (50/50)
- [x] Context-aware generation working
- [x] E2E tests with actual compilation

### Phase 2 Success Criteria ✅ (2025-01-17)
- [x] CLI executable (`lumos` command)
- [x] File I/O working
- [x] Generate code to filesystem
- [x] 4 commands implemented (generate, validate, init, check)
- [x] Help documentation complete
- [x] Colorized output

### Phase 3.1 Success Criteria 🔄 (IN PROGRESS)

**Week 1 - AST & Parser ✅**
- [x] Enum syntax design with 8 comprehensive patterns
- [x] AST support for 3 enum variant types
- [x] Complete parser implementation
- [x] 5 new parser tests passing
- [x] Design documentation (500+ lines)

**Week 2 - IR & Transform ✅**
- [x] Enum-based TypeDefinition IR
- [x] EnumDefinition and EnumVariantDefinition types
- [x] Complete AST→IR transform for all variants
- [x] 3 new transform tests passing
- [x] All generators updated for new IR
- [x] 57/57 tests passing (100%)

**Week 3 - Code Generation ✅**
- [x] Rust enum generator with context-aware derives
- [x] TypeScript discriminated union generator with `kind` field
- [x] Borsh schema generation for enums (`borsh.rustEnum`)
- [x] 6 new unit tests (3 Rust + 3 TypeScript)
- [x] E2E compilation test with enums
- [x] 64/64 tests passing (100%)

**Week 4 - Polish ⏳ (IN PROGRESS)**
- [x] CLAUDE.md documentation updates
- [ ] execution-plan.md comprehensive update
- [ ] Real-world Solana instruction pattern validation
- [ ] Performance optimization (if needed)

---

## Team & Ownership

**Primary Developer:** RECTOR (rz1989s)
**Organization:** RECTOR-LABS
**Repository:** https://github.com/RECTOR-LABS/lumos
**License:** MIT or Apache 2.0 (dual-licensed)

---

## Resources

- **Anchor Framework:** https://www.anchor-lang.com/
- **Borsh Specification:** https://borsh.io/
- **Solana Docs:** https://docs.solana.com/
- **syn crate:** https://docs.rs/syn/
- **Project Execution Plan:** `docs/execution-plan.md`

---

**Last Updated:** 2025-11-17 (Phase 3.1 Week 3 - Enum Code Generation complete)
**Next Update:** When Phase 3.1 Week 4 (Documentation & Polish) completes
