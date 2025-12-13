# LUMOS Syntax Reference

**Version:** 0.1.0
**Status:** Draft Specification
**File Extension:** `.lumos`

---

## Table of Contents

1. [Overview](#overview)
2. [Basic Structure](#basic-structure)
3. [Type System](#type-system)
4. [Attributes](#attributes)
5. [Solana-Specific Features](#solana-specific-features)
6. [Advanced Features](#advanced-features)
7. [Complete Examples](#complete-examples)

---

## Overview

LUMOS uses a hybrid syntax combining:
- **Rust-style** struct definitions and attributes (familiar to Solana developers)
- **TypeScript-style** conveniences (optional types with `?`)
- **Rust-style attributes** (`#[...]`) for metadata

### Philosophy

- **Concise:** Express intent clearly without boilerplate
- **Familiar:** Leverage existing knowledge (Rust + TypeScript)
- **Explicit:** No magic, clear mappings
- **Extensible:** Room for future Solana features

---

## Basic Structure

### Simple Struct

```rust
struct User {
    id: u64,
    name: string,
    active: bool,
}
```

This generates:

**Rust:**
```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub active: bool,
}
```

**TypeScript:**
```typescript
export interface User {
  id: number;
  name: string;
  active: boolean;
}
```

---

## Type System

### Primitive Types

LUMOS supports both **Rust types** and **TypeScript-friendly aliases**:

| LUMOS Type | Rust Type | TypeScript Type | Notes |
|------------|-----------|-----------------|-------|
| `u8` | `u8` | `number` | Unsigned 8-bit |
| `u16` | `u16` | `number` | Unsigned 16-bit |
| `u32` | `u32` | `number` | Unsigned 32-bit |
| `u64` | `u64` | `number` | Unsigned 64-bit |
| `u128` | `u128` | `bigint` | Unsigned 128-bit |
| `i8` | `i8` | `number` | Signed 8-bit |
| `i16` | `i16` | `number` | Signed 16-bit |
| `i32` | `i32` | `number` | Signed 32-bit |
| `i64` | `i64` | `number` | Signed 64-bit |
| `i128` | `i128` | `bigint` | Signed 128-bit |
| `f32` | `f32` | `number` | 32-bit float |
| `f64` | `f64` | `number` | 64-bit float |
| `bool` | `bool` | `boolean` | Boolean |
| `string` | `String` | `string` | UTF-8 string |
| `number` | `u64` | `number` | Alias for u64 |
| `boolean` | `bool` | `boolean` | Alias for bool |

### TypeScript-Friendly Aliases

```rust
struct Product {
    price: number,      // Maps to u64 in Rust, number in TS
    name: string,       // Maps to String in Rust, string in TS
    available: boolean, // Maps to bool in Rust, boolean in TS
}
```

### Optional Types

Use `?` suffix for optional fields:

```rust
struct User {
    id: u64,
    email?: string,      // Optional field
    verified?: bool,     // Optional field
}
```

**Generates:**

**Rust:**
```rust
pub struct User {
    pub id: u64,
    pub email: Option<String>,
    pub verified: Option<bool>,
}
```

**TypeScript:**
```typescript
export interface User {
  id: number;
  email?: string;
  verified?: boolean;
}
```

### Arrays

```rust
struct Team {
    name: string,
    members: [u64],        // Array of user IDs
    tags: [string],        // Array of strings
}
```

**Generates:**

**Rust:**
```rust
pub struct Team {
    pub name: String,
    pub members: Vec<u64>,
    pub tags: Vec<String>,
}
```

**TypeScript:**
```typescript
export interface Team {
  name: string;
  members: number[];
  tags: string[];
}
```

### Nested Types

```rust
struct Address {
    street: string,
    city: string,
}

struct User {
    name: string,
    address: Address,    // Nested type
}
```

---

## Attributes

Attributes provide metadata and generation hints.

### Global Attributes

#### `#[solana]` - Mark as Solana Program Type

```rust
#[solana]
struct UserAccount {
    owner: PublicKey,
    balance: u64,
}
```

**Effect:**
- Adds Solana-specific derives to Rust
- Generates appropriate TypeScript imports

#### `#[account]` - Mark as Anchor Account

```rust
#[solana]
#[account]
struct UserAccount {
    owner: PublicKey,
    balance: u64,
}
```

**Generates:**

**Rust:**
```rust
#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub balance: u64,
}
```

#### `#[version = "X.Y.Z"]` - Schema Versioning

Track schema versions using semantic versioning:

```rust
#[solana]
#[version = "1.0.0"]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    level: u16,
    experience: u64,
}

#[solana]
#[version = "2.1.3"]
enum GameState {
    Active,
    Paused,
    Finished,
}
```

**Generates:**

**Rust:**
```rust
pub const PLAYERACCOUNT_VERSION: &str = "1.0.0";

#[account]
pub struct PlayerAccount {
    pub wallet: Pubkey,
    pub level: u16,
    pub experience: u64,
}

pub const GAMESTATE_VERSION: &str = "2.1.3";

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum GameState {
    Active,
    Paused,
    Finished,
}
```

**TypeScript:**
```typescript
export const PLAYERACCOUNT_VERSION = "1.0.0";

export interface PlayerAccount {
  wallet: PublicKey;
  level: number;
  experience: number;
}

export const GAMESTATE_VERSION = "2.1.3";

export type GameState =
  | { kind: 'Active' }
  | { kind: 'Paused' }
  | { kind: 'Finished' };
```

**Effect:**
- Generates version constants for tracking schema changes
- Supports semantic versioning (MAJOR.MINOR.PATCH)
- Accepts pre-release versions (e.g., `"1.0.0-beta.1"`)
- Accepts build metadata (e.g., `"1.0.0+build.123"`)
- Useful for migration planning and compatibility checking

### Field Attributes

#### `#[key]` - Primary Key Field

```rust
#[solana]
struct UserAccount {
    #[key]
    wallet: PublicKey,
    balance: u64,
}
```

**Effect:**
- Documents primary identifier
- Used in PDA generation (future)

#### `#[max(n)]` - Maximum Length

```rust
struct Post {
    #[max(100)]
    title: string,
    #[max(5000)]
    body: string,
}
```

**Effect:**
- Adds validation in generated code
- Documents constraints

---

## Solana-Specific Features

### Solana Types

LUMOS provides native Solana types:

| LUMOS Type | Rust Type | TypeScript Type |
|------------|-----------|-----------------|
| `PublicKey` | `Pubkey` | `PublicKey` |
| `Signature` | `Signature` | `string` |
| `Keypair` | `Keypair` | N/A (server-side only) |

```rust
#[solana]
struct Transaction {
    from: PublicKey,
    to: PublicKey,
    amount: u64,
    signature: Signature,
}
```

### Borsh Serialization

All `#[solana]` types automatically get Borsh serialization:

```rust
#[solana]
struct TokenAccount {
    mint: PublicKey,
    owner: PublicKey,
    amount: u64,
}
```

**Generates:**

**Rust:**
```rust
#[derive(BorshSerialize, BorshDeserialize)]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
}
```

---

## Advanced Features

### Enums (Future)

```rust
enum Status {
    Pending,
    Active,
    Completed,
}

struct Order {
    id: u64,
    status: Status,
}
```

### Generics (Future)

```rust
struct Result<T> {
    success: bool,
    data?: T,
}
```

### Imports (Future)

```rust
import { Address } from "./common.lumos";

struct User {
    name: string,
    address: Address,
}
```

---

## Complete Examples

### Example 1: Simple User Account

```rust
#[solana]
struct UserAccount {
    #[key]
    wallet: PublicKey,
    #[max(32)]
    username: string,
    created_at: i64,
    balance: u64,
}
```

### Example 2: NFT Metadata

```rust
#[solana]
#[account]
struct NftMetadata {
    #[key]
    mint: PublicKey,
    #[max(32)]
    name: string,
    #[max(10)]
    symbol: string,
    #[max(200)]
    uri: string,
    seller_fee_basis_points: u16,
    creators: [PublicKey],
}
```

### Example 3: DEX Order

```rust
#[solana]
struct Order {
    trader: PublicKey,
    market: PublicKey,
    side: u8,              // 0 = buy, 1 = sell
    price: u64,
    size: u64,
    filled: u64,
    timestamp: i64,
}
```

### Example 4: Escrow Account

```rust
#[solana]
#[account]
struct EscrowAccount {
    initializer: PublicKey,
    initializer_token_account: PublicKey,
    initializer_amount: u64,

    taker_token_account: PublicKey,
    expected_amount: u64,

    is_initialized: bool,
}
```

### Example 5: Staking Pool

```rust
#[solana]
#[account]
struct StakingPool {
    #[key]
    authority: PublicKey,
    token_mint: PublicKey,

    total_staked: u64,
    reward_rate: u64,
    last_update_time: i64,

    stakers: [PublicKey],
}

#[solana]
struct StakerInfo {
    #[key]
    owner: PublicKey,
    amount_staked: u64,
    reward_debt: u64,
    last_claim_time: i64,
}
```

---

## Type Mapping Reference

### Complete Mapping Table

| LUMOS | Rust | TypeScript | Borsh Size | Notes |
|-------|------|------------|------------|-------|
| `u8` | `u8` | `number` | 1 byte | 0 to 255 |
| `u16` | `u16` | `number` | 2 bytes | 0 to 65,535 |
| `u32` | `u32` | `number` | 4 bytes | 0 to 4.2B |
| `u64` | `u64` | `number` | 8 bytes | 0 to 18.4 quintillion |
| `i64` | `i64` | `number` | 8 bytes | Signed equivalent |
| `bool` | `bool` | `boolean` | 1 byte | true/false |
| `string` | `String` | `string` | 4 + len | UTF-8 encoded |
| `PublicKey` | `Pubkey` | `PublicKey` | 32 bytes | Ed25519 public key |
| `[T]` | `Vec<T>` | `T[]` | 4 + (n × size) | Dynamic array |
| `T?` | `Option<T>` | `T \| undefined` | 1 + size | Optional value |

---

## Validation Rules

1. **Field names:** Must be valid Rust identifiers (snake_case recommended)
2. **Struct names:** Must be valid Rust identifiers (PascalCase recommended)
3. **String lengths:** Should specify `#[max(n)]` for Solana accounts
4. **Public keys:** Use `PublicKey` type, not raw bytes
5. **Timestamps:** Use `i64` (Unix timestamp)

---

## Error Messages

LUMOS provides clear error messages:

```
Error: Invalid type 'uint64'
  --> user.lumos:3:10
   |
 3 |     id: uint64,
   |         ^^^^^^ Did you mean 'u64'?
   |
   = note: Valid integer types: u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
```

---

## Future Syntax Features

Planned for Phase 2+:

- **Constraints:** `#[min(n)]`, `#[range(min, max)]`
- **Validation:** `#[validate(regex)]`
- **PDA Macros:** `#[pda(seeds = [...])]`
- **Instructions:** `#[instruction]` for Anchor methods
- **Events:** `#[event]` for program logs
- **Comments:** `//` single-line, `/* */` multi-line

---

## Design Principles

1. **Explicit over implicit** - Clear type mappings, no surprises
2. **Familiar syntax** - Leverage Rust and TypeScript knowledge
3. **Solana-first** - Native support for blockchain primitives
4. **Extensible** - Room to grow with Solana ecosystem
5. **Beautiful** - Code should be pleasant to read and write

---

**Version History:**
- v0.1.0 (2025-01-17) - Initial specification

**License:** MIT OR Apache-2.0

**Created by:** RECTOR at RECTOR-LABS

Bismillah! May this specification guide us to build something truly beneficial for the Solana community. 🤲
