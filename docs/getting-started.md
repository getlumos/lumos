# Getting Started with LUMOS

> **Note:** LUMOS is currently in early development (Phase 1). This guide describes the planned functionality.

## Installation

```bash
cargo install lumos-cli
```

## Quick Start

### 1. Create a new project

```bash
lumos init my-solana-project
cd my-solana-project
```

This creates:
```
my-solana-project/
├── lumos.toml          # Your schema definitions
└── generated/          # Generated code will appear here
```

### 2. Define your schema

Edit `lumos.toml`:

```toml
[schema]
name = "UserAccount"
solana = true

[[schema.fields]]
name = "id"
type = "u64"

[[schema.fields]]
name = "wallet"
type = "PublicKey"

[[schema.fields]]
name = "balance"
type = "u64"
```

### 3. Generate code

```bash
lumos build
```

This generates:

**Rust** (`generated/rust/lib.rs`):
```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserAccount {
    pub id: u64,
    pub wallet: Pubkey,
    pub balance: u64,
}
```

**TypeScript** (`generated/typescript/types.ts`):
```typescript
export interface UserAccount {
  id: number;
  wallet: PublicKey;
  balance: number;
}
```

### 4. Use in your project

**In your Anchor program:**
```rust
use crate::generated::UserAccount;

#[program]
pub mod my_program {
    use super::*;

    pub fn create_user(ctx: Context<CreateUser>, id: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        user.id = id;
        user.balance = 0;
        Ok(())
    }
}
```

**In your frontend:**
```typescript
import { UserAccount } from './generated/types';

const user: UserAccount = await program.account.userAccount.fetch(
  userPubkey
);
console.log(`Balance: ${user.balance}`);
```

## Type Mappings

| LUMOS Type | Rust Type | TypeScript Type |
|-----------|-----------|-----------------|
| `u64` | `u64` | `number` |
| `i64` | `i64` | `number` |
| `string` | `String` | `string` |
| `bool` | `bool` | `boolean` |
| `PublicKey` | `Pubkey` | `PublicKey` |

## Next Steps

- Read the [Syntax Guide](./syntax.md)
- Explore [Examples](../examples/)
- Check out the [Architecture](./architecture.md)
