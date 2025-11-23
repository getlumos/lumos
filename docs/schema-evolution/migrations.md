# Schema Migration Guide

**Automatic migration code generation for evolving Solana program schemas**

## Overview

When your Solana program's data structures evolve, you need migration code to safely transform existing account data to the new format. LUMOS automatically generates type-safe migration code for both Rust and TypeScript.

## Quick Start

```bash
# Generate migration code from v1 to v2
lumos migrate schema_v1.lumos schema_v2.lumos --output migration
```

This creates:
- `migration.rs` - Rust migration functions
- `migration.ts` - TypeScript migration helpers

## How It Works

1. **Parse both schemas** - Old and new versions
2. **Detect changes** - Fields added, removed, or modified
3. **Assess safety** - Classify migration as safe or unsafe
4. **Generate code** - Automatic migration functions with defaults

## Migration Safety

### Safe Migrations ✅

**Adding Optional Fields**
```lumos
// v1
struct User {
    id: u64,
    name: String,
}

// v2 - SAFE
struct User {
    id: u64,
    name: String,
    email: Option<String>,  // Safe: optional field
}
```

**Reordering Fields**
```lumos
// v1
struct Player {
    wallet: PublicKey,
    level: u16,
    score: u64,
}

// v2 - SAFE (Borsh uses field names, not positions)
struct Player {
    level: u16,        // Reordered
    wallet: PublicKey,
    score: u64,
}
```

**Adding Enum Variants**
```lumos
// v1
enum Status {
    Active,
    Paused,
}

// v2 - SAFE
enum Status {
    Active,
    Paused,
    Completed,  // New variant
}
```

### Unsafe Migrations ⚠️

Require `--force` flag and careful handling.

**Adding Required Fields**
```lumos
// v1
struct User {
    id: u64,
}

// v2 - UNSAFE (requires default value)
struct User {
    id: u64,
    balance: u64,  // Must have default
}
```

**Removing Fields**
```lumos
// v1
struct Account {
    balance: u64,
    deprecated_field: bool,
}

// v2 - UNSAFE (data loss)
struct Account {
    balance: u64,
    // deprecated_field removed
}
```

**Changing Field Types**
```lumos
// v1
struct Stats {
    count: u32,
}

// v2 - UNSAFE (potential overflow)
struct Stats {
    count: u16,  // Type changed
}
```

## CLI Usage

### Basic Command

```bash
lumos migrate <FROM_SCHEMA> <TO_SCHEMA> [OPTIONS]
```

### Options

```
-o, --output <FILE>      Output file path (default: stdout)
-l, --language <LANG>    rust, typescript, or both (default: both)
-n, --dry-run            Show changes without generating code
-f, --force              Proceed with unsafe migrations
```

### Examples

**Preview Changes**
```bash
lumos migrate v1.lumos v2.lumos --dry-run
```

**Generate Rust Only**
```bash
lumos migrate v1.lumos v2.lumos --output migration.rs --language rust
```

**Force Unsafe Migration**
```bash
lumos migrate v1.lumos v2.lumos --output migration --force
```

**Output to Stdout**
```bash
lumos migrate v1.lumos v2.lumos
```

## Generated Code

### Rust Migration

Given a schema change from v1 to v2:

```rust
// Auto-generated migration code by LUMOS
// Migration from v1.0.0 to v2.0.0

// Old version struct definition
#[derive(BorshSerialize, BorshDeserialize)]
pub struct PlayerAccountV1_0_0 {
    pub wallet: Pubkey,
    pub level: u16,
}

impl PlayerAccount {
    /// Migrate from v1.0.0 to v2.0.0
    ///
    /// Changes:
    /// - Added field: experience
    /// - Added field: inventory
    pub fn migrate_from_v1_0_0(old: PlayerAccountV1_0_0) -> Self {
        Self {
            wallet: old.wallet,
            level: old.level,
            experience: 0,          // Default: Added in v2.0.0
            inventory: Vec::new(),  // Default: Added in v2.0.0
        }
    }
}
```

### TypeScript Migration

```typescript
// Auto-generated migration code by LUMOS
// Migration from v1.0.0 to v2.0.0

// Old version interface
export interface PlayerAccountV1_0_0 {
  wallet: PublicKey;
  level: number;
}

/**
 * Migrate PlayerAccount from v1.0.0 to v2.0.0
 *
 * Changes:
 * - Added field: experience
 * - Added field: inventory
 */
export function migratePlayerAccountFromV1_0_0(
  old: PlayerAccountV1_0_0
): PlayerAccount {
  return {
    wallet: old.wallet,
    level: old.level,
    experience: 0,    // Default: Added in v2.0.0
    inventory: [],    // Default: Added in v2.0.0
  };
}
```

## Default Values

LUMOS uses sensible defaults for new fields:

| Type | Default Value (Rust) | Default Value (TypeScript) |
|------|---------------------|---------------------------|
| Numeric (`u8`, `u64`, etc.) | `0` | `0` |
| `bool` | `false` | `false` |
| `String` | `String::new()` | `""` |
| `PublicKey` | `Pubkey::default()` | `PublicKey.default` |
| `Vec<T>` | `Vec::new()` | `[]` |
| `Option<T>` | `None` | `undefined` |

## Using Migration Code

### In Your Anchor Program

```rust
use borsh::{BorshDeserialize, BorshSerialize};
mod migration;

#[program]
pub mod my_program {
    use super::*;

    pub fn migrate_account(ctx: Context<Migrate>) -> Result<()> {
        // Read old account data
        let account_data = &ctx.accounts.account.data.borrow();
        let old = migration::PlayerAccountV1_0_0::try_from_slice(account_data)?;

        // Migrate to new version
        let new = PlayerAccount::migrate_from_v1_0_0(old);

        // Serialize and write back
        let mut data = ctx.accounts.account.try_borrow_mut_data()?;
        new.serialize(&mut &mut data[..])?;

        Ok(())
    }
}
```

### In Your TypeScript Client

```typescript
import { migratePlayerAccountFromV1_0_0 } from "./migration";

// Fetch old account
const oldAccount = await program.account.playerAccount.fetch(accountPubkey);

// Migrate data
const newAccount = migratePlayerAccountFromV1_0_0(oldAccount);

// Send migration transaction
await program.methods
  .migrateAccount()
  .accounts({ account: accountPubkey })
  .rpc();
```

## Best Practices

### 1. Version Everything

Always use `#[version]` attribute:

```lumos
#[solana]
#[version = "1.0.0"]
#[account]
struct MyAccount {
    // ...
}
```

### 2. Test Migrations

Write unit tests for migration functions:

```rust
#[test]
fn test_v1_to_v2_migration() {
    let v1 = PlayerAccountV1_0_0 {
        wallet: Pubkey::default(),
        level: 10,
    };

    let v2 = PlayerAccount::migrate_from_v1_0_0(v1);

    assert_eq!(v2.level, 10);
    assert_eq!(v2.experience, 0);
}
```

### 3. Gradual Rollout

Support both versions temporarily:

```rust
pub fn process_account(data: &[u8]) -> Result<PlayerAccount> {
    // Try new format first
    if let Ok(account) = PlayerAccount::try_from_slice(data) {
        return Ok(account);
    }

    // Fall back to old format and migrate
    let old = PlayerAccountV1_0_0::try_from_slice(data)?;
    Ok(PlayerAccount::migrate_from_v1_0_0(old))
}
```

### 4. Document Changes

Keep a changelog of schema versions:

```markdown
## v2.0.0 (2025-01-15)
- Added: `experience` field for XP tracking
- Added: `inventory` array for items

## v1.0.0 (2025-01-01)
- Initial release
```

### 5. Backup Before Migrating

Always backup account data before running migrations on mainnet:

```bash
# Export all accounts
solana program dump <PROGRAM_ID> backup.so

# Or use snapshot tools
anchor account-snapshot --program <PROGRAM_ID>
```

## Advanced: Batch Migration

Migrate multiple accounts efficiently:

```rust
pub fn batch_migrate_accounts(
    program_id: &Pubkey,
    accounts: Vec<(Pubkey, Account)>,
) -> Result<Vec<(Pubkey, PlayerAccount)>> {
    accounts
        .into_iter()
        .map(|(pubkey, account)| {
            let old = PlayerAccountV1_0_0::try_from_slice(&account.data)?;
            let new = PlayerAccount::migrate_from_v1_0_0(old);
            Ok((pubkey, new))
        })
        .collect()
}
```

## Troubleshooting

### "Unsafe migration requires --force flag"

**Problem**: Adding required fields or removing fields.

**Solution**: Review changes and use `--force` if acceptable:
```bash
lumos migrate v1.lumos v2.lumos --force
```

### "Type names don't match"

**Problem**: Comparing different types.

**Solution**: Ensure both schemas define the same type names.

### Migration Produces Wrong Defaults

**Problem**: Auto-generated defaults don't match your needs.

**Solution**: Manually edit generated migration code:
```rust
// Generated
experience: 0,

// Customize
experience: calculate_default_xp(old.level),
```

## Examples

- [Basic Migration](/examples/migration/) - Adding fields to a player account
- [Version Tracking](/examples/versioning/) - Using `#[version]` attribute

## Related

- [Schema Versioning](/docs/syntax-reference.md#version) - `#[version]` attribute
- [Borsh Serialization](https://borsh.io/) - Understanding data format
- [Anchor Account Upgrades](https://www.anchor-lang.com/) - Program upgrades

---

**Next Steps**:
- [Backward Compatibility Validation](#42) - Coming in Phase 5.1
- [Automatic Migration Testing](#43) - Coming in Phase 5.1
- [Schema Diff Tool](#44) - Coming in Phase 5.1
