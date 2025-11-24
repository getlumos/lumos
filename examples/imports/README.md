# Multi-File Imports Example

This example demonstrates **LUMOS's multi-file import system**, allowing you to organize large schemas across multiple files with shared type definitions.

## 📁 File Structure

```
imports/
├── types.lumos          # Common type aliases and enums
├── accounts.lumos       # Account definitions (imports from types.lumos)
├── instructions.lumos   # Instruction enums (imports from types.lumos)
└── README.md           # This file
```

## 🎯 What This Demonstrates

### 1. **Shared Type Definitions** (`types.lumos`)
- Common type aliases (UserId, Lamports, TokenAmount, etc.)
- Shared enums (AccountStatus, PermissionLevel)
- Single source of truth for type definitions

### 2. **Cross-File Imports** (`accounts.lumos`, `instructions.lumos`)
- JavaScript-style import syntax
- Multiple imports from same file
- Automatic dependency resolution

### 3. **Circular Import Detection**
- LUMOS prevents circular dependencies
- Files can't import from each other in a cycle

## 🚀 Usage

### Generate Code from Entry Point

```bash
# Generate from accounts.lumos (will auto-discover types.lumos)
lumos generate imports/accounts.lumos --output generated/

# Generate from instructions.lumos (will also discover types.lumos)
lumos generate imports/instructions.lumos --output generated/
```

### What Gets Generated

#### Rust Output (`generated.rs`)
```rust
// From types.lumos
pub type UserId = solana_program::pubkey::Pubkey;
pub type Lamports = u64;
pub type TokenAmount = u64;
// ... all type aliases

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum AccountStatus {
    Uninitialized,
    Active,
    Frozen,
    Closed,
}

// From accounts.lumos
#[account]
pub struct UserAccount {
    pub user_id: UserId,           // Expands to Pubkey
    pub balance: Lamports,         // Expands to u64
    pub token_balance: TokenAmount, // Expands to u64
    pub status: AccountStatus,
    // ...
}
```

#### TypeScript Output (`generated.ts`)
```typescript
// From types.lumos
import { PublicKey } from '@solana/web3.js';

export type UserId = PublicKey;
export type Lamports = number;
export type TokenAmount = number;
// ... all type aliases

export enum AccountStatus {
    Uninitialized = 0,
    Active = 1,
    Frozen = 2,
    Closed = 3,
}

// From accounts.lumos
export interface UserAccount {
    user_id: UserId;           // = PublicKey
    balance: Lamports;         // = number
    token_balance: TokenAmount; // = number
    status: AccountStatus;
    // ...
}
```

## 📝 Import Syntax

### Basic Import
```rust
import { UserId, Lamports } from "./types.lumos";
```

### Multiple Imports
```rust
import {
    UserId,
    WalletAddress,
    Lamports,
    TokenAmount,
    AccountStatus
} from "./types.lumos";
```

### Relative Paths
```rust
// Same directory
import { Type } from "./types.lumos";

// Parent directory
import { Type } from "../types.lumos";

// Subdirectory
import { Type } from "./shared/types.lumos";
```

### Extension Optional
```rust
// Both work
import { UserId } from "./types.lumos";
import { UserId } from "./types";
```

## 🎓 Key Concepts

### Type Resolution Order
1. **Load**: LUMOS loads entry file and discovers imports
2. **Parse**: All imported files are parsed
3. **Collect**: Type aliases from ALL files are collected
4. **Resolve**: Aliases are resolved recursively
5. **Transform**: All files are transformed to IR with shared context
6. **Generate**: Single output with all types

### Circular Import Detection
```rust
// ❌ This will fail
// a.lumos
import { B } from "./b.lumos";
struct A { b: B }

// b.lumos
import { A } from "./a.lumos";
struct B { a: A }

// Error: Circular import detected: a.lumos -> b.lumos -> a.lumos
```

### Type Alias Resolution
```rust
// types.lumos
type UserId = PublicKey;
type AccountId = UserId;       // Alias of an alias
type Creator = AccountId;      // Nested alias

// accounts.lumos
import { Creator } from "./types.lumos";

struct NFT {
    creator: Creator,  // Fully resolved to PublicKey
}
```

## ✅ Benefits

### 1. **Better Organization**
- Separate concerns: types, accounts, instructions
- Smaller, focused files
- Easier to navigate large projects

### 2. **Reusability**
- Define common types once
- Import where needed
- No duplication

### 3. **Consistency**
- Same type names across entire project
- Single source of truth
- Refactor once, updates everywhere

### 4. **Type Safety**
- Import validation at compile time
- Undefined type errors caught early
- Circular dependency detection

### 5. **Team Collaboration**
- Multiple developers can work on different files
- Merge conflicts reduced
- Clear module boundaries

## 🏗️ Project Structure Best Practices

### Small Projects
```
project/
└── schema.lumos          # Single file is fine
```

### Medium Projects
```
project/
├── types.lumos           # Common types
└── main.lumos            # Main definitions
```

### Large Projects
```
project/
├── types/
│   ├── common.lumos      # Shared types
│   ├── tokens.lumos      # Token types
│   └── governance.lumos  # DAO types
├── accounts/
│   ├── user.lumos
│   ├── vault.lumos
│   └── dao.lumos
└── instructions/
    ├── token.lumos
    ├── staking.lumos
    └── governance.lumos
```

## 🐛 Common Issues

### Import Not Found
```
Error: Failed to read file './types.lumos': No such file or directory
```
**Solution**: Check relative path is correct from the importing file.

### Undefined Type
```
Error: Import error in 'accounts.lumos': type 'InvalidType' not found in './types.lumos'
```
**Solution**: Make sure the type is defined in types.lumos and is not just imported there.

### Circular Import
```
Error: Circular import detected: a.lumos -> b.lumos -> a.lumos
```
**Solution**: Restructure to move shared types to a third file.

## 📚 Related Examples

- [`../type_aliases.lumos`](../type_aliases.lumos) - Single-file type alias patterns
- [`../custom_derives.lumos`](../custom_derives.lumos) - Custom derive macros
- [`../enums/`](../enums/) - Enum variant patterns

## 💡 Next Steps

1. Try generating code from these examples
2. Modify types.lumos and see changes propagate
3. Create your own multi-file project structure
4. Add more modules following these patterns

## 🤝 Contributing

Found an issue or have a suggestion? Please open an issue at:
https://github.com/getlumos/lumos/issues
