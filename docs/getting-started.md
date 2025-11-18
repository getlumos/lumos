# Getting Started with LUMOS

> **Status:** Phase 3.1 (Enum Support) - Production Ready

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
├── schema.lumos        # Your schema definitions
└── .gitignore          # Ignores generated files
```

### 2. Define your schema

Edit `schema.lumos`:

```lumos
#[solana]
#[account]
struct UserAccount {
    wallet: PublicKey,
    balance: u64,
    level: u16,
}

#[solana]
enum UserStatus {
    Active,
    Inactive,
    Suspended,
}
```

### 3. Generate code

```bash
lumos generate schema.lumos
```

This generates:

**Rust** (`generated.rs`):
```rust
use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

#[account]
pub struct UserAccount {
    pub wallet: Pubkey,
    pub balance: u64,
    pub level: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}
```

**TypeScript** (`generated.ts`):
```typescript
import * as borsh from '@coral-xyz/borsh';
import { PublicKey } from '@solana/web3.js';

export interface UserAccount {
  wallet: PublicKey;
  balance: number;
  level: number;
}

export const UserAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.u64('balance'),
  borsh.u16('level'),
]);

export type UserStatus =
  | { kind: 'Active' }
  | { kind: 'Inactive' }
  | { kind: 'Suspended' };

export const UserStatusSchema = borsh.rustEnum([
  borsh.unit('Active'),
  borsh.unit('Inactive'),
  borsh.unit('Suspended'),
]);
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
