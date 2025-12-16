# Migration Guide: Adding LUMOS to Existing Anchor Projects

> **Adopt LUMOS incrementally without rewriting your Anchor program**

This guide walks you through adding LUMOS to an existing Anchor project. Whether you want to replace manual IDL files, generate type-safe TypeScript, or gradually migrate account definitions, LUMOS integrates seamlessly with your current workflow.

---

## Table of Contents

1. [Introduction & Benefits](#introduction--benefits)
2. [Understanding LUMOS + Anchor](#understanding-lumos--anchor)
3. [Migration Strategies](#migration-strategies)
4. [Analyzing Your Existing Anchor Program](#analyzing-your-existing-anchor-program)
5. [Creating LUMOS Schemas from Anchor](#creating-lumos-schemas-from-anchor)
6. [Anchor Plugin Commands](#anchor-plugin-commands)
7. [Space Calculation Deep Dive](#space-calculation-deep-dive)
8. [Real-World Migration Examples](#real-world-migration-examples)
9. [Best Practices & Checklist](#best-practices--checklist)

---

## Introduction & Benefits

### Why Add LUMOS to Existing Anchor?

You've built an Anchor program. It works. But you're maintaining:

- **Manual IDL files** that drift from your code
- **Scattered type definitions** across Rust and TypeScript
- **Hand-calculated space constants** that break when you add fields
- **Duplicate interfaces** in your frontend that must match your program

LUMOS solves this by providing a **single source of truth**:

```
schema.lumos → Rust accounts + TypeScript types + Anchor IDL + Space calculations
```

### What Changes vs What Stays

| Stays the Same | Changes with LUMOS |
|----------------|-------------------|
| Instruction handlers | Account struct definitions |
| Business logic | IDL generation |
| Program deployment | TypeScript interfaces |
| Anchor CLI workflow | Space calculations |
| Test structure | Type synchronization |

### Migration Benefits

| Before (Manual Anchor) | After (LUMOS + Anchor) |
|------------------------|------------------------|
| IDL hand-written or auto-generated with drift | IDL generated from schema |
| Account structs scattered in lib.rs | Account structs from single schema |
| `const LEN: usize = 8 + 32 + ...` manual | `lumos anchor space` automatic |
| TypeScript types manually duplicated | Generated alongside Rust |
| No precision warnings | Auto-documented u64 limits |

### Three Adoption Paths

Choose based on your comfort level and project state:

| Strategy | Risk Level | Effort | Best For |
|----------|------------|--------|----------|
| **IDL-Only** | Low | 1-2 hours | Existing stable programs |
| **Gradual Migration** | Medium | 1-2 days | Active development |
| **Full Generation** | Higher | 2-3 days | New features or rewrites |

---

## Understanding LUMOS + Anchor

### Context-Aware Code Generation

LUMOS automatically detects when you're working with Anchor and adjusts its output:

```lumos
// This schema...
#[solana]
#[account]
struct UserAccount {
    authority: PublicKey,
    balance: u64,
}

#[solana]
struct TransferParams {
    amount: u64,
    memo: String,
}
```

**Generates context-aware Rust:**

```rust
// LUMOS detects #[account] and uses Anchor imports for ENTIRE module
use anchor_lang::prelude::*;

// Account struct - Anchor provides serialization
#[account]
pub struct UserAccount {
    pub authority: Pubkey,
    pub balance: u64,
}

// Non-account struct in Anchor module - uses Anchor serialization
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TransferParams {
    pub amount: u64,
    pub memo: String,
}
```

### Detection Logic

LUMOS scans your schema for `#[account]` attributes:

| Condition | Import Style | Struct Derives |
|-----------|--------------|----------------|
| Any `#[account]` in schema | `use anchor_lang::prelude::*;` | None for accounts (Anchor provides) |
| No `#[account]` anywhere | `use borsh::{BorshSerialize, BorshDeserialize};` | BorshSerialize, BorshDeserialize |
| Non-account in Anchor schema | Same as above | AnchorSerialize, AnchorDeserialize |

### LUMOS Attributes for Anchor

| LUMOS Attribute | Purpose | Anchor Equivalent |
|-----------------|---------|-------------------|
| `#[solana]` | Marks Solana-compatible type | Required for all types |
| `#[account]` | On-chain account state | `#[account]` macro |
| `#[key]` | PDA seed field marker | Used in `seeds = [...]` |
| `#[max(N)]` | String/array max length | Space calculation |
| `#[deprecated]` | Mark deprecated fields | Compile warnings |

### What LUMOS Generates vs What You Write

**LUMOS generates:**
- Account struct definitions with `#[account]`
- Non-account data types with proper derives
- Anchor IDL (JSON)
- TypeScript interfaces and Borsh schemas
- Space calculations

**You still write:**
- Instruction handlers (`pub fn initialize(...)`)
- `#[derive(Accounts)]` contexts
- Business logic and validations
- Error definitions
- Events

---

## Migration Strategies

### Strategy 1: IDL-Only Migration (Lowest Risk)

**Best for:** Stable programs where you don't want to touch Rust code

**What changes:** Only IDL and TypeScript types

```
Existing Program          LUMOS Addition
─────────────────         ──────────────
programs/                 schema.lumos (NEW)
├── src/lib.rs           │
│   └── #[account]       │ ← Mirror these
│       structs          │
└── Cargo.toml           │
                         │
target/idl/              │
└── program.json    ←────┘ Generated from schema
                         │
app/src/                 │
└── types.ts        ←────┘ Generated from schema
```

**Steps:**

1. **Create schema.lumos** matching your existing account structs:

```lumos
// schema.lumos - mirrors programs/src/lib.rs accounts
#[solana]
#[account]
struct UserAccount {
    authority: PublicKey,
    balance: u64,
    is_initialized: bool,
}

#[solana]
#[account]
struct VaultAccount {
    owner: PublicKey,
    token_mint: PublicKey,
    amount: u64,
    bump: u8,
}
```

2. **Generate IDL:**

```bash
lumos anchor idl schema.lumos \
  --output target/idl/my_program.json \
  --address YOUR_PROGRAM_ID \
  --pretty
```

3. **Generate TypeScript types:**

```bash
lumos generate schema.lumos \
  --lang typescript \
  --output app/src/generated-types.ts
```

4. **Update imports in your frontend:**

```typescript
// Before
import { UserAccount } from './types'; // Manual types

// After
import { UserAccount, UserAccountSchema } from './generated-types';
```

**Benefits:**
- Zero Rust code changes
- Immediate type safety in TypeScript
- IDL always matches schema
- Can be done in under an hour

---

### Strategy 2: Gradual Account Migration (Medium Risk)

**Best for:** Active development where you want type-safe accounts

**What changes:** Account definitions move to LUMOS, handlers stay in lib.rs

```
Before                    After
──────                    ─────
programs/src/             programs/src/
├── lib.rs               ├── lib.rs
│   ├── #[account]       │   ├── mod accounts;  ← Import generated
│   │   structs (DELETE) │   └── handlers only
│   └── handlers         │
└── Cargo.toml           ├── accounts.rs  ← Generated by LUMOS
                         └── Cargo.toml

                         schema.lumos  ← Source of truth
```

**Steps:**

1. **Create schema.lumos** with all account types:

```lumos
#[solana]
#[account]
struct StakingPool {
    #[key]
    authority: PublicKey,
    token_mint: PublicKey,
    vault: PublicKey,
    total_staked: u64,
    reward_rate: u64,
    is_active: bool,
    bump: u8,
}

#[solana]
#[account]
struct Staker {
    #[key]
    owner: PublicKey,
    pool: PublicKey,
    staked_amount: u64,
    reward_debt: u64,
    last_stake_time: i64,
}
```

2. **Generate Rust accounts:**

```bash
lumos generate schema.lumos \
  --lang rust \
  --output programs/staking/src/accounts.rs
```

3. **Update lib.rs to import generated accounts:**

```rust
// programs/staking/src/lib.rs

use anchor_lang::prelude::*;

// Import generated account types
mod accounts;
pub use accounts::*;

declare_id!("YOUR_PROGRAM_ID");

#[program]
pub mod staking {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, reward_rate: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.reward_rate = reward_rate;
        pool.is_active = true;
        // ... rest of handler
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        // Handler logic stays here
        Ok(())
    }
}

// Accounts contexts stay in lib.rs
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = StakingPool::LEN,  // Use generated constant
        seeds = [b"pool", authority.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, StakingPool>,  // Uses generated type

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

4. **Add generation to build script:**

```json
// package.json
{
  "scripts": {
    "generate": "lumos generate schema.lumos -o programs/staking/src/accounts.rs",
    "build": "npm run generate && anchor build"
  }
}
```

**Benefits:**
- Account types are type-safe and centralized
- Handlers and business logic unchanged
- Gradual adoption - migrate one account at a time
- Easy rollback if issues arise

---

### Strategy 3: Full Program Generation (New Features)

**Best for:** New programs or major feature additions

**What changes:** Complete program structure generated from schema

```bash
lumos anchor generate schema.lumos \
  --address YOUR_PROGRAM_ID \
  --name my_program \
  --typescript \
  --output ./
```

**Generates:**
```
./
├── programs/my_program/src/lib.rs    # Complete Anchor program
├── target/idl/my_program.json        # Anchor IDL
└── app/src/types.ts                  # TypeScript client types
```

**Generated lib.rs structure:**

```rust
use anchor_lang::prelude::*;

declare_id!("YOUR_PROGRAM_ID");

// ============================================================
// ACCOUNTS - Generated from schema.lumos
// ============================================================

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub vault: Pubkey,
    pub total_staked: u64,
    pub reward_rate: u64,
    pub is_active: bool,
    pub bump: u8,
}

impl StakingPool {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 8 + 1 + 1; // 122 bytes
}

// ============================================================
// CUSTOM TYPES - Generated from schema.lumos
// ============================================================

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct StakeParams {
    pub amount: u64,
    pub lock_duration: i64,
}

// ============================================================
// PROGRAM - Add your handlers below
// ============================================================

#[program]
pub mod my_program {
    use super::*;

    // TODO: Implement your instruction handlers
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

// ============================================================
// CONTEXTS - Add your Accounts derives below
// ============================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    // TODO: Define your accounts
}
```

**Benefits:**
- Complete scaffolding in seconds
- Guaranteed type consistency
- All space calculations done
- TypeScript ready immediately

---

## Analyzing Your Existing Anchor Program

Before migration, audit your existing code:

### Step 1: Find All Account Definitions

```bash
# Find all #[account] structs
grep -n "#\[account\]" programs/*/src/*.rs

# Example output:
# programs/staking/src/lib.rs:15:#[account]
# programs/staking/src/lib.rs:25:#[account]
# programs/staking/src/state.rs:8:#[account]
```

### Step 2: Document Account Structures

Create an inventory of existing accounts:

```markdown
## Account Inventory

| Account | File | Fields | Space | PDA Seeds |
|---------|------|--------|-------|-----------|
| StakingPool | lib.rs:15 | 7 | 122 | ["pool", authority] |
| Staker | lib.rs:25 | 5 | 89 | ["staker", owner, pool] |
| RewardVault | state.rs:8 | 4 | 73 | ["vault", pool] |
```

### Step 3: Extract Account Definitions

For each account, note the structure:

```rust
// Existing Anchor code at lib.rs:15
#[account]
pub struct StakingPool {
    pub authority: Pubkey,       // 32 bytes - PDA seed
    pub token_mint: Pubkey,      // 32 bytes
    pub vault: Pubkey,           // 32 bytes
    pub total_staked: u64,       // 8 bytes
    pub reward_rate: u64,        // 8 bytes
    pub is_active: bool,         // 1 byte
    pub bump: u8,                // 1 byte
}

impl StakingPool {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 8 + 1 + 1; // 122
}
```

### Step 4: Identify Type Mappings

Map Anchor types to LUMOS:

| Anchor Type | LUMOS Type | Notes |
|-------------|-----------|-------|
| `Pubkey` | `PublicKey` | Solana address |
| `u64` | `u64` | Direct mapping |
| `i64` | `i64` | Direct mapping |
| `bool` | `bool` | Direct mapping |
| `String` | `String` | Add `#[max(N)]` |
| `Vec<T>` | `[T]` | Dynamic array |
| `[T; N]` | `[T; N]` | Fixed array |
| `Option<T>` | `Option<T>` | Direct mapping |

### Step 5: Identify PDAs and Seeds

Look for seed patterns in your Accounts contexts:

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = StakingPool::LEN,
        seeds = [b"pool", authority.key().as_ref()],  // ← Note these
        bump
    )]
    pub pool: Account<'info, StakingPool>,
}
```

**Seed analysis:**
- `b"pool"` - Static seed (string literal)
- `authority.key()` - Dynamic seed (PublicKey field)

In LUMOS, mark dynamic seed fields with `#[key]`:

```lumos
#[solana]
#[account]
struct StakingPool {
    #[key]  // ← Marks this as PDA seed component
    authority: PublicKey,
    // ... other fields
}
```

---

## Creating LUMOS Schemas from Anchor

### Basic Account Conversion

**Anchor:**
```rust
#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub balance: u64,
    pub created_at: i64,
    pub is_active: bool,
}
```

**LUMOS:**
```lumos
#[solana]
#[account]
struct UserAccount {
    owner: PublicKey,
    balance: u64,
    created_at: i64,
    is_active: bool,
}
```

### Strings with Length Constraints

**Anchor:**
```rust
#[account]
pub struct Profile {
    pub owner: Pubkey,
    pub username: String,  // Unknown max length!
    pub bio: String,       // Unknown max length!
}

impl Profile {
    // Manual space calculation
    pub const LEN: usize = 8 + 32 + (4 + 20) + (4 + 200); // 268
}
```

**LUMOS:**
```lumos
#[solana]
#[account]
struct Profile {
    owner: PublicKey,
    #[max(20)]   // Explicit: 20 chars max
    username: String,
    #[max(200)]  // Explicit: 200 chars max
    bio: String,
}
// Space auto-calculated: 8 + 32 + 24 + 204 = 268 bytes
```

### Arrays and Vectors

**Anchor:**
```rust
#[account]
pub struct Inventory {
    pub owner: Pubkey,
    pub items: Vec<Pubkey>,      // Dynamic array
    pub quick_slots: [Pubkey; 4], // Fixed array
}

impl Inventory {
    // Manual: 8 + 32 + (4 + 32*MAX_ITEMS) + (32*4)
    pub const LEN: usize = 8 + 32 + (4 + 32 * 50) + 128; // 1772
}
```

**LUMOS:**
```lumos
#[solana]
#[account]
struct Inventory {
    owner: PublicKey,
    #[max(50)]  // Max 50 items
    items: [PublicKey],
    quick_slots: [PublicKey; 4],
}
// Space: 8 + 32 + (4 + 32*50) + (32*4) = 1772 bytes
```

### Optional Fields

**Anchor:**
```rust
#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub price: u64,
    pub expires_at: Option<i64>,  // Optional expiration
    pub buyer: Option<Pubkey>,    // Optional buyer (when sold)
}

impl Listing {
    // Manual: 8 + 32 + 8 + (1 + 8) + (1 + 32)
    pub const LEN: usize = 8 + 32 + 8 + 9 + 33; // 90
}
```

**LUMOS:**
```lumos
#[solana]
#[account]
struct Listing {
    seller: PublicKey,
    price: u64,
    expires_at: Option<i64>,
    buyer: Option<PublicKey>,
}
// Space: 8 + 32 + 8 + 9 + 33 = 90 bytes
```

### Enums

**Anchor:**
```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum ListingStatus {
    Active,
    Sold,
    Cancelled,
    Expired,
}

#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub status: ListingStatus,  // 1 byte (enum discriminant)
}
```

**LUMOS:**
```lumos
#[solana]
enum ListingStatus {
    Active,
    Sold,
    Cancelled,
    Expired,
}

#[solana]
#[account]
struct Listing {
    seller: PublicKey,
    status: ListingStatus,
}
```

### Enums with Data

**Anchor:**
```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum GameResult {
    Pending,
    Victory { score: u64 },
    Defeat { score: u64 },
    Draw,
}
```

**LUMOS:**
```lumos
#[solana]
enum GameResult {
    Pending,
    Victory { score: u64 },
    Defeat { score: u64 },
    Draw,
}
```

### Complex Nested Types

**Anchor:**
```rust
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ItemStats {
    pub power: u16,
    pub defense: u16,
    pub speed: u16,
}

#[account]
pub struct GameItem {
    pub owner: Pubkey,
    pub item_type: u8,
    pub stats: ItemStats,
    pub enchantments: Vec<u8>,
}
```

**LUMOS:**
```lumos
#[solana]
struct ItemStats {
    power: u16,
    defense: u16,
    speed: u16,
}

#[solana]
#[account]
struct GameItem {
    owner: PublicKey,
    item_type: u8,
    stats: ItemStats,
    #[max(10)]
    enchantments: [u8],
}
```

### PDA Seed Fields

Mark fields used in PDA seeds with `#[key]`:

**Anchor (context):**
```rust
#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(
        init,
        seeds = [b"vault", owner.key().as_ref(), token_mint.key().as_ref()],
        bump,
        // ...
    )]
    pub vault: Account<'info, Vault>,
    pub owner: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
}
```

**LUMOS:**
```lumos
#[solana]
#[account]
struct Vault {
    #[key]  // Used in seeds
    owner: PublicKey,
    #[key]  // Used in seeds
    token_mint: PublicKey,
    balance: u64,
    bump: u8,
}
```

---

## Anchor Plugin Commands

LUMOS provides three specialized commands for Anchor integration:

### Command 1: `lumos anchor generate`

**Purpose:** Generate a complete Anchor program from schema

**Usage:**
```bash
lumos anchor generate <SCHEMA> [OPTIONS]

Arguments:
  <SCHEMA>  Path to LUMOS schema file

Options:
  --address <PUBKEY>     Program address (required)
  --name <NAME>          Program name (default: schema filename)
  --version <VERSION>    Program version (default: 0.1.0)
  --output <DIR>         Output directory (default: current)
  --typescript           Also generate TypeScript types
  --dry-run              Preview without writing files
```

**Example:**
```bash
lumos anchor generate schema.lumos \
  --address "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS" \
  --name staking_program \
  --typescript \
  --output ./my-project
```

**Output Structure:**
```
my-project/
├── programs/staking_program/
│   └── src/
│       └── lib.rs              # Complete Anchor program
├── target/
│   └── idl/
│       └── staking_program.json # Anchor IDL
└── app/
    └── src/
        └── types.ts            # TypeScript types (if --typescript)
```

**Generated lib.rs:**
```rust
use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// ============================================================
// ACCOUNTS
// ============================================================

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub total_staked: u64,
    pub is_active: bool,
}

impl StakingPool {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1; // 81 bytes
}

// ============================================================
// PROGRAM
// ============================================================

#[program]
pub mod staking_program {
    use super::*;

    // TODO: Add instruction handlers
}
```

---

### Command 2: `lumos anchor idl`

**Purpose:** Generate Anchor IDL only (for existing programs)

**Usage:**
```bash
lumos anchor idl <SCHEMA> [OPTIONS]

Arguments:
  <SCHEMA>  Path to LUMOS schema file

Options:
  --output <PATH>        Output file (default: target/idl/<name>.json)
  --name <NAME>          Program name
  --version <VERSION>    IDL version (default: 0.1.0)
  --address <PUBKEY>     Program address (for metadata)
  --pretty               Pretty-print JSON output
```

**Example:**
```bash
lumos anchor idl schema.lumos \
  --output target/idl/my_program.json \
  --address "YOUR_PROGRAM_ID" \
  --pretty
```

**Generated IDL:**
```json
{
  "version": "0.1.0",
  "name": "my_program",
  "accounts": [
    {
      "name": "StakingPool",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "authority", "type": "publicKey" },
          { "name": "tokenMint", "type": "publicKey" },
          { "name": "totalStaked", "type": "u64" },
          { "name": "isActive", "type": "bool" }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "StakeParams",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "amount", "type": "u64" }
        ]
      }
    }
  ],
  "metadata": {
    "address": "YOUR_PROGRAM_ID"
  }
}
```

**IDL Type Mappings:**

| LUMOS Type | IDL Type |
|-----------|----------|
| `u8` - `u128` | `u8` - `u128` |
| `i8` - `i128` | `i8` - `i128` |
| `bool` | `bool` |
| `String` | `string` |
| `PublicKey` | `publicKey` |
| `[T]` | `{ "vec": T }` |
| `[T; N]` | `{ "array": [T, N] }` |
| `Option<T>` | `{ "option": T }` |

---

### Command 3: `lumos anchor space`

**Purpose:** Calculate account space requirements

**Usage:**
```bash
lumos anchor space <SCHEMA> [OPTIONS]

Arguments:
  <SCHEMA>  Path to LUMOS schema file

Options:
  --format <FORMAT>      Output format: "text" or "rust" (default: text)
  --account <NAME>       Calculate for specific account only
```

**Example - Text Format:**
```bash
$ lumos anchor space schema.lumos

Account Space Calculation
═════════════════════════════════════════════════════════════

StakingPool                                         122 bytes
├── discriminator                                     8 bytes
├── authority: PublicKey                             32 bytes
├── token_mint: PublicKey                            32 bytes
├── vault: PublicKey                                 32 bytes
├── total_staked: u64                                 8 bytes
├── reward_rate: u64                                  8 bytes
├── is_active: bool                                   1 byte
└── bump: u8                                          1 byte

Staker                                               89 bytes
├── discriminator                                     8 bytes
├── owner: PublicKey                                 32 bytes
├── pool: PublicKey                                  32 bytes
├── staked_amount: u64                                8 bytes
├── reward_debt: u64                                  8 bytes
└── last_stake_time: i64                              8 bytes

═════════════════════════════════════════════════════════════
Total accounts: 2
```

**Example - Rust Format:**
```bash
$ lumos anchor space schema.lumos --format rust

// Auto-generated by LUMOS - copy into your program

impl StakingPool {
    /// Account size: 8 (discriminator) + 114 (data) = 122 bytes
    pub const LEN: usize = 122;
}

impl Staker {
    /// Account size: 8 (discriminator) + 81 (data) = 89 bytes
    pub const LEN: usize = 89;
}
```

**Example - Specific Account:**
```bash
$ lumos anchor space schema.lumos --account StakingPool

StakingPool: 122 bytes
  8   discriminator
  32  authority (PublicKey)
  32  token_mint (PublicKey)
  32  vault (PublicKey)
  8   total_staked (u64)
  8   reward_rate (u64)
  1   is_active (bool)
  1   bump (u8)
```

---

## Space Calculation Deep Dive

### The Formula

```
Total Space = 8 (Anchor discriminator) + sum(field_sizes)
```

The 8-byte discriminator is a hash of the account name, used by Anchor to identify account types.

### Type Size Reference

| Type | Size (bytes) | Notes |
|------|--------------|-------|
| `bool` | 1 | 0 or 1 |
| `u8` / `i8` | 1 | |
| `u16` / `i16` | 2 | |
| `u32` / `i32` | 4 | |
| `u64` / `i64` | 8 | |
| `u128` / `i128` | 16 | |
| `f32` | 4 | IEEE 754 |
| `f64` | 8 | IEEE 754 |
| `PublicKey` | 32 | Solana address |
| `Signature` | 64 | ed25519 |
| `String` | 4 + max_len | Length prefix + content |
| `Vec<T>` | 4 + (size_of::<T>() × max_items) | Length prefix + elements |
| `[T; N]` | size_of::<T>() × N | Fixed size |
| `Option<T>` | 1 + size_of::<T>() | Tag byte + inner |
| `enum` | 1 + max_variant_size | Discriminant + largest variant |

### String Space Calculation

Strings use a 4-byte length prefix plus content:

```lumos
#[solana]
#[account]
struct Profile {
    #[max(20)]
    username: String,    // 4 + 20 = 24 bytes
    #[max(200)]
    bio: String,         // 4 + 200 = 204 bytes
}
// Space: 8 + 24 + 204 = 236 bytes
```

**Why `#[max(N)]` matters:**
- Without it, LUMOS assumes 0 max length (just the prefix)
- Anchor needs to know max size for rent calculation
- Prevents runtime errors from oversized data

### Vector Space Calculation

Vectors use a 4-byte length prefix plus element space:

```lumos
#[solana]
#[account]
struct Inventory {
    #[max(50)]
    items: [PublicKey],  // 4 + (32 × 50) = 1604 bytes
    #[max(100)]
    scores: [u64],       // 4 + (8 × 100) = 804 bytes
}
// Space: 8 + 1604 + 804 = 2416 bytes
```

### Option Space Calculation

Options use a 1-byte tag plus the inner type size:

```lumos
#[solana]
#[account]
struct Auction {
    seller: PublicKey,           // 32 bytes
    current_bid: Option<u64>,    // 1 + 8 = 9 bytes
    winner: Option<PublicKey>,   // 1 + 32 = 33 bytes
}
// Space: 8 + 32 + 9 + 33 = 82 bytes
```

### Enum Space Calculation

Enums use 1 byte for discriminant plus the largest variant:

```lumos
#[solana]
enum GameResult {
    Pending,                      // 0 bytes data
    Victory { score: u64 },       // 8 bytes data
    Defeat { score: u64 },        // 8 bytes data
    Draw,                         // 0 bytes data
}
// Size: 1 (discriminant) + 8 (largest variant) = 9 bytes
```

### Complete Calculation Example

```lumos
#[solana]
#[account]
struct PlayerAccount {
    owner: PublicKey,              // 32
    #[max(20)]
    username: String,              // 4 + 20 = 24
    level: u16,                    // 2
    experience: u64,               // 8
    #[max(10)]
    equipped_items: [PublicKey],   // 4 + (32 × 10) = 324
    active_quest: Option<u32>,     // 1 + 4 = 5
    status: PlayerStatus,          // 1 + 8 = 9 (enum)
}

#[solana]
enum PlayerStatus {
    Idle,
    InGame { match_id: u64 },
    Trading { partner: PublicKey },  // Largest: 32 bytes
}
// PlayerStatus size: 1 + 32 = 33 bytes

// Total PlayerAccount: 8 + 32 + 24 + 2 + 8 + 324 + 5 + 33 = 436 bytes
```

---

## Real-World Migration Examples

### Example 1: Simple Token Vault

**Existing Anchor Program:**

```rust
// programs/vault/src/lib.rs

use anchor_lang::prelude::*;

declare_id!("VauLT...");

#[account]
pub struct Vault {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub token_account: Pubkey,
    pub total_deposited: u64,
    pub is_locked: bool,
    pub bump: u8,
}

impl Vault {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 1 + 1; // 114
}

#[program]
pub mod vault {
    // ... handlers
}
```

**Migration Steps:**

1. **Create schema.lumos:**

```lumos
#[solana]
#[account]
struct Vault {
    #[key]
    authority: PublicKey,
    token_mint: PublicKey,
    token_account: PublicKey,
    total_deposited: u64,
    is_locked: bool,
    bump: u8,
}
```

2. **Verify space:**

```bash
$ lumos anchor space schema.lumos --account Vault

Vault: 114 bytes ✓ (matches existing LEN)
```

3. **Generate IDL:**

```bash
lumos anchor idl schema.lumos -o target/idl/vault.json --address "VauLT..."
```

4. **Generate TypeScript:**

```bash
lumos generate schema.lumos --lang typescript -o app/src/types.ts
```

---

### Example 2: NFT Marketplace with Multiple Accounts

**Existing Program Structure:**

```rust
// programs/marketplace/src/state.rs

#[account]
pub struct Marketplace {
    pub authority: Pubkey,
    pub fee_recipient: Pubkey,
    pub fee_basis_points: u16,
    pub total_volume: u64,
    pub total_sales: u64,
    pub is_active: bool,
}

#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub price: u64,
    pub token_mint: Pubkey,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ListingStatus {
    Active,
    Sold,
    Cancelled,
}
```

**LUMOS Schema:**

```lumos
// schema.lumos

#[solana]
#[account]
struct Marketplace {
    #[key]
    authority: PublicKey,
    fee_recipient: PublicKey,
    fee_basis_points: u16,
    total_volume: u64,
    total_sales: u64,
    is_active: bool,
}

#[solana]
#[account]
struct Listing {
    seller: PublicKey,
    #[key]
    nft_mint: PublicKey,
    price: u64,
    token_mint: PublicKey,
    created_at: i64,
    expires_at: Option<i64>,
    is_active: bool,
}

#[solana]
enum ListingStatus {
    Active,
    Sold,
    Cancelled,
}
```

**Space Verification:**

```bash
$ lumos anchor space schema.lumos

Marketplace                                          91 bytes
├── discriminator                                     8 bytes
├── authority: PublicKey                             32 bytes
├── fee_recipient: PublicKey                         32 bytes
├── fee_basis_points: u16                             2 bytes
├── total_volume: u64                                 8 bytes
├── total_sales: u64                                  8 bytes
└── is_active: bool                                   1 byte

Listing                                              122 bytes
├── discriminator                                     8 bytes
├── seller: PublicKey                                32 bytes
├── nft_mint: PublicKey                              32 bytes
├── price: u64                                        8 bytes
├── token_mint: PublicKey                            32 bytes
├── created_at: i64                                   8 bytes
├── expires_at: Option<i64>                           9 bytes
└── is_active: bool                                   1 byte
```

---

### Example 3: Gaming with Complex State

**Existing Program:**

```rust
#[account]
pub struct PlayerAccount {
    pub wallet: Pubkey,
    pub username: String,           // Max 20 chars
    pub level: u16,
    pub experience: u64,
    pub gold: u64,
    pub inventory: Vec<Pubkey>,     // Max 50 items
    pub achievements: Vec<u32>,      // Max 100
    pub created_at: i64,
}

impl PlayerAccount {
    pub const MAX_USERNAME: usize = 20;
    pub const MAX_INVENTORY: usize = 50;
    pub const MAX_ACHIEVEMENTS: usize = 100;

    pub const LEN: usize = 8
        + 32                              // wallet
        + (4 + Self::MAX_USERNAME)        // username
        + 2                               // level
        + 8                               // experience
        + 8                               // gold
        + (4 + 32 * Self::MAX_INVENTORY)  // inventory
        + (4 + 4 * Self::MAX_ACHIEVEMENTS) // achievements
        + 8;                              // created_at
    // = 8 + 32 + 24 + 2 + 8 + 8 + 1604 + 404 + 8 = 2098
}
```

**LUMOS Schema:**

```lumos
#[solana]
#[account]
struct PlayerAccount {
    #[key]
    wallet: PublicKey,
    #[max(20)]
    username: String,
    level: u16,
    experience: u64,
    gold: u64,
    #[max(50)]
    inventory: [PublicKey],
    #[max(100)]
    achievements: [u32],
    created_at: i64,
}
```

**Verify and Generate:**

```bash
# Verify space matches
$ lumos anchor space schema.lumos --account PlayerAccount
PlayerAccount: 2098 bytes ✓

# Generate Rust constants
$ lumos anchor space schema.lumos --format rust > src/space.rs

# Generate complete IDL
$ lumos anchor idl schema.lumos -o target/idl/gaming.json

# Generate TypeScript
$ lumos generate schema.lumos --lang typescript -o app/src/generated.ts
```

**Generated TypeScript:**

```typescript
// app/src/generated.ts

import { PublicKey } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';

export interface PlayerAccount {
  wallet: PublicKey;
  username: string;
  level: number;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1.
   */
  experience: number;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1.
   */
  gold: number;
  inventory: PublicKey[];
  achievements: number[];
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1.
   */
  created_at: number;
}

export const PlayerAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.str('username'),
  borsh.u16('level'),
  borsh.u64('experience'),
  borsh.u64('gold'),
  borsh.vec(borsh.publicKey(), 'inventory'),
  borsh.vec(borsh.u32(), 'achievements'),
  borsh.i64('created_at'),
]);
```

---

## Best Practices & Checklist

### Best Practices

1. **Start with IDL-Only Migration**
   - Lowest risk, immediate benefits
   - Validate LUMOS output before deeper integration

2. **Verify Space Calculations**
   ```bash
   # Compare LUMOS output with existing LEN constants
   lumos anchor space schema.lumos
   ```

3. **Mark PDA Seeds with `#[key]`**
   - Documents which fields are used in seeds
   - Helps tooling understand account relationships

4. **Always Use `#[max(N)]` for Strings and Vectors**
   - Required for accurate space calculation
   - Prevents runtime surprises

5. **Keep Schema as Source of Truth**
   - Edit schema.lumos, not generated files
   - Add generation to CI/CD pipeline

6. **Version Your Schema**
   ```lumos
   // schema.lumos v1.2.0
   #[solana]
   #[account]
   struct MyAccount { ... }
   ```

7. **Validate on Every PR**
   ```yaml
   # .github/workflows/ci.yml
   - name: Validate Schema
     run: lumos validate schema.lumos

   - name: Check Space
     run: lumos anchor space schema.lumos
   ```

### Migration Checklist

**Pre-Migration:**
- [ ] Audit all `#[account]` structs in existing code
- [ ] Document current space calculations (LEN constants)
- [ ] Identify PDA seeds and patterns
- [ ] Create backup branch

**Schema Creation:**
- [ ] Create schema.lumos file
- [ ] Add all account types with `#[solana]` and `#[account]`
- [ ] Add `#[key]` for PDA seed fields
- [ ] Add `#[max(N)]` for strings and vectors
- [ ] Add non-account types (enums, data structs)

**Validation:**
- [ ] Run `lumos validate schema.lumos`
- [ ] Compare space calculations with existing LEN
- [ ] Generate IDL and compare with existing

**Integration:**
- [ ] Generate TypeScript types
- [ ] Update frontend imports
- [ ] Add generation scripts to package.json
- [ ] Update CI/CD pipeline

**Cleanup (Optional):**
- [ ] Remove manual LEN constants (use generated)
- [ ] Remove manual TypeScript types
- [ ] Update documentation

---

## Next Steps

After migrating your Anchor project:

1. **Explore Full Generation:**
   - Use `lumos anchor generate` for new features
   - Let LUMOS scaffold complete instruction contexts

2. **Read Related Guides:**
   - [Migration: TypeScript → LUMOS](/guides/migration-typescript) - Frontend type migration
   - [LUMOS + web3.js Integration](/guides/web3js-integration) - Frontend patterns
   - [LUMOS + Solana CLI Integration](/guides/solana-cli-integration) - Deployment workflows

3. **Use Case Examples:**
   - [Gaming Projects](/guides/use-cases/gaming) - Complex state management
   - [NFT Marketplaces](/guides/use-cases/nft) - Metaplex integration
   - [DeFi Protocols](/guides/use-cases/defi) - Staking and vesting

---

**Related Guides:**
- [Migration: TypeScript → LUMOS](/guides/migration-typescript)
- [LUMOS + web3.js Integration](/guides/web3js-integration)
- [LUMOS + Solana CLI Integration](/guides/solana-cli-integration)
- [Gaming Use Case](/guides/use-cases/gaming)
- [NFT Use Case](/guides/use-cases/nft)
- [DeFi Use Case](/guides/use-cases/defi)
