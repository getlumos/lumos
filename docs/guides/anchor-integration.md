# LUMOS + Anchor Integration Guide

> **Build type-safe Anchor programs with schema-first development**

This guide covers building NEW Anchor programs with LUMOS from the ground up. Learn the schema-first workflow, generate complete Anchor programs, and leverage instruction contexts for maximum type safety.

> **📦 Version Compatibility**
>
> This guide requires **LUMOS CLI v0.1.1+** with Anchor plugin support.
> Install: `cargo install lumos-cli` or `npm install -g @getlumos/cli@0.1.0`
>
> Key commands: `lumos anchor generate`, `lumos anchor idl`, `lumos anchor space`

---

## Table of Contents

1. [Introduction & Schema-First Philosophy](#introduction--schema-first-philosophy)
2. [Quick Start](#quick-start)
3. [Complete Generated Output](#complete-generated-output)
4. [Account Types Deep Dive](#account-types-deep-dive)
5. [Instruction Contexts](#instruction-contexts)
6. [Anchor Attribute Reference](#anchor-attribute-reference)
7. [Development Workflow](#development-workflow)
8. [Project Structure](#project-structure)
9. [Testing Patterns](#testing-patterns)
10. [Complete Example: Vault Program](#complete-example-vault-program)

---

## Prerequisites

Before starting this guide, you should have:

- **Anchor fundamentals** - Familiarity with the [Anchor framework](https://www.anchor-lang.com/)
- **Basic Rust knowledge** - Understanding of structs, enums, and traits
- **Solana account model** - How accounts, PDAs, and programs work
- **LUMOS CLI installed** - `cargo install lumos-cli`

**Recommended reading:**
- [LUMOS + Solana CLI Integration](/guides/solana-cli-integration) - Deployment workflows
- [Migration: Anchor → LUMOS](/guides/migration-anchor) - If adding to existing project

---

## Introduction & Schema-First Philosophy

### Traditional Anchor Development

The typical Anchor workflow involves writing Rust code first:

```
1. Write account structs in lib.rs
2. Manually calculate space (8 + 32 + 8 + ...)
3. Write #[derive(Accounts)] contexts
4. Implement instruction handlers
5. Hope IDL stays synchronized
6. Manually write TypeScript types
```

**Problems with this approach:**
- Space calculations are error-prone
- IDL can drift from actual code
- TypeScript types must be maintained separately
- No single source of truth

### Schema-First with LUMOS

LUMOS inverts the workflow:

```
1. Define schema.lumos (accounts + instructions)
2. Generate Rust, IDL, and TypeScript
3. Add business logic to handlers
4. Build with anchor build
```

**Benefits:**
- Single source of truth
- Automatic space calculations
- Generated IDL always matches
- TypeScript types guaranteed in sync
- Precision warnings for u64 fields

### What Gets Generated vs What You Write

| LUMOS Generates | You Write |
|-----------------|-----------|
| Account struct definitions | Instruction handler logic |
| `#[derive(Accounts)]` contexts | Business rules and validation |
| Space constants (LEN) | Custom errors |
| Anchor IDL (JSON) | Events (optional) |
| TypeScript interfaces | Additional helper functions |
| Borsh schemas | Tests |

### The LUMOS + Anchor Pipeline

```
schema.lumos
     │
     ▼
┌────────────────────────────────────────┐
│         lumos anchor generate          │
└────────────────────────────────────────┘
     │
     ├──► programs/name/src/lib.rs  (Rust)
     ├──► target/idl/name.json      (IDL)
     └──► app/src/types.ts          (TypeScript)

     │
     ▼
┌────────────────────────────────────────┐
│  Add handlers  →  anchor build/test    │
└────────────────────────────────────────┘
```

---

## Quick Start

Get from zero to working Anchor program in 5 minutes.

### Step 1: Initialize Anchor Project

```bash
# Create new Anchor project
anchor init my_vault
cd my_vault

# Create schemas directory
mkdir schemas
```

### Step 2: Create LUMOS Schema

```bash
cat > schemas/schema.lumos << 'EOF'
// Vault Program Schema
// Single source of truth for all types

#[solana]
#[account]
struct Vault {
    owner: PublicKey,
    balance: u64,
    bump: u8,
}

#[solana]
#[instruction]
struct Initialize {
    #[anchor(init, payer = owner, space = 8 + 41, seeds = [b"vault", owner.key().as_ref()], bump)]
    vault: Vault,

    #[anchor(mut)]
    owner: Signer,

    system_program: SystemProgram,
}

#[solana]
#[instruction]
struct Deposit {
    #[anchor(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump)]
    vault: Vault,

    #[anchor(mut)]
    owner: Signer,

    system_program: SystemProgram,
}

// Built-in type stubs
#[solana]
struct Signer {}

#[solana]
struct SystemProgram {}
EOF
```

### Step 3: Validate Schema

```bash
lumos validate schemas/schema.lumos
```

**Expected output:**
```
✓ Parsed 5 types
✓ All type references valid
✓ Schema is valid
```

### Step 4: Generate Anchor Program

```bash
# Get your program ID
PROGRAM_ID=$(solana-keygen pubkey target/deploy/my_vault-keypair.json 2>/dev/null || echo "11111111111111111111111111111111")

# Generate everything
lumos anchor generate schemas/schema.lumos \
  --name my_vault \
  --address "$PROGRAM_ID" \
  --typescript \
  --output .
```

**Generated files:**
```
programs/my_vault/src/lib.rs    # Anchor program
target/idl/my_vault.json        # Anchor IDL
app/src/types.ts                # TypeScript types
```

### Step 5: Add Handler Logic

Edit `programs/my_vault/src/lib.rs` and implement the handlers:

```rust
#[program]
pub mod my_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.owner.key();
        vault.balance = 0;
        vault.bump = ctx.bumps.vault;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Transfer SOL from owner to vault
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;

        ctx.accounts.vault.balance += amount;
        Ok(())
    }
}
```

### Step 6: Build and Test

```bash
# Build the program
anchor build

# Run tests
anchor test
```

**Congratulations!** You've built your first LUMOS + Anchor program.

---

## Complete Generated Output

Understanding what LUMOS generates helps you work effectively with the output.

### Generated Rust Program (lib.rs)

```rust
// ═══════════════════════════════════════════════════════════════════════════════
// AUTO-GENERATED BY LUMOS v0.1.1 - DO NOT EDIT
// Source: schemas/schema.lumos
// Program: my_vault
// Version: 0.1.0
// ═══════════════════════════════════════════════════════════════════════════════

use anchor_lang::prelude::*;

declare_id!("YOUR_PROGRAM_ID_HERE");

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Vault account storing user funds
#[account]
pub struct Vault {
    /// Owner of this vault
    pub owner: Pubkey,
    /// Current balance in lamports
    pub balance: u64,
    /// PDA bump seed
    pub bump: u8,
}

impl Vault {
    /// Account size: 8 (discriminator) + 32 (owner) + 8 (balance) + 1 (bump)
    pub const LEN: usize = 49;
}

// ═══════════════════════════════════════════════════════════════════════════════
// INSTRUCTION CONTEXTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize a new vault for a user
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The vault account to create
    #[account(
        init,
        payer = owner,
        space = Vault::LEN,
        seeds = [b"vault", owner.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    /// The user creating the vault (pays for account creation)
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Required for account creation
    pub system_program: Program<'info, System>,
}

/// Deposit funds into a vault
#[derive(Accounts)]
pub struct Deposit<'info> {
    /// The vault to deposit into
    #[account(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    /// The vault owner making the deposit
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Required for SOL transfer
    pub system_program: Program<'info, System>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PROGRAM
// Add your instruction handlers below
// ═══════════════════════════════════════════════════════════════════════════════

#[program]
pub mod my_vault {
    use super::*;

    /// Initialize a new vault
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // TODO: Implement initialization logic
        Ok(())
    }

    /// Deposit funds into vault
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // TODO: Implement deposit logic
        Ok(())
    }
}
```

### Generated Anchor IDL (JSON)

```json
{
  "version": "0.1.0",
  "name": "my_vault",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [
        {
          "name": "vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "deposit",
      "accounts": [
        {
          "name": "vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "Vault",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "owner", "type": "publicKey" },
          { "name": "balance", "type": "u64" },
          { "name": "bump", "type": "u8" }
        ]
      }
    }
  ],
  "metadata": {
    "address": "YOUR_PROGRAM_ID_HERE"
  }
}
```

### Generated TypeScript Types

```typescript
// ═══════════════════════════════════════════════════════════════════════════════
// AUTO-GENERATED BY LUMOS v0.1.1 - DO NOT EDIT
// Source: schemas/schema.lumos
// ═══════════════════════════════════════════════════════════════════════════════

import { PublicKey } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TYPES
// ═══════════════════════════════════════════════════════════════════════════════

export interface Vault {
  owner: PublicKey;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
   * For Solana lamports or large values, ensure they stay within safe range.
   */
  balance: number;
  bump: number;
}

export const VaultSchema = borsh.struct([
  borsh.publicKey('owner'),
  borsh.u64('balance'),
  borsh.u8('bump'),
]);

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

export const VAULT_SIZE = 49;

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

export function decodeVault(data: Buffer): Vault {
  return VaultSchema.decode(data);
}
```

---

## Account Types Deep Dive

### Basic Account Definition

Account types represent on-chain state:

```lumos
#[solana]
#[account]
struct UserProfile {
    owner: PublicKey,
    #[max(32)]
    username: String,
    level: u16,
    experience: u64,
    created_at: i64,
}
```

**Generated Rust:**

```rust
#[account]
pub struct UserProfile {
    pub owner: Pubkey,
    pub username: String,
    pub level: u16,
    pub experience: u64,
    pub created_at: i64,
}

impl UserProfile {
    /// 8 (discriminator) + 32 (owner) + 36 (username: 4 + 32) + 2 + 8 + 8 = 94
    pub const LEN: usize = 94;
}
```

### The `#[key]` Attribute

Mark fields used in PDA seeds:

```lumos
#[solana]
#[account]
struct PlayerVault {
    #[key]  // This field is used in PDA seeds
    player: PublicKey,
    #[key]  // This field is also used in PDA seeds
    game_id: u64,
    balance: u64,
}
```

The `#[key]` attribute:
- Documents which fields are PDA seed components
- Helps tooling understand account relationships
- Useful for client-side PDA derivation

### String and Array Constraints

Always specify max lengths for variable-size types:

```lumos
#[solana]
#[account]
struct GameItem {
    owner: PublicKey,

    #[max(50)]              // String: 4 + 50 = 54 bytes
    name: String,

    #[max(200)]             // String: 4 + 200 = 204 bytes
    description: String,

    #[max(10)]              // Vec: 4 + (32 * 10) = 324 bytes
    attributes: [PublicKey],

    stats: [u16; 6],        // Fixed array: 2 * 6 = 12 bytes
}
```

### Optional Fields

Use `Option<T>` for nullable fields:

```lumos
#[solana]
#[account]
struct Listing {
    seller: PublicKey,
    price: u64,
    expires_at: Option<i64>,     // 1 + 8 = 9 bytes
    buyer: Option<PublicKey>,    // 1 + 32 = 33 bytes
}
```

### Nested Types

Reference other types in your schema:

```lumos
#[solana]
struct Stats {
    power: u16,
    defense: u16,
    speed: u16,
}

#[solana]
#[account]
struct Character {
    owner: PublicKey,
    #[max(20)]
    name: String,
    stats: Stats,        // Embedded struct
    level: u8,
}
```

### Enums in Accounts

```lumos
#[solana]
enum CharacterClass {
    Warrior,
    Mage,
    Archer,
    Healer,
}

#[solana]
enum GameState {
    Waiting,
    Active,
    Paused,
    Finished { winner: PublicKey },
}

#[solana]
#[account]
struct Player {
    owner: PublicKey,
    class: CharacterClass,    // 1 byte (simple enum)
    game_state: GameState,    // 1 + 32 bytes (enum with data)
}
```

---

## Instruction Contexts

Instruction contexts define the accounts required for each instruction.

### The `#[instruction]` Attribute

Mark a struct as an Anchor Accounts context:

```lumos
#[solana]
#[instruction]
struct InitializeGame {
    #[anchor(init, payer = creator, space = 8 + 256)]
    game: GameAccount,

    #[anchor(mut)]
    creator: Signer,

    system_program: SystemProgram,
}
```

**Generated Rust:**

```rust
#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(init, payer = creator, space = 8 + 256)]
    pub game: Account<'info, GameAccount>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

### Type Inference

LUMOS automatically wraps types in appropriate Anchor wrappers:

| LUMOS Field Type | Generated Anchor Type |
|------------------|----------------------|
| `MyAccount` (has `#[account]`) | `Account<'info, MyAccount>` |
| `Signer` | `Signer<'info>` |
| `SystemProgram` | `Program<'info, System>` |
| `TokenProgram` | `Program<'info, Token>` |
| `AssociatedTokenProgram` | `Program<'info, AssociatedToken>` |
| `Rent` | `Sysvar<'info, Rent>` |
| `Clock` | `Sysvar<'info, Clock>` |
| `Option<MyAccount>` | `Option<Account<'info, MyAccount>>` |
| `UncheckedAccount` | `UncheckedAccount<'info>` |

### Built-in Type Stubs

Define these stubs in your schema to use Anchor's built-in types:

```lumos
// Signer account (must sign transaction)
#[solana]
struct Signer {}

// System program
#[solana]
struct SystemProgram {}

// Token program
#[solana]
struct TokenProgram {}

// Associated token program
#[solana]
struct AssociatedTokenProgram {}

// Rent sysvar
#[solana]
struct Rent {}

// Clock sysvar
#[solana]
struct Clock {}

// Unchecked account (use carefully!)
#[solana]
struct UncheckedAccount {}
```

### Instruction Arguments

Fields without `#[anchor(...)]` become instruction arguments:

```lumos
#[solana]
#[instruction]
struct Transfer {
    #[anchor(mut, has_one = owner)]
    from_vault: Vault,

    #[anchor(mut)]
    to_vault: Vault,

    owner: Signer,

    // These become instruction arguments (no anchor attribute)
    amount: u64,
    memo: String,
}
```

**Generated handler signature:**

```rust
pub fn transfer(
    ctx: Context<Transfer>,
    amount: u64,      // Instruction argument
    memo: String,     // Instruction argument
) -> Result<()>
```

### Complex Instruction Example

```lumos
#[solana]
#[instruction]
struct PlaceBid {
    // Mutable auction account with validation
    #[anchor(mut, has_one = highest_bidder, constraint = "!auction.is_ended")]
    auction: Auction,

    // The bidder (signer)
    #[anchor(mut)]
    bidder: Signer,

    // Previous highest bidder (for refund)
    #[anchor(mut)]
    highest_bidder: UncheckedAccount,

    // PDA for escrow
    #[anchor(mut, seeds = [b"escrow", auction.key().as_ref()], bump)]
    escrow: Vault,

    system_program: SystemProgram,
}
```

---

## Anchor Attribute Reference

Complete reference for all supported Anchor attributes.

### Account Initialization

| Attribute | Description | Example |
|-----------|-------------|---------|
| `init` | Initialize new account | `#[anchor(init)]` |
| `init_if_needed` | Init only if doesn't exist | `#[anchor(init_if_needed)]` |
| `payer = <account>` | Account paying for creation | `#[anchor(payer = user)]` |
| `space = <expr>` | Account size in bytes | `#[anchor(space = 8 + 64)]` |
| `zero` | Zero out account memory | `#[anchor(zero)]` |

**Example: Account initialization**
```lumos
#[anchor(init, payer = authority, space = 8 + 128)]
config: ConfigAccount,
```

### PDA (Program Derived Address)

| Attribute | Description | Example |
|-----------|-------------|---------|
| `seeds = [...]` | PDA seed components | `#[anchor(seeds = [b"vault", user.key().as_ref()])]` |
| `bump` | Auto-derive bump seed | `#[anchor(bump)]` |
| `bump = <expr>` | Use stored bump | `#[anchor(bump = vault.bump)]` |

**Example: PDA with seeds**
```lumos
#[anchor(
    init,
    payer = user,
    space = 8 + 64,
    seeds = [b"player", user.key().as_ref(), game.key().as_ref()],
    bump
)]
player_account: PlayerAccount,
```

### Account Mutability

| Attribute | Description | Example |
|-----------|-------------|---------|
| `mut` | Account is mutable | `#[anchor(mut)]` |

**Example: Mutable account**
```lumos
#[anchor(mut)]
user: Signer,

#[anchor(mut, has_one = owner)]
vault: Vault,
```

### Validation Constraints

| Attribute | Description | Example |
|-----------|-------------|---------|
| `has_one = <field>` | Field must match account | `#[anchor(has_one = owner)]` |
| `constraint = "<expr>"` | Custom constraint | `#[anchor(constraint = "amount > 0")]` |
| `address = "<pubkey>"` | Exact address check | `#[anchor(address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")]` |
| `owner = <program>` | Account owner check | `#[anchor(owner = token_program)]` |

**Example: Multiple constraints**
```lumos
#[anchor(
    mut,
    has_one = owner,
    has_one = token_mint,
    constraint = "vault.balance >= amount @ CustomError::InsufficientFunds"
)]
vault: Vault,
```

### Account Operations

| Attribute | Description | Example |
|-----------|-------------|---------|
| `close = <account>` | Close and transfer lamports | `#[anchor(close = owner)]` |
| `realloc = <size>` | Resize account | `#[anchor(realloc = new_size)]` |
| `realloc::payer = <acc>` | Payer for realloc | `#[anchor(realloc::payer = user)]` |
| `realloc::zero = <bool>` | Zero new memory | `#[anchor(realloc::zero = true)]` |

**Example: Close account**
```lumos
#[anchor(mut, close = owner, has_one = owner)]
vault: Vault,

#[anchor(mut)]
owner: Signer,
```

### Combined Attributes

Attributes can be combined:

```lumos
#[solana]
#[instruction]
struct ComplexInstruction {
    // Initialize PDA with multiple constraints
    #[anchor(
        init,
        payer = authority,
        space = 8 + 256,
        seeds = [b"config", authority.key().as_ref()],
        bump,
        constraint = "!existing_config.is_some() @ ErrorCode::AlreadyInitialized"
    )]
    config: ConfigAccount,

    // Mutable with validation
    #[anchor(
        mut,
        has_one = authority,
        constraint = "vault.balance >= min_balance @ ErrorCode::InsufficientBalance"
    )]
    vault: Vault,

    // Authority signer
    #[anchor(mut)]
    authority: Signer,

    // Optional account
    existing_config: Option<ConfigAccount>,

    system_program: SystemProgram,
}
```

---

## Development Workflow

### Complete Workflow Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    DEVELOPMENT CYCLE                        │
└─────────────────────────────────────────────────────────────┘

    ┌──────────────┐
    │ 1. DESIGN    │  Edit schema.lumos
    │    SCHEMA    │  Define accounts & instructions
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐
    │ 2. VALIDATE  │  lumos validate schema.lumos
    │              │  Catch errors early
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐
    │ 3. GENERATE  │  lumos anchor generate ...
    │              │  Create Rust, IDL, TypeScript
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐
    │ 4. IMPLEMENT │  Edit lib.rs
    │    HANDLERS  │  Add business logic
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐
    │ 5. BUILD     │  anchor build
    │              │  Compile program
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐
    │ 6. TEST      │  anchor test
    │              │  Run test suite
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐
    │ 7. DEPLOY    │  anchor deploy
    │              │  Deploy to cluster
    └──────────────┘
```

### Phase 1: Schema Design

```bash
# Create/edit your schema
vim schemas/schema.lumos

# Validate syntax and references
lumos validate schemas/schema.lumos
```

**Design tips:**
- Start with account types
- Add instruction contexts one at a time
- Use `#[key]` to document PDA seeds
- Add `#[max(N)]` for all strings and vectors

### Phase 2: Code Generation

```bash
# Preview what will be generated
lumos anchor generate schemas/schema.lumos --dry-run

# Generate all files
lumos anchor generate schemas/schema.lumos \
  --name my_program \
  --address $(solana-keygen pubkey target/deploy/my_program-keypair.json) \
  --typescript \
  --output .
```

**Generated files:**
- `programs/my_program/src/lib.rs` - Rust program
- `target/idl/my_program.json` - Anchor IDL
- `app/src/types.ts` - TypeScript types

### Phase 3: Implement Handlers

Edit the generated `lib.rs` to add your business logic:

```rust
#[program]
pub mod my_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let account = &mut ctx.accounts.my_account;
        account.owner = ctx.accounts.owner.key();
        account.created_at = Clock::get()?.unix_timestamp;
        account.bump = ctx.bumps.my_account;

        emit!(InitializeEvent {
            owner: account.owner,
            timestamp: account.created_at,
        });

        Ok(())
    }

    pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> {
        require!(amount > 0, CustomError::InvalidAmount);

        let account = &mut ctx.accounts.my_account;
        account.balance = account.balance
            .checked_add(amount)
            .ok_or(CustomError::Overflow)?;

        Ok(())
    }
}
```

### Phase 4: Build

```bash
# Build the program
anchor build

# Check for warnings
cargo clippy --package my_program
```

### Phase 5: Test

```bash
# Run all tests
anchor test

# Run specific test
anchor test --skip-local-validator -- --test initialize

# Run with logs
RUST_LOG=debug anchor test
```

### Phase 6: Deploy

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet (be careful!)
anchor deploy --provider.cluster mainnet

# Verify deployment
solana program show <PROGRAM_ID>
```

### CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Solana
        run: |
          sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
          echo "$HOME/.local/share/solana/install/active_release/bin" >> $GITHUB_PATH

      - name: Install Anchor
        run: cargo install --git https://github.com/coral-xyz/anchor anchor-cli

      - name: Install LUMOS
        run: cargo install lumos-cli

      - name: Validate Schema
        run: lumos validate schemas/schema.lumos

      - name: Generate Code
        run: |
          lumos anchor generate schemas/schema.lumos \
            --name my_program \
            --address 11111111111111111111111111111111

      - name: Build
        run: anchor build

      - name: Test
        run: anchor test
```

### Regeneration on Schema Changes

When you modify `schema.lumos`:

```bash
# 1. Validate changes
lumos validate schemas/schema.lumos

# 2. Regenerate (overwrites generated sections)
lumos anchor generate schemas/schema.lumos \
  --name my_program \
  --address $PROGRAM_ID \
  --typescript

# 3. Review changes
git diff programs/my_program/src/lib.rs

# 4. Rebuild
anchor build
```

---

## Project Structure

### Recommended Directory Layout

```
my-anchor-project/
├── Anchor.toml                      # Anchor configuration
├── Cargo.toml                       # Workspace root
├── package.json                     # Node.js dependencies
│
├── schemas/
│   └── schema.lumos                 # 🔑 SOURCE OF TRUTH
│
├── programs/
│   └── my_program/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs               # Generated + handlers
│           ├── instructions/        # Organized handlers
│           │   ├── mod.rs
│           │   ├── initialize.rs
│           │   ├── deposit.rs
│           │   └── withdraw.rs
│           ├── state/               # Additional state logic
│           │   ├── mod.rs
│           │   └── helpers.rs
│           └── errors.rs            # Custom errors
│
├── target/
│   ├── idl/
│   │   └── my_program.json          # Generated IDL
│   └── types/
│       └── my_program.ts            # Anchor types
│
├── app/                             # Frontend application
│   ├── package.json
│   ├── tsconfig.json
│   └── src/
│       ├── types.ts                 # Generated types
│       ├── client.ts                # Program client
│       ├── hooks/                   # React hooks
│       └── utils/                   # Utilities
│
├── tests/
│   └── my_program.ts                # Anchor tests
│
├── scripts/
│   ├── generate.sh                  # Generation script
│   └── deploy.sh                    # Deployment script
│
└── .gitignore
```

### What to Commit vs Ignore

**Commit these:**
```
schemas/schema.lumos                 # Source of truth
programs/*/src/lib.rs                # With your handlers
programs/*/src/instructions/         # Handler modules
programs/*/src/errors.rs             # Custom errors
tests/                               # Test files
app/src/client.ts                    # Custom client code
```

**Gitignore these (optional):**
```gitignore
# Generated files (can regenerate from schema)
target/idl/*.json
app/src/types.ts

# Build artifacts
target/
node_modules/
.anchor/
```

### Organizing Large Programs

For programs with many instructions:

```
programs/my_program/src/
├── lib.rs                           # Entry point
│   └── mod instructions;
│   └── mod state;
│   └── mod errors;
│   └── mod events;
│
├── instructions/
│   ├── mod.rs                       # Re-exports
│   ├── admin/
│   │   ├── mod.rs
│   │   ├── initialize.rs
│   │   └── update_config.rs
│   ├── user/
│   │   ├── mod.rs
│   │   ├── deposit.rs
│   │   └── withdraw.rs
│   └── game/
│       ├── mod.rs
│       ├── start.rs
│       └── end.rs
│
├── state/
│   ├── mod.rs
│   └── helpers.rs                   # State helper methods
│
├── errors.rs                        # Custom error codes
│
└── events.rs                        # Event definitions
```

**lib.rs with modules:**

```rust
use anchor_lang::prelude::*;

mod instructions;
mod state;
mod errors;
mod events;

use instructions::*;
use errors::*;
use events::*;

declare_id!("YOUR_PROGRAM_ID");

// Generated accounts and contexts here...

#[program]
pub mod my_program {
    use super::*;

    // Admin instructions
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::admin::initialize::handler(ctx)
    }

    pub fn update_config(ctx: Context<UpdateConfig>, new_fee: u64) -> Result<()> {
        instructions::admin::update_config::handler(ctx, new_fee)
    }

    // User instructions
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::user::deposit::handler(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        instructions::user::withdraw::handler(ctx, amount)
    }
}
```

---

## Testing Patterns

### Basic Anchor Test Setup

```typescript
// tests/my_program.ts
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MyProgram } from "../target/types/my_program";
import { PublicKey, SystemProgram, Keypair } from "@solana/web3.js";
import { expect } from "chai";

describe("my_program", () => {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.MyProgram as Program<MyProgram>;
  const user = provider.wallet;

  // Test accounts
  let vaultPda: PublicKey;
  let vaultBump: number;

  before(async () => {
    // Derive PDA
    [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );
  });

  it("initializes vault", async () => {
    await program.methods
      .initialize()
      .accounts({
        vault: vaultPda,
        owner: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Fetch and verify
    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.owner.toString()).to.equal(user.publicKey.toString());
    expect(vault.balance.toNumber()).to.equal(0);
    expect(vault.bump).to.equal(vaultBump);
  });

  it("deposits funds", async () => {
    const depositAmount = 1_000_000_000; // 1 SOL

    await program.methods
      .deposit(new anchor.BN(depositAmount))
      .accounts({
        vault: vaultPda,
        owner: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.balance.toNumber()).to.equal(depositAmount);
  });

  it("fails with insufficient funds", async () => {
    try {
      await program.methods
        .withdraw(new anchor.BN(999_999_999_999))
        .accounts({
          vault: vaultPda,
          owner: user.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("Should have thrown error");
    } catch (err) {
      expect(err.message).to.include("InsufficientFunds");
    }
  });
});
```

### Using Generated TypeScript Types

```typescript
// Import generated types
import { Vault, VaultSchema, decodeVault } from "../app/src/types";

it("decodes vault with generated schema", async () => {
  const accountInfo = await provider.connection.getAccountInfo(vaultPda);

  // Skip 8-byte discriminator
  const data = accountInfo.data.slice(8);

  // Decode using generated schema
  const vault: Vault = decodeVault(data);

  expect(vault.owner.toString()).to.equal(user.publicKey.toString());
  expect(typeof vault.balance).to.equal("number");
});
```

### Testing Error Cases

```typescript
import { AnchorError } from "@coral-xyz/anchor";

it("rejects unauthorized access", async () => {
  const attacker = Keypair.generate();

  try {
    await program.methods
      .withdraw(new anchor.BN(100))
      .accounts({
        vault: vaultPda,
        owner: attacker.publicKey,  // Wrong owner
        systemProgram: SystemProgram.programId,
      })
      .signers([attacker])
      .rpc();
    expect.fail("Should have thrown");
  } catch (err) {
    expect(err).to.be.instanceOf(AnchorError);
    expect(err.error.errorCode.code).to.equal("ConstraintHasOne");
  }
});
```

### Testing Events

```typescript
it("emits initialize event", async () => {
  const listener = program.addEventListener("InitializeEvent", (event) => {
    expect(event.owner.toString()).to.equal(user.publicKey.toString());
    expect(event.timestamp.toNumber()).to.be.greaterThan(0);
  });

  await program.methods
    .initialize()
    .accounts({ /* ... */ })
    .rpc();

  // Allow time for event
  await new Promise((resolve) => setTimeout(resolve, 1000));
  program.removeEventListener(listener);
});
```

---

## Complete Example: Vault Program

A complete working example demonstrating all concepts.

### Schema (schemas/schema.lumos)

```lumos
// ═══════════════════════════════════════════════════════════════════════════════
// VAULT PROGRAM SCHEMA
// A simple vault for depositing and withdrawing SOL
// ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// ACCOUNT TYPES
// ─────────────────────────────────────────────────────────────────────────────────

#[solana]
#[account]
struct Vault {
    /// Owner of this vault (used in PDA seeds)
    #[key]
    owner: PublicKey,
    /// Current balance in lamports
    balance: u64,
    /// Total deposited over lifetime
    total_deposited: u64,
    /// Total withdrawn over lifetime
    total_withdrawn: u64,
    /// Creation timestamp
    created_at: i64,
    /// Last activity timestamp
    last_activity: i64,
    /// PDA bump seed
    bump: u8,
}

#[solana]
#[account]
struct VaultConfig {
    /// Admin authority
    authority: PublicKey,
    /// Minimum deposit amount
    min_deposit: u64,
    /// Maximum deposit amount (0 = unlimited)
    max_deposit: u64,
    /// Fee in basis points (100 = 1%)
    fee_basis_points: u16,
    /// Fee recipient
    fee_recipient: PublicKey,
    /// Is the vault system paused
    is_paused: bool,
    /// Config bump
    bump: u8,
}

// ─────────────────────────────────────────────────────────────────────────────────
// INSTRUCTION CONTEXTS
// ─────────────────────────────────────────────────────────────────────────────────

#[solana]
#[instruction]
struct InitializeConfig {
    #[anchor(init, payer = authority, space = 8 + 113, seeds = [b"config"], bump)]
    config: VaultConfig,

    #[anchor(mut)]
    authority: Signer,

    system_program: SystemProgram,
}

#[solana]
#[instruction]
struct InitializeVault {
    #[anchor(init, payer = owner, space = 8 + 73, seeds = [b"vault", owner.key().as_ref()], bump)]
    vault: Vault,

    #[anchor(seeds = [b"config"], bump = config.bump)]
    config: VaultConfig,

    #[anchor(mut)]
    owner: Signer,

    system_program: SystemProgram,
}

#[solana]
#[instruction]
struct Deposit {
    #[anchor(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump, has_one = owner)]
    vault: Vault,

    #[anchor(seeds = [b"config"], bump = config.bump, constraint = "!config.is_paused")]
    config: VaultConfig,

    #[anchor(mut)]
    owner: Signer,

    system_program: SystemProgram,
}

#[solana]
#[instruction]
struct Withdraw {
    #[anchor(mut, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump, has_one = owner)]
    vault: Vault,

    #[anchor(seeds = [b"config"], bump = config.bump, constraint = "!config.is_paused")]
    config: VaultConfig,

    #[anchor(mut)]
    owner: Signer,

    system_program: SystemProgram,
}

#[solana]
#[instruction]
struct CloseVault {
    #[anchor(mut, close = owner, seeds = [b"vault", owner.key().as_ref()], bump = vault.bump, has_one = owner, constraint = "vault.balance == 0")]
    vault: Vault,

    #[anchor(mut)]
    owner: Signer,
}

#[solana]
#[instruction]
struct UpdateConfig {
    #[anchor(mut, seeds = [b"config"], bump = config.bump, has_one = authority)]
    config: VaultConfig,

    authority: Signer,
}

// ─────────────────────────────────────────────────────────────────────────────────
// BUILT-IN TYPE STUBS
// ─────────────────────────────────────────────────────────────────────────────────

#[solana]
struct Signer {}

#[solana]
struct SystemProgram {}
```

### Generated + Implemented Handlers (lib.rs)

```rust
use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("VauLT1111111111111111111111111111111111111");

// ... Generated accounts and contexts ...

// Custom errors
#[error_code]
pub enum VaultError {
    #[msg("Deposit amount is below minimum")]
    BelowMinimum,
    #[msg("Deposit amount exceeds maximum")]
    ExceedsMaximum,
    #[msg("Insufficient vault balance")]
    InsufficientBalance,
    #[msg("Arithmetic overflow")]
    Overflow,
}

// Events
#[event]
pub struct DepositEvent {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
    pub vault: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
    pub timestamp: i64,
}

#[program]
pub mod vault_program {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        min_deposit: u64,
        max_deposit: u64,
        fee_basis_points: u16,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.min_deposit = min_deposit;
        config.max_deposit = max_deposit;
        config.fee_basis_points = fee_basis_points;
        config.fee_recipient = ctx.accounts.authority.key();
        config.is_paused = false;
        config.bump = ctx.bumps.config;
        Ok(())
    }

    pub fn initialize_vault(ctx: Context<InitializeVault>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        vault.owner = ctx.accounts.owner.key();
        vault.balance = 0;
        vault.total_deposited = 0;
        vault.total_withdrawn = 0;
        vault.created_at = clock.unix_timestamp;
        vault.last_activity = clock.unix_timestamp;
        vault.bump = ctx.bumps.vault;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let config = &ctx.accounts.config;
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        // Validate amount
        require!(amount >= config.min_deposit, VaultError::BelowMinimum);
        if config.max_deposit > 0 {
            require!(amount <= config.max_deposit, VaultError::ExceedsMaximum);
        }

        // Transfer SOL to vault
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.owner.to_account_info(),
                to: vault.to_account_info(),
            },
        );
        system_program::transfer(cpi_context, amount)?;

        // Update vault state
        vault.balance = vault.balance.checked_add(amount).ok_or(VaultError::Overflow)?;
        vault.total_deposited = vault.total_deposited.checked_add(amount).ok_or(VaultError::Overflow)?;
        vault.last_activity = clock.unix_timestamp;

        // Emit event
        emit!(DepositEvent {
            vault: vault.key(),
            owner: vault.owner,
            amount,
            new_balance: vault.balance,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let clock = Clock::get()?;

        // Validate balance
        require!(vault.balance >= amount, VaultError::InsufficientBalance);

        // Transfer SOL from vault (PDA signer)
        let owner_key = ctx.accounts.owner.key();
        let seeds = &[
            b"vault",
            owner_key.as_ref(),
            &[vault.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: vault.to_account_info(),
                to: ctx.accounts.owner.to_account_info(),
            },
            signer_seeds,
        );
        system_program::transfer(cpi_context, amount)?;

        // Update vault state
        vault.balance = vault.balance.checked_sub(amount).ok_or(VaultError::Overflow)?;
        vault.total_withdrawn = vault.total_withdrawn.checked_add(amount).ok_or(VaultError::Overflow)?;
        vault.last_activity = clock.unix_timestamp;

        // Emit event
        emit!(WithdrawEvent {
            vault: vault.key(),
            owner: vault.owner,
            amount,
            new_balance: vault.balance,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    pub fn close_vault(ctx: Context<CloseVault>) -> Result<()> {
        // Account closing handled by Anchor's close constraint
        msg!("Vault closed for owner: {}", ctx.accounts.owner.key());
        Ok(())
    }

    pub fn update_config(
        ctx: Context<UpdateConfig>,
        min_deposit: Option<u64>,
        max_deposit: Option<u64>,
        fee_basis_points: Option<u16>,
        is_paused: Option<bool>,
    ) -> Result<()> {
        let config = &mut ctx.accounts.config;

        if let Some(v) = min_deposit {
            config.min_deposit = v;
        }
        if let Some(v) = max_deposit {
            config.max_deposit = v;
        }
        if let Some(v) = fee_basis_points {
            config.fee_basis_points = v;
        }
        if let Some(v) = is_paused {
            config.is_paused = v;
        }

        Ok(())
    }
}
```

### Generate and Build

```bash
# Generate from schema
lumos anchor generate schemas/schema.lumos \
  --name vault_program \
  --address VauLT1111111111111111111111111111111111111 \
  --typescript

# Build
anchor build

# Test
anchor test
```

---

## Next Steps

After mastering LUMOS + Anchor integration:

1. **Explore Migration Guides:**
   - [Migration: TypeScript → LUMOS](/guides/migration-typescript) - Migrate existing TypeScript
   - [Migration: Anchor → LUMOS](/guides/migration-anchor) - Add LUMOS to existing Anchor

2. **Use Case Examples:**
   - [Gaming Projects](/guides/use-cases/gaming) - Complex state management
   - [NFT Marketplaces](/guides/use-cases/nft) - Metaplex integration
   - [DeFi Protocols](/guides/use-cases/defi) - Staking and vesting

3. **Deployment Guides:**
   - [LUMOS + Solana CLI](/guides/solana-cli-integration) - Deployment workflows
   - [LUMOS + web3.js](/guides/web3js-integration) - Frontend integration

---

## Summary

LUMOS + Anchor integration provides:

| Feature | Benefit |
|---------|---------|
| Schema-first development | Single source of truth |
| Auto-generated accounts | No manual space calculations |
| Instruction contexts | Type-safe `#[derive(Accounts)]` |
| Full attribute support | All Anchor constraints supported |
| IDL synchronization | Always matches code |
| TypeScript generation | Frontend types included |

**Get started:**

```bash
# Install LUMOS
cargo install lumos-cli

# Create schema
mkdir schemas && touch schemas/schema.lumos

# Generate Anchor program
lumos anchor generate schemas/schema.lumos --name my_program --typescript

# Build and test
anchor build && anchor test
```

---

**Related Guides:**
- [Client-Side Interaction](/guides/client-side-interaction) - React hooks, state management, and frontend patterns
- [Usage Examples for Generated Code](/guides/usage-examples) - Practical patterns for handlers & clients
- [Migration: TypeScript → LUMOS](/guides/migration-typescript)
- [Migration: Anchor → LUMOS](/guides/migration-anchor)
- [LUMOS + Solana CLI Integration](/guides/solana-cli-integration)
- [LUMOS + web3.js Integration](/guides/web3js-integration)
- [LUMOS + Solv/Jito Integration](/guides/solv-jito-integration) - Liquid staking & MEV
- [Gaming Use Case](/guides/use-cases/gaming)
- [NFT Use Case](/guides/use-cases/nft)
- [DeFi Use Case](/guides/use-cases/defi)
