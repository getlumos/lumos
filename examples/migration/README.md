# Schema Migration Example

This example demonstrates how to use LUMOS to generate automatic migration code when schemas evolve over time.

## Overview

When your Solana program's data structures change, you need migration code to transform existing account data to the new format. LUMOS can automatically generate this migration code for you.

## Example Schemas

### v1.0.0 - Initial Schema
```lumos
#[solana]
#[version = "1.0.0"]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    username: String,
    level: u16,
    score: u64,
}
```

### v2.0.0 - Updated Schema
```lumos
#[solana]
#[version = "2.0.0"]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    username: String,
    level: u16,
    score: u64,
    experience: u64,        // NEW
    inventory: [u64],       // NEW
    email: Option<String>,  // NEW (optional)
}
```

## Changes from v1 → v2

- **Added** `experience` (u64) - XP tracking for the player
- **Added** `inventory` (Vec<u64>) - player's item IDs
- **Added** `email` (Option<String>) - optional contact information

## Generating Migration Code

### Using the CLI

```bash
# Generate migration code for both Rust and TypeScript
lumos migrate player_v1.lumos player_v2.lumos --output migration

# This creates:
#   - migration.rs   (Rust migration code)
#   - migration.ts   (TypeScript migration code)
```

### Dry Run Mode

Preview the generated migration code without writing files:

```bash
lumos migrate player_v1.lumos player_v2.lumos --dry-run
```

### Language-Specific Generation

```bash
# Generate only Rust migration code
lumos migrate player_v1.lumos player_v2.lumos --output migration.rs --language rust

# Generate only TypeScript migration code
lumos migrate player_v1.lumos player_v2.lumos --output migration.ts --language typescript
```

## Generated Migration Code

### Rust

The generated Rust code includes:

```rust
// Old version struct definition
#[derive(BorshSerialize, BorshDeserialize)]
pub struct PlayerAccountV1_0_0 {
    pub wallet: Pubkey,
    pub username: String,
    pub level: u16,
    pub score: u64,
}

impl PlayerAccount {
    /// Migrate from v1.0.0 to v2.0.0
    ///
    /// Changes:
    /// - Added field: experience
    /// - Added field: inventory
    /// - Added field: email
    pub fn migrate_from_v1_0_0(old: PlayerAccountV1_0_0) -> Self {
        Self {
            wallet: old.wallet,
            username: old.username,
            level: old.level,
            score: old.score,
            experience: 0,          // Default: Added in v2.0.0
            inventory: Vec::new(),  // Default: Added in v2.0.0
            email: None,            // Default: Added in v2.0.0
        }
    }
}
```

### TypeScript

The generated TypeScript code includes:

```typescript
// Old version interface
export interface PlayerAccountV1_0_0 {
  wallet: PublicKey;
  username: string;
  level: number;
  score: number;
}

/**
 * Migrate PlayerAccount from v1.0.0 to v2.0.0
 *
 * Changes:
 * - Added field: experience
 * - Added field: inventory
 * - Added field: email
 */
export function migratePlayerAccountFromV1_0_0(
  old: PlayerAccountV1_0_0
): PlayerAccount {
  return {
    wallet: old.wallet,
    username: old.username,
    level: old.level,
    score: old.score,
    experience: 0,       // Default: Added in v2.0.0
    inventory: [],       // Default: Added in v2.0.0
    email: undefined,    // Default: Added in v2.0.0
  };
}
```

## Migration Safety

LUMOS analyzes the changes and classifies migrations as safe or unsafe:

### Safe Migrations ✅
- Adding optional fields (Option<T>)
- Reordering fields (Borsh uses names, not positions)
- Adding enum variants

### Unsafe Migrations ⚠️
- Adding required fields (need default values)
- Removing fields (data loss)
- Changing field types (potential overflow/conversion errors)
- Removing enum variants

For unsafe migrations, use the `--force` flag:

```bash
lumos migrate v1.lumos v2.lumos --output migration --force
```

## Using Migration Code

### In Your Anchor Program

```rust
use borsh::{BorshDeserialize, BorshSerialize};

// Import generated migration code
mod migration;
use migration::PlayerAccountV1_0_0;

#[program]
pub mod my_program {
    use super::*;

    pub fn migrate_player_account(ctx: Context<MigratePlayer>) -> Result<()> {
        // Read old account data
        let old_data = PlayerAccountV1_0_0::try_from_slice(
            &ctx.accounts.player_account.data.borrow()
        )?;

        // Migrate to new version
        let new_data = PlayerAccount::migrate_from_v1_0_0(old_data);

        // Write migrated data back
        let mut data = ctx.accounts.player_account.try_borrow_mut_data()?;
        new_data.serialize(&mut &mut data[..])?;

        Ok(())
    }
}
```

### In Your TypeScript Client

```typescript
import { migratePlayerAccountFromV1_0_0 } from "./migration";
import { PlayerAccountV1_0_0 } from "./generated_v1";
import { PlayerAccount } from "./generated_v2";

// Read old account data
const oldAccount: PlayerAccountV1_0_0 = await program.account.playerAccount.fetch(
  playerAccountPubkey
);

// Migrate data
const newAccount: PlayerAccount = migratePlayerAccountFromV1_0_0(oldAccount);

// Update on-chain (if supported by your program)
await program.methods
  .migratePlayerAccount()
  .accounts({
    playerAccount: playerAccountPubkey,
  })
  .rpc();
```

## Best Practices

1. **Version Everything**: Always use `#[version]` attribute
2. **Test Migrations**: Write unit tests for migration functions
3. **Document Changes**: Add comments explaining what changed and why
4. **Gradual Rollout**: Support both versions temporarily during transition
5. **Backup Data**: Always backup account data before migrating
6. **Validate**: Test migration on devnet first

## Advanced: Batch Migration

For migrating many accounts at once:

```rust
pub fn batch_migrate_players(
    accounts: Vec<AccountInfo>,
) -> Result<Vec<PlayerAccount>> {
    accounts
        .iter()
        .map(|account| {
            let old = PlayerAccountV1_0_0::try_from_slice(&account.data.borrow())?;
            Ok(PlayerAccount::migrate_from_v1_0_0(old))
        })
        .collect()
}
```

## Further Reading

- [LUMOS Syntax Reference](/docs/syntax-reference.md) - Full language documentation
- [Schema Versioning Example](/examples/versioning/) - Version tracking with `#[version]`
- [Borsh Specification](https://borsh.io/) - Understanding serialization format

---

**Generated with LUMOS** - The type-safe schema language for Solana
