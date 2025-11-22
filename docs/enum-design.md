# Enum Support Design Document

**Status:** Design Phase
**Target:** Phase 3.1
**Author:** RECTOR
**Date:** 2025-01-17

---

## Overview

This document outlines the design and implementation strategy for adding enum support to LUMOS. Enums are critical for Solana programs (instruction variants, state machines, error codes) and represent the most important missing feature.

---

## Syntax Design

### 1. Unit Enums (Simple State Machines)

**LUMOS Syntax:**
```rust
#[solana]
enum GameState {
    Inactive,
    Active,
    Paused,
    Finished,
}
```

**Generated Rust:**
```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum GameState {
    Inactive,
    Active,
    Paused,
    Finished,
}
```

**Generated TypeScript:**
```typescript
export type GameState =
    | { kind: 'Inactive' }
    | { kind: 'Active' }
    | { kind: 'Paused' }
    | { kind: 'Finished' };

export const GameStateSchema = borsh.enum([
    borsh.unit('Inactive'),
    borsh.unit('Active'),
    borsh.unit('Paused'),
    borsh.unit('Finished'),
]);
```

---

### 2. Tuple Variants (Data-Carrying Variants)

**LUMOS Syntax:**
```rust
#[solana]
enum GameEvent {
    PlayerJoined(PublicKey),
    ScoreUpdated(PublicKey, u64),
    ItemAcquired(PublicKey, u64, String),
}
```

**Generated Rust:**
```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum GameEvent {
    PlayerJoined(Pubkey),
    ScoreUpdated(Pubkey, u64),
    ItemAcquired(Pubkey, u64, String),
}
```

**Generated TypeScript:**
```typescript
export type GameEvent =
    | { kind: 'PlayerJoined'; value: [PublicKey] }
    | { kind: 'ScoreUpdated'; value: [PublicKey, number] }
    | { kind: 'ItemAcquired'; value: [PublicKey, number, string] };

export const GameEventSchema = borsh.enum([
    borsh.tuple('PlayerJoined', [borsh.publicKey()]),
    borsh.tuple('ScoreUpdated', [borsh.publicKey(), borsh.u64()]),
    borsh.tuple('ItemAcquired', [borsh.publicKey(), borsh.u64(), borsh.string()]),
]);
```

---

### 3. Struct Variants (Instruction Pattern)

**LUMOS Syntax:**
```rust
#[solana]
enum GameInstruction {
    Initialize {
        authority: PublicKey,
        max_players: u8,
    },

    UpdateScore {
        player: PublicKey,
        score: u64,
    },

    Terminate,
}
```

**Generated Rust:**
```rust
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum GameInstruction {
    Initialize {
        authority: Pubkey,
        max_players: u8,
    },

    UpdateScore {
        player: Pubkey,
        score: u64,
    },

    Terminate,
}
```

**Generated TypeScript:**
```typescript
export type GameInstruction =
    | {
        kind: 'Initialize';
        authority: PublicKey;
        max_players: number;
      }
    | {
        kind: 'UpdateScore';
        player: PublicKey;
        score: number;
      }
    | { kind: 'Terminate' };

export const GameInstructionSchema = borsh.enum([
    borsh.struct('Initialize', [
        borsh.publicKey('authority'),
        borsh.u8('max_players'),
    ]),
    borsh.struct('UpdateScore', [
        borsh.publicKey('player'),
        borsh.u64('score'),
    ]),
    borsh.unit('Terminate'),
]);
```

---

## Type Mapping Strategy

### Borsh Serialization Format

Borsh encodes enums with:
1. **Discriminant (u8)**: Variant index (0, 1, 2, ...)
2. **Variant data**: Serialized according to variant type

**Example:**
```rust
enum Example {
    A,              // Discriminant 0, no data
    B(u64),         // Discriminant 1, + u64 data
    C { x: u32 },   // Discriminant 2, + u32 data
}
```

**Serialization:**
- `Example::A` → `[0]` (1 byte)
- `Example::B(42)` → `[1, 42, 0, 0, 0, 0, 0, 0, 0]` (1 + 8 bytes)
- `Example::C { x: 100 }` → `[2, 100, 0, 0, 0]` (1 + 4 bytes)

### TypeScript Discriminated Union Pattern

TypeScript discriminated unions use a `kind` field for type narrowing:

```typescript
type Example =
    | { kind: 'A' }
    | { kind: 'B'; value: number }
    | { kind: 'C'; x: number };

// Type narrowing works:
function process(e: Example) {
    if (e.kind === 'A') {
        // TypeScript knows: e is { kind: 'A' }
    } else if (e.kind === 'B') {
        // TypeScript knows: e.value is number
        console.log(e.value);
    }
}
```

---

## Implementation Plan

### Phase 1: AST & Parser (Week 1)

**Files to Modify:**
1. `packages/core/src/ast.rs`
2. `packages/core/src/parser.rs`

**AST Additions:**
```rust
// In ast.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Struct(StructDef),
    Enum(EnumDef),  // NEW
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub attributes: Vec<String>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnumVariant {
    Unit {
        name: String,
    },
    Tuple {
        name: String,
        types: Vec<Type>,
    },
    Struct {
        name: String,
        fields: Vec<Field>,
    },
}
```

**Parser Implementation:**
- Detect `enum` keyword
- Parse variant types
- Handle mixed variant types in single enum

---

### Phase 2: IR & Transform (Week 2)

**Files to Modify:**
1. `packages/core/src/ir.rs`
2. `packages/core/src/transform.rs`

**IR Additions:**
```rust
// In ir.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeDefinition {
    Struct(StructDefinition),
    Enum(EnumDefinition),  // NEW
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumDefinition {
    pub name: String,
    pub is_solana: bool,
    pub variants: Vec<EnumVariantDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnumVariantDef {
    Unit { name: String },
    Tuple { name: String, types: Vec<FieldType> },
    Struct { name: String, fields: Vec<(String, FieldType)> },
}
```

**Transform Logic:**
- Convert `ast::EnumDef` → `ir::EnumDefinition`
- Map variant types correctly
- Preserve attributes

---

### Phase 3: Rust Generator (Week 3)

**Files to Modify:**
1. `packages/core/src/generators/rust.rs`

**Generation Strategy:**

```rust
fn generate_enum(enum_def: &EnumDefinition) -> String {
    let mut code = String::new();

    // Derives
    code.push_str("#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]\n");

    // Enum declaration
    code.push_str(&format!("pub enum {} {{\n", enum_def.name));

    // Variants
    for variant in &enum_def.variants {
        match variant {
            EnumVariantDef::Unit { name } => {
                code.push_str(&format!("    {},\n", name));
            }
            EnumVariantDef::Tuple { name, types } => {
                let type_list = types.iter()
                    .map(|t| map_rust_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                code.push_str(&format!("    {}({}),\n", name, type_list));
            }
            EnumVariantDef::Struct { name, fields } => {
                code.push_str(&format!("    {} {{\n", name));
                for (field_name, field_type) in fields {
                    code.push_str(&format!("        pub {}: {},\n",
                        field_name, map_rust_type(field_type)));
                }
                code.push_str("    },\n");
            }
        }
    }

    code.push_str("}\n");
    code
}
```

**Context-Aware Derives:**
- If module has `#[account]` structs → use `AnchorSerialize/AnchorDeserialize`
- Pure Borsh modules → use `BorshSerialize/BorshDeserialize`

---

### Phase 4: TypeScript Generator (Week 4)

**Files to Modify:**
1. `packages/core/src/generators/typescript.rs`

**Generation Strategy:**

```rust
fn generate_enum_type(enum_def: &EnumDefinition) -> String {
    let mut variants = Vec::new();

    for variant in &enum_def.variants {
        match variant {
            EnumVariantDef::Unit { name } => {
                variants.push(format!("  | {{ kind: '{}' }}", name));
            }
            EnumVariantDef::Tuple { name, types } => {
                let type_list = types.iter()
                    .map(|t| map_typescript_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                variants.push(format!("  | {{ kind: '{}'; value: [{}] }}",
                    name, type_list));
            }
            EnumVariantDef::Struct { name, fields } => {
                let mut field_defs = vec![format!("kind: '{}'", name)];
                for (field_name, field_type) in fields {
                    field_defs.push(format!("{}: {}",
                        field_name, map_typescript_type(field_type)));
                }
                variants.push(format!("  | {{ {} }}", field_defs.join("; ")));
            }
        }
    }

    format!("export type {} =\n{};",
        enum_def.name, variants.join("\n"))
}

fn generate_enum_schema(enum_def: &EnumDefinition) -> String {
    let mut schema_variants = Vec::new();

    for variant in &enum_def.variants {
        match variant {
            EnumVariantDef::Unit { name } => {
                schema_variants.push(format!("  borsh.unit('{}')", name));
            }
            EnumVariantDef::Tuple { name, types } => {
                let type_schemas = types.iter()
                    .map(|t| map_borsh_type(t))
                    .collect::<Vec<_>>()
                    .join(", ");
                schema_variants.push(format!("  borsh.tuple('{}', [{}])",
                    name, type_schemas));
            }
            EnumVariantDef::Struct { name, fields } => {
                let field_schemas = fields.iter()
                    .map(|(fname, ftype)| {
                        format!("borsh.{}('{}')",
                            map_borsh_type(ftype), fname)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                schema_variants.push(format!("  borsh.struct('{}', [{}])",
                    name, field_schemas));
            }
        }
    }

    format!("export const {}Schema = borsh.enum([\n{}\n]);",
        enum_def.name, schema_variants.join(",\n"))
}
```

---

## Testing Strategy

### Unit Tests

**Parser Tests** (`packages/core/src/parser.rs`):
- ✅ Parse unit enum
- ✅ Parse tuple enum
- ✅ Parse struct enum
- ✅ Parse mixed enum
- ✅ Parse enum with attributes
- ✅ Error on invalid syntax

**Transform Tests** (`packages/core/src/transform.rs`):
- ✅ Transform unit enum to IR
- ✅ Transform tuple enum to IR
- ✅ Transform struct enum to IR
- ✅ Preserve enum attributes

**Rust Generator Tests** (`packages/core/src/generators/rust.rs`):
- ✅ Generate unit enum
- ✅ Generate tuple enum
- ✅ Generate struct enum
- ✅ Context-aware derives
- ✅ Proper type mapping

**TypeScript Generator Tests** (`packages/core/src/generators/typescript.rs`):
- ✅ Generate discriminated union type
- ✅ Generate Borsh enum schema
- ✅ All variant types
- ✅ Proper type mapping

### Integration Tests

**E2E Compilation Tests** (`packages/core/tests/test_e2e.rs`):
- ✅ Compile Rust enum code with `cargo check`
- ✅ Validate TypeScript enum syntax
- ✅ Test instruction enum pattern
- ✅ Test state machine enum pattern
- ✅ Test error code enum pattern

### Example Schemas

Use `examples/enums/schema.lumos` for comprehensive testing:
- 8 different enum patterns
- All variant types covered
- Real-world Solana patterns (instructions, state machines, errors)

---

## Edge Cases to Handle

1. **Empty enums** - Should error
2. **Enums with single variant** - Valid, should generate
3. **Nested enums** - Not supported initially (future enhancement)
4. **Recursive enums** - Not supported initially
5. **Generic enums** - Not supported initially
6. **Enums as struct fields** - Must support (critical use case)
7. **Option<Enum>** - Must support
8. **[Enum]** (Vec of enums) - Must support

---

## Borsh Compatibility Notes

### Rust Borsh Enum Serialization

Borsh automatically implements serialization for Rust enums:
```rust
#[derive(BorshSerialize, BorshDeserialize)]
enum Example {
    A,
    B(u64),
}
```

### TypeScript Borsh Enum Deserialization

The `@coral-xyz/borsh` library provides:
```typescript
borsh.enum([
    borsh.unit('A'),
    borsh.struct('B', [borsh.u64('value')]),
])
```

**Critical:** Discriminant order must match between Rust and TypeScript!

---

## Migration Strategy

### Breaking Changes

Adding enum support does NOT break existing code:
- Existing struct schemas continue to work
- No changes to struct generation
- Enums are purely additive

### Versioning

- Current: v0.0.1 (Phase 1 + 2)
- With enums: v0.1.0 (Phase 3.1)
- Enum support is a minor version bump

---

## Success Criteria

### Minimum Viable Enum Support (v0.1.0)

- ✅ Parse all three variant types (unit, tuple, struct)
- ✅ Generate valid Rust enums with Borsh derives
- ✅ Generate TypeScript discriminated unions
- ✅ Generate correct Borsh schemas for TypeScript
- ✅ All E2E tests pass (Rust compiles, TypeScript valid)
- ✅ Documentation updated with enum examples
- ✅ At least 15 new tests passing

### Future Enhancements (v0.2.0+)

- ⏳ Generic enums (e.g., `Result<T, E>`)
- ⏳ Nested enums
- ⏳ Custom discriminant values (`= 5`)
- ⏳ Enum helper methods (is_variant(), as_variant())
- ⏳ Pattern matching helpers in TypeScript

---

## Implementation Checklist

**Week 1: Parser & AST**
- [ ] Update `ast.rs` with enum types
- [ ] Implement enum parser in `parser.rs`
- [ ] Add parser unit tests
- [ ] Test with `examples/enums/schema.lumos`

**Week 2: IR & Transform**
- [ ] Update `ir.rs` with enum definitions
- [ ] Implement AST → IR transform
- [ ] Add transform unit tests
- [ ] Verify IR correctness

**Week 3: Rust Generator**
- [ ] Implement enum generation in `rust.rs`
- [ ] Context-aware derive selection
- [ ] Add generator unit tests
- [ ] E2E Rust compilation tests

**Week 4: TypeScript Generator**
- [ ] Implement discriminated union generation
- [ ] Implement Borsh schema generation
- [ ] Add generator unit tests
- [ ] E2E TypeScript validation tests

**Week 5: Testing & Documentation**
- [ ] Comprehensive integration tests
- [ ] Update README.md with enum examples
- [ ] Update execution-plan.md
- [ ] Create tutorial for enum usage
- [ ] Commit and push enum support

---

## Questions & Decisions

### Q1: Should we support custom discriminant values?

```rust
enum Status {
    Active = 1,
    Inactive = 2,
}
```

**Decision:** Not in v0.1.0. Use sequential discriminants (0, 1, 2, ...) to match Borsh default. Can add in v0.2.0 if needed.

### Q2: How to handle enum imports in TypeScript?

**Decision:** Import discriminated union types directly:
```typescript
import { GameState, GameStateSchema } from './generated';
```

### Q3: Should enums support `#[account]` attribute?

**Decision:** No. `#[account]` is for structs only. Enums cannot be Solana accounts themselves, but can be fields in account structs.

### Q4: TypeScript enum helpers?

**Decision:** Generate type guards in v0.2.0:
```typescript
export function isGameStateActive(state: GameState): boolean {
    return state.kind === 'Active';
}
```

---

## Related Documents

- `examples/enums/schema.lumos` - Comprehensive enum examples
- `docs/execution-plan.md` - Phase 3 timeline
- `README.md` - User-facing documentation

---

**Status:** Design Complete - Ready for Implementation
**Next Step:** Begin AST & Parser implementation (Week 1)
