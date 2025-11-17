# LUMOS Architecture

## Overview

LUMOS follows a pipeline architecture:

```
Schema (TOML)
    ↓
Parser
    ↓
Intermediate Representation (IR)
    ↓
Code Generators
    ├─→ Rust Generator
    └─→ TypeScript Generator
```

## Components

### 1. Schema Parser

**Responsibility:** Parse TOML schema files into structured data

**Location:** `packages/core/src/schema.rs`

**Input:** TOML string
**Output:** `Schema` struct

### 2. Intermediate Representation (IR)

**Responsibility:** Language-agnostic representation of types

**Location:** `packages/core/src/ir.rs`

**Key Types:**
- `TypeDefinition` - Represents a type
- `FieldDefinition` - Represents a field
- `TypeInfo` - Type information
- `Metadata` - Additional attributes

### 3. Code Generators

**Responsibility:** Transform IR into target language code

**Locations:**
- Rust: `packages/core/src/generators/rust.rs`
- TypeScript: `packages/core/src/generators/typescript.rs`

**Input:** `TypeDefinition`
**Output:** Source code string

### 4. CLI

**Responsibility:** User interface for LUMOS

**Location:** `packages/cli/src/main.rs`

**Commands:**
- `init` - Create new project
- `build` - Generate code
- `watch` - Watch for changes

## Design Principles

### 1. Extensibility

The generator architecture allows adding new target languages easily:

```rust
pub trait CodeGenerator {
    fn generate(&self, type_def: &TypeDefinition) -> String;
}
```

### 2. Type Safety

Strong typing throughout the pipeline ensures correctness.

### 3. Separation of Concerns

- Parser knows nothing about code generation
- Generators know nothing about schema format
- IR bridges the gap

## Future Architecture

### Phase 2: Custom Syntax

```
.lumos files
    ↓
Custom Parser (syn-based)
    ↓
IR (same)
    ↓
Generators (same)
```

### Phase 3: Plugin System

```
IR
  ├─→ Rust Generator
  ├─→ TypeScript Generator
  ├─→ Go Generator (plugin)
  └─→ Python Generator (plugin)
```

## Performance Considerations

- Lazy parsing (parse only when needed)
- Incremental builds (track changes)
- Parallel generation (multiple files at once)

## Error Handling

Errors are categorized:
- `SchemaParse` - Invalid schema
- `CodeGen` - Generation failed
- `Io` - File system errors

All errors provide actionable messages for users.
