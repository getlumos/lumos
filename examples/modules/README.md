# Rust-Style Modules Example

This example demonstrates LUMOS's Rust-style module system for organizing larger projects with hierarchical structure.

## Project Structure

```
modules/
├── simple/              # Basic 2-file module example
│   ├── main.lumos       # Entry point with `mod types;`
│   └── types.lumos      # Shared types
│
├── nested/              # 3-level hierarchy example
│   ├── main.lumos       # Entry point with `mod models;`
│   ├── models.lumos     # Parent module with `mod user; mod account;`
│   └── models/
│       ├── user.lumos   # User model
│       └── account.lumos # Account model
│
└── visibility/          # Public/private visibility example
    ├── main.lumos       # Entry point
    ├── api.lumos        # Public types
    └── internal.lumos   # Private types
```

## Key Concepts

### 1. Module Declaration

Declare modules with `mod`:

```rust
// main.lumos
mod types;       // Loads ./types.lumos
mod models;      // Loads ./models.lumos or ./models/mod.lumos
```

### 2. Module Resolution

LUMOS looks for modules in two locations:

1. **Sibling file**: `./module_name.lumos`
2. **Directory module**: `./module_name/mod.lumos`

### 3. Use Statements

Import types from other modules:

```rust
// Import from root module
use crate::types::UserId;

// Import from parent module
use super::Timestamp;

// Import from current module
use self::helpers::validate;
```

### 4. Visibility

Control type visibility:

```rust
// Public - accessible from other modules
pub struct PublicUser {
    wallet: PublicKey,
}

// Private - only accessible within this module (default)
struct InternalState {
    secret: [u8],
}

// Crate-visible (like Rust's pub(crate))
pub(crate) struct CrateInternal {
    data: u64,
}
```

## Examples

### Simple (2 files)

```rust
// simple/main.lumos
mod types;

#[solana]
#[account]
struct UserProfile {
    wallet: PublicKey,
    role: u8,
}

// simple/types.lumos
type UserId = u64;

#[solana]
enum UserRole {
    Guest,
    Member,
    Admin,
}
```

### Nested (3 levels)

```rust
// nested/main.lumos
mod models;

// nested/models.lumos
mod user;
mod account;

type Timestamp = i64;

// nested/models/user.lumos
#[solana]
#[account]
struct User {
    wallet: PublicKey,
    username: String,
}
```

### Visibility

```rust
// visibility/api.lumos
pub struct PublicConfig {
    name: String,
    enabled: bool,
}

// visibility/internal.lumos
pub(crate) struct InternalState {
    secret_key: [u8],
}
```

## Generate Code

```bash
# Generate from entry point
lumos generate simple/main.lumos
lumos generate nested/main.lumos
lumos generate visibility/main.lumos

# Validate module structure
lumos validate nested/main.lumos --verbose
```

## Module vs Import: When to Use Which?

| Feature | JavaScript Imports | Rust Modules |
|---------|-------------------|--------------|
| Syntax | `import { Type } from "./file.lumos"` | `mod name; use crate::path::Type;` |
| Best for | Small projects, explicit dependencies | Large projects, hierarchical organization |
| Visibility | All imports are public | Fine-grained with `pub`/`pub(crate)` |
| Organization | Flat structure | Hierarchical structure |

### Recommendation

- **Small projects (< 5 files)**: Use JavaScript-style imports
- **Medium projects (5-15 files)**: Either works well
- **Large projects (15+ files)**: Use Rust-style modules for organization

## Related

- [Multi-File Schemas Guide](https://lumos-lang.org/guides/multi-file-schemas)
- [examples/imports/](../imports/) - JavaScript-style import examples
