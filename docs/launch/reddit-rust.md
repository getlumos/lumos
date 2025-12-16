# Reddit Post: r/rust

---

## Title

LUMOS: A Rust-based schema compiler for cross-language code generation (syn + custom DSL → Rust/TypeScript/Python/Go/Ruby)

---

## Post Content

Hi r/rust,

I wanted to share a project we've been working on: **LUMOS**, a schema language compiler built entirely in Rust that generates code for multiple target languages.

### The Technical Problem

In blockchain development (specifically Solana), you often need identical data structures in both Rust (on-chain programs) and TypeScript (frontend clients). Manual synchronization is error-prone—field order matters for serialization, and a single mismatch causes runtime failures.

### The Solution

LUMOS provides a custom DSL for defining schemas:

```rust
#[solana]
#[account]
struct UserAccount {
    wallet: PublicKey,
    balance: u64,
    items: [PublicKey],
    metadata: Option<String>,
}
```

The compiler then generates idiomatic code for each target language with guaranteed serialization compatibility.

### Architecture

```
.lumos file → Parser (syn) → AST → Transform → IR → Generators → Output
```

**Key design decisions:**

1. **syn for parsing**: We leverage `syn` to parse a Rust-like syntax. This gives us familiar syntax while allowing domain-specific extensions (`#[solana]`, `#[account]`).

2. **IR-based architecture**: Language-agnostic intermediate representation decouples parsing from generation. Adding new target languages just means writing a new generator.

3. **Context-aware generation**: The Rust generator detects whether schemas use Anchor framework features and adjusts imports/derives accordingly.

### Implementation Details

- **Parser** (~800 lines): Uses `syn::parse::Parse` with custom item types
- **IR** (~400 lines): Normalized representation with `TypeInfo`, `StructDefinition`, `EnumDefinition`
- **Rust generator** (~600 lines): Handles Anchor integration, derive selection, module awareness
- **TypeScript generator** (~700 lines): Generates interfaces + Borsh schema definitions

**Interesting challenges:**

- **Derive conflicts**: Anchor's `#[account]` macro provides Borsh derives. Adding manual derives causes compilation errors. We detect this and skip conflicting derives.

- **Enum discriminants**: Borsh uses sequential u8 discriminants. TypeScript needs discriminated unions with a `kind` field. The generator handles this mapping.

- **Generic types**: Supporting `struct Response<T>` required threading generic context through the entire pipeline.

### Language Server

We also built an LSP server (`lumos-lsp`) providing:
- Real-time diagnostics
- Completion for types and attributes
- Hover information

The LSP reuses the core parsing logic, so diagnostics are always accurate.

### Numbers

- 322 tests (unit + integration + E2E compilation tests)
- ~15,000 lines of Rust
- Generates valid, compilable code for all target languages
- Published to crates.io

### Code

- GitHub: https://github.com/getlumos/lumos
- crates.io: https://crates.io/crates/lumos-core

### Questions for r/rust

1. We're using `syn` to parse a custom DSL that's "Rust-like but not Rust." Is there a better approach for this use case?

2. The IR uses a lot of `Vec<String>` for things like generic parameters. Would interning help here, or is it premature optimization?

3. Any suggestions for the generator architecture? Currently each generator is a standalone module—considering a trait-based approach.

Would love feedback from the community!

---

## Post Guidelines

- Focus on technical implementation, not marketing
- Be genuinely curious about feedback
- Acknowledge limitations and areas for improvement
- Respond thoughtfully to technical criticism
- Link to specific code when discussing implementation details
