# Migration Guide: Manual TypeScript → LUMOS

> **Transform scattered TypeScript interfaces and Borsh schemas into a single source of truth**

This guide walks you through migrating from hand-written TypeScript type definitions and Borsh serialization schemas to LUMOS-generated code. By the end, you'll have cleaner code, guaranteed type safety, and 70%+ less boilerplate.

---

## Table of Contents

1. [Introduction & Value Proposition](#introduction--value-proposition)
2. [Before/After: Visual Comparison](#beforeafter-visual-comparison)
3. [Type Mapping Reference](#type-mapping-reference)
4. [Step-by-Step Migration Process](#step-by-step-migration-process)
5. [Common Patterns Replaced](#common-patterns-replaced)
6. [Real-World Migration Examples](#real-world-migration-examples)
7. [Troubleshooting & Edge Cases](#troubleshooting--edge-cases)
8. [Migration Checklist](#migration-checklist)

---

## Introduction & Value Proposition

### The Problem with Manual TypeScript

When building Solana programs, developers typically maintain parallel code structures:

```
src/
├── types/
│   ├── player.ts      # TypeScript interfaces
│   ├── game.ts
│   └── item.ts
├── schemas/
│   ├── player.ts      # Borsh schemas (must match types!)
│   ├── game.ts
│   └── item.ts
└── index.ts           # Manual re-exports
```

**This approach has significant problems:**

| Problem | Impact |
|---------|--------|
| **Type Drift** | Interface and schema can diverge silently |
| **Double Maintenance** | Every change requires updating 2+ files |
| **No Precision Warnings** | `u64` fields silently lose precision |
| **Manual Synchronization** | Re-exports must be maintained by hand |
| **Testing Burden** | Must test that schemas match interfaces |

### The LUMOS Solution

Write your types once in a `.lumos` schema file:

```lumos
#[solana]
#[account]
struct PlayerAccount {
  wallet: PublicKey,
  username: String,
  level: u16,
  experience: u64,
  items: [PublicKey],
}
```

Generate both TypeScript interface AND Borsh schema:

```bash
lumos generate schema.lumos -o src/generated.ts
```

**Result:**
- Single source of truth
- Guaranteed Rust ↔ TypeScript compatibility
- Auto-generated precision warnings
- Zero manual synchronization

### Migration Metrics

| Metric | Before (Manual) | After (LUMOS) | Improvement |
|--------|-----------------|---------------|-------------|
| Lines of code | 100+ | 30 | **70% reduction** |
| Files to maintain | 3+ per type | 1 total | **Simplified** |
| Type safety | Manual verification | Guaranteed | **100% safe** |
| Precision warnings | None | Auto-generated | **Built-in** |
| Schema drift risk | High | Zero | **Eliminated** |

---

## Before/After: Visual Comparison

### Complete Example: Player Account

#### Before: Manual TypeScript (67 lines across 3 files)

**File 1: `src/types/player.ts`**
```typescript
import { PublicKey } from '@solana/web3.js';

/**
 * Player account interface
 * WARNING: Must match PlayerAccountSchema exactly!
 */
export interface PlayerAccount {
  /** Player's wallet address */
  wallet: PublicKey;
  /** Display name (max 20 chars) */
  username: string;
  /** Current level (1-100) */
  level: number;
  /** Total XP earned - WARNING: may exceed safe integer! */
  experience: number;
  /** Gold balance */
  gold: number;
  /** Premium currency */
  gems: number;
  /** Equipped item public keys */
  equippedItems: PublicKey[];
  /** Inventory item public keys */
  inventoryItems: PublicKey[];
  /** Achievement IDs unlocked */
  achievements: number[];
  /** Unix timestamp of account creation */
  createdAt: number;
  /** Unix timestamp of last login */
  lastLogin: number;
}
```

**File 2: `src/schemas/player.ts`**
```typescript
import * as borsh from '@coral-xyz/borsh';

/**
 * Borsh schema for PlayerAccount
 * WARNING: Field order must match interface!
 * WARNING: Field names must use snake_case for Rust compatibility!
 */
export const PlayerAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.str('username'),
  borsh.u16('level'),
  borsh.u64('experience'),
  borsh.u64('gold'),
  borsh.u32('gems'),
  borsh.vec(borsh.publicKey(), 'equipped_items'),
  borsh.vec(borsh.publicKey(), 'inventory_items'),
  borsh.vec(borsh.u32(), 'achievements'),
  borsh.i64('created_at'),
  borsh.i64('last_login'),
]);

// Manual size calculation for rent exemption
export const PLAYER_ACCOUNT_SIZE =
  32 +  // wallet
  4 + 20 +  // username (length prefix + max chars)
  2 +   // level
  8 +   // experience
  8 +   // gold
  4 +   // gems
  4 + (32 * 10) +  // equipped_items (assuming max 10)
  4 + (32 * 50) +  // inventory_items (assuming max 50)
  4 + (4 * 100) +  // achievements (assuming max 100)
  8 +   // created_at
  8;    // last_login
```

**File 3: `src/types/index.ts`**
```typescript
// Manual re-exports - must update when adding new types!
export { PlayerAccount } from './player';
export { PlayerAccountSchema, PLAYER_ACCOUNT_SIZE } from '../schemas/player';
export { GameState } from './game';
export { GameStateSchema } from '../schemas/game';
export { Item } from './item';
export { ItemSchema } from '../schemas/item';
// ... more exports as codebase grows
```

#### After: LUMOS (15 lines in 1 file)

**File: `schema.lumos`**
```lumos
#[solana]
#[account]
struct PlayerAccount {
  wallet: PublicKey,
  #[max(20)]
  username: String,
  level: u16,
  experience: u64,
  gold: u64,
  gems: u32,
  equipped_items: [PublicKey],
  inventory_items: [PublicKey],
  achievements: [u32],
  created_at: i64,
  last_login: i64,
}
```

**Generated: `src/generated.ts` (auto-created)**
```typescript
// Auto-generated by LUMOS v0.1.1
// DO NOT EDIT - Changes will be overwritten

import { PublicKey } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';

export interface PlayerAccount {
  wallet: PublicKey;
  username: string;
  level: number;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
   * For Solana lamports or large values, ensure they stay within safe range.
   * Values exceeding this limit will lose precision during serialization.
   */
  experience: number;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
   * For Solana lamports or large values, ensure they stay within safe range.
   * Values exceeding this limit will lose precision during serialization.
   */
  gold: number;
  gems: number;
  equipped_items: PublicKey[];
  inventory_items: PublicKey[];
  achievements: number[];
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
   * For Solana lamports or large values, ensure they stay within safe range.
   * Values exceeding this limit will lose precision during serialization.
   */
  created_at: number;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
   * For Solana lamports or large values, ensure they stay within safe range.
   * Values exceeding this limit will lose precision during serialization.
   */
  last_login: number;
}

export const PlayerAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.str('username'),
  borsh.u16('level'),
  borsh.u64('experience'),
  borsh.u64('gold'),
  borsh.u32('gems'),
  borsh.vec(borsh.publicKey(), 'equipped_items'),
  borsh.vec(borsh.publicKey(), 'inventory_items'),
  borsh.vec(borsh.u32(), 'achievements'),
  borsh.i64('created_at'),
  borsh.i64('last_login'),
]);
```

#### Comparison Summary

| Aspect | Manual | LUMOS |
|--------|--------|-------|
| **Source files** | 3 | 1 |
| **Lines written** | 67 | 15 |
| **Precision warnings** | Manual comments | Auto-generated JSDoc |
| **Schema sync** | Manual verification | Guaranteed |
| **Field naming** | Must remember snake_case | Automatic |
| **Re-exports** | Manual | Not needed |

---

## Type Mapping Reference

### Primitive Types

| LUMOS Type | TypeScript Type | Borsh Schema | Range/Notes |
|-----------|-----------------|--------------|-------------|
| `u8` | `number` | `borsh.u8()` | 0 to 255 |
| `u16` | `number` | `borsh.u16()` | 0 to 65,535 |
| `u32` | `number` | `borsh.u32()` | 0 to 4,294,967,295 |
| `u64` | `number` | `borsh.u64()` | ⚠️ 0 to 2^64-1 (precision warning) |
| `u128` | `bigint` | `borsh.u128()` | 0 to 2^128-1 (native BigInt) |
| `i8` | `number` | `borsh.i8()` | -128 to 127 |
| `i16` | `number` | `borsh.i16()` | -32,768 to 32,767 |
| `i32` | `number` | `borsh.i32()` | -2^31 to 2^31-1 |
| `i64` | `number` | `borsh.i64()` | ⚠️ Precision warning added |
| `i128` | `bigint` | `borsh.i128()` | Native BigInt |
| `f32` | `number` | `borsh.f32()` | IEEE 754 single precision |
| `f64` | `number` | `borsh.f64()` | IEEE 754 double precision |
| `bool` | `boolean` | `borsh.bool()` | true/false |
| `String` | `string` | `borsh.str()` | UTF-8 encoded |

### Solana Types

| LUMOS Type | TypeScript Type | Borsh Schema | Import |
|-----------|-----------------|--------------|--------|
| `PublicKey` | `PublicKey` | `borsh.publicKey()` | `@solana/web3.js` |
| `Pubkey` | `PublicKey` | `borsh.publicKey()` | `@solana/web3.js` |
| `Signature` | `Uint8Array` | `borsh.fixedArray(borsh.u8(), 64)` | N/A |

### Complex Types

| LUMOS Type | TypeScript Type | Borsh Schema | Example |
|-----------|-----------------|--------------|---------|
| `[T]` | `T[]` | `borsh.vec(T_schema)` | `[PublicKey]` → `PublicKey[]` |
| `[T; N]` | `T[]` | `borsh.array(T_schema, N)` | `[u8; 32]` → `number[]` |
| `Option<T>` | `T \| null` | `borsh.option(T_schema)` | `Option<String>` → `string \| null` |

### Type Conversion Examples

```lumos
// LUMOS Schema
struct Example {
  // Primitives
  small_number: u8,           // → number
  medium_number: u32,         // → number
  large_number: u64,          // → number (with warning)
  huge_number: u128,          // → bigint

  // Solana
  owner: PublicKey,           // → PublicKey

  // Complex
  items: [u32],               // → number[]
  fixed_data: [u8; 32],       // → number[] (length 32)
  maybe_value: Option<u64>,   // → number | null

  // Nested
  accounts: [PublicKey],      // → PublicKey[]
  optional_list: Option<[u8]>,// → number[] | null
}
```

### Precision Warning Details

LUMOS automatically adds JSDoc warnings for `u64` and `i64` fields:

```typescript
/**
 * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
 * For Solana lamports or large values, ensure they stay within safe range.
 * Values exceeding this limit will lose precision during serialization.
 */
balance: number;
```

**Why this matters:**
- JavaScript `number` is IEEE 754 double-precision (53-bit mantissa)
- Maximum safe integer: `Number.MAX_SAFE_INTEGER` = 9,007,199,254,740,991
- Solana lamports can exceed this (1 SOL = 1,000,000,000 lamports)
- Values above 2^53-1 silently lose precision

**Solutions:**
1. Use `u128` for very large values (maps to `bigint`)
2. Keep values within safe range
3. Use string representation for display

---

## Step-by-Step Migration Process

### Overview

```
┌─────────────────┐
│ 1. Audit        │ Find all manual TypeScript types and schemas
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 2. Analyze      │ Map types to LUMOS equivalents
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 3. Create       │ Write schema.lumos file
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 4. Validate     │ Run lumos validate
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 5. Generate     │ Create TypeScript output
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 6. Replace      │ Update imports in codebase
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 7. Test         │ Verify serialization works
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 8. Cleanup      │ Remove manual files
└─────────────────┘
```

### Step 1: Audit Your Codebase

Find all manual TypeScript interfaces and Borsh schemas:

```bash
# Find TypeScript interfaces
grep -r "export interface" --include="*.ts" src/

# Find Borsh schema definitions
grep -r "borsh.struct\|borsh.rustEnum" --include="*.ts" src/

# Find manual schema objects
grep -r "kind: 'struct'\|kind: 'enum'" --include="*.ts" src/
```

**Create an inventory:**

```markdown
## Types to Migrate

| Interface | Schema File | Fields | Priority |
|-----------|-------------|--------|----------|
| PlayerAccount | schemas/player.ts | 12 | High |
| GameState | schemas/game.ts | 5 | High |
| Item | schemas/item.ts | 8 | Medium |
| Config | schemas/config.ts | 4 | Low |
```

### Step 2: Analyze Type Mappings

For each interface, map fields to LUMOS types:

```typescript
// Original TypeScript
interface PlayerAccount {
  wallet: PublicKey;        // → PublicKey
  username: string;         // → String (add #[max(N)] if needed)
  level: number;            // → u16 (check actual range)
  experience: number;       // → u64 (check if bigint needed)
  isActive: boolean;        // → bool
  items: PublicKey[];       // → [PublicKey]
  metadata?: string;        // → Option<String>
}
```

**Key decisions:**
- `number` → Choose specific size (`u8`, `u16`, `u32`, `u64`)
- `bigint` → Use `u128` or `i128`
- Optional fields (`?`) → Use `Option<T>`
- Arrays → Use `[T]` for dynamic, `[T; N]` for fixed

### Step 3: Create schema.lumos

Create a new file at project root or `schemas/` directory:

```lumos
// schema.lumos - Single source of truth for all types

#[solana]
#[account]
struct PlayerAccount {
  wallet: PublicKey,
  #[max(20)]
  username: String,
  level: u16,
  experience: u64,
  is_active: bool,
  items: [PublicKey],
  metadata: Option<String>,
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
struct Item {
  id: u32,
  #[max(50)]
  name: String,
  item_type: u8,
  rarity: u8,
  power: u16,
  owner: PublicKey,
  created_at: i64,
  is_tradeable: bool,
}
```

**Naming conventions:**
- Use `snake_case` for field names (Rust convention)
- LUMOS handles camelCase conversion for TypeScript if configured
- Keep names consistent with your Rust program

### Step 4: Validate Schema

Run validation to catch errors early:

```bash
lumos validate schema.lumos
```

**Expected output:**
```
✓ Parsed 3 types
✓ All type references valid
✓ No circular dependencies
✓ Schema is valid
```

**Common validation errors:**

```bash
# Undefined type reference
Error: Unknown type 'Player' referenced in field 'GameState.winner'
  --> schema.lumos:15:20
   |
15 |   Finished { winner: Player },
   |                      ^^^^^^ undefined type

# Fix: Define Player or use PublicKey
```

### Step 5: Generate TypeScript

Generate the TypeScript output:

```bash
# Basic generation
lumos generate schema.lumos -o src/generated.ts

# With specific options
lumos generate schema.lumos \
  --lang typescript \
  --output src/types/generated.ts
```

**Verify the output:**

```bash
# Check generated file
cat src/generated.ts

# Verify TypeScript compiles
npx tsc --noEmit src/generated.ts
```

### Step 6: Replace Imports

Update your codebase to use generated types:

**Before:**
```typescript
// src/client/player.ts
import { PlayerAccount } from '../types/player';
import { PlayerAccountSchema } from '../schemas/player';
import { GameState } from '../types/game';
import { GameStateSchema } from '../schemas/game';
```

**After:**
```typescript
// src/client/player.ts
import {
  PlayerAccount,
  PlayerAccountSchema,
  GameState,
  GameStateSchema
} from '../generated';
```

**Find and replace across codebase:**

```bash
# Find all imports from old type files
grep -r "from.*types/player" --include="*.ts" src/
grep -r "from.*schemas/player" --include="*.ts" src/

# Use IDE find-and-replace or sed
sed -i "s|from '../types/player'|from '../generated'|g" src/**/*.ts
sed -i "s|from '../schemas/player'|from '../generated'|g" src/**/*.ts
```

### Step 7: Test Serialization

Verify that generated schemas work with existing data:

```typescript
// test/migration.test.ts
import { PlayerAccount, PlayerAccountSchema } from '../src/generated';
import * as borsh from '@coral-xyz/borsh';

describe('Migration Validation', () => {
  it('should deserialize existing account data', async () => {
    // Fetch existing account from devnet
    const connection = new Connection('https://api.devnet.solana.com');
    const accountInfo = await connection.getAccountInfo(PLAYER_ACCOUNT_PUBKEY);

    // Deserialize with new schema
    const player = borsh.deserialize(
      PlayerAccountSchema,
      accountInfo.data
    ) as PlayerAccount;

    // Verify fields
    expect(player.wallet).toBeDefined();
    expect(typeof player.level).toBe('number');
    expect(Array.isArray(player.items)).toBe(true);
  });

  it('should serialize new data correctly', () => {
    const player: PlayerAccount = {
      wallet: Keypair.generate().publicKey,
      username: 'TestPlayer',
      level: 1,
      experience: 0,
      is_active: true,
      items: [],
      metadata: null,
    };

    // Serialize
    const buffer = Buffer.alloc(1000);
    borsh.serialize(PlayerAccountSchema, player, buffer);

    // Deserialize and compare
    const decoded = borsh.deserialize(
      PlayerAccountSchema,
      buffer
    ) as PlayerAccount;

    expect(decoded.username).toBe(player.username);
    expect(decoded.level).toBe(player.level);
  });
});
```

### Step 8: Cleanup

Remove old manual files:

```bash
# Remove old type files
rm src/types/player.ts
rm src/types/game.ts
rm src/types/item.ts

# Remove old schema files
rm src/schemas/player.ts
rm src/schemas/game.ts
rm src/schemas/item.ts

# Remove index re-exports (if now empty)
rm src/types/index.ts
rm src/schemas/index.ts
```

**Update .gitignore:**

```gitignore
# Generated files (optional - some prefer to commit)
src/generated.ts
```

**Add generation to build process:**

```json
// package.json
{
  "scripts": {
    "generate": "lumos generate schema.lumos -o src/generated.ts",
    "prebuild": "npm run generate",
    "build": "tsc"
  }
}
```

---

## Common Patterns Replaced

### Pattern 1: Manual Enum Handling

#### Before: Numeric Enums (No Type Safety)

```typescript
// types/game.ts
export enum GameState {
  Waiting = 0,
  Active = 1,
  Paused = 2,
  Finished = 3,
}

// schemas/game.ts
export const GameStateSchema = borsh.u8(); // Just a number!

// Usage - no type safety
function handleState(state: GameState) {
  if (state === GameState.Finished) {
    // How do we access the winner? We can't!
    // The enum is just a number, no associated data
  }
}
```

#### After: Discriminated Unions (Full Type Safety)

```lumos
#[solana]
enum GameState {
  Waiting,
  Active,
  Paused,
  Finished { winner: PublicKey },
}
```

**Generated TypeScript:**

```typescript
export type GameState =
  | { kind: 'Waiting' }
  | { kind: 'Active' }
  | { kind: 'Paused' }
  | { kind: 'Finished'; winner: PublicKey };

export const GameStateSchema = borsh.rustEnum([
  borsh.struct([], 'Waiting'),
  borsh.struct([], 'Active'),
  borsh.struct([], 'Paused'),
  borsh.struct([borsh.publicKey('winner')], 'Finished'),
]);

// Usage - full type safety!
function handleState(state: GameState) {
  switch (state.kind) {
    case 'Waiting':
      console.log('Waiting for players...');
      break;
    case 'Active':
      console.log('Game in progress');
      break;
    case 'Paused':
      console.log('Game paused');
      break;
    case 'Finished':
      // TypeScript knows 'winner' exists here!
      console.log(`Winner: ${state.winner.toBase58()}`);
      break;
  }
}
```

### Pattern 2: Scattered Type Definitions

#### Before: Types Spread Across Multiple Files

```
src/
├── types/
│   ├── player.ts        # PlayerAccount interface
│   ├── game.ts          # GameState, GameConfig interfaces
│   ├── item.ts          # Item, ItemType interfaces
│   ├── marketplace.ts   # Listing, Offer interfaces
│   └── index.ts         # Manual re-exports (50+ lines)
├── schemas/
│   ├── player.ts        # PlayerAccount schema
│   ├── game.ts          # GameState, GameConfig schemas
│   ├── item.ts          # Item, ItemType schemas
│   ├── marketplace.ts   # Listing, Offer schemas
│   └── index.ts         # Manual re-exports (50+ lines)
└── utils/
    └── sizes.ts         # Manual size calculations (100+ lines)
```

**Problems:**
- 10+ files to maintain
- Manual re-exports must be updated
- Size calculations are error-prone
- Easy for types and schemas to drift

#### After: Single Source of Truth

```
src/
├── schema.lumos         # All type definitions (100 lines)
├── generated.ts         # Auto-generated (500 lines)
└── client/
    └── ...              # Your application code
```

**Benefits:**
- 1 file to edit
- 1 file generated
- Zero manual re-exports
- Guaranteed consistency

### Pattern 3: Missing Precision Warnings

#### Before: Silent Precision Loss

```typescript
// types/account.ts
interface StakingAccount {
  totalStaked: number;    // u64 in Rust - but no warning!
  rewardDebt: number;     // u64 in Rust - but no warning!
  lastUpdate: number;     // i64 timestamp - but no warning!
}

// Usage - silent data corruption possible
const account = deserialize(data);
const newTotal = account.totalStaked + 1000000000000000;
// If totalStaked > 2^53, this calculation is WRONG!
```

#### After: Auto-Generated Warnings

```lumos
#[solana]
#[account]
struct StakingAccount {
  total_staked: u64,
  reward_debt: u64,
  last_update: i64,
}
```

**Generated TypeScript:**

```typescript
export interface StakingAccount {
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
   * For Solana lamports or large values, ensure they stay within safe range.
   * Values exceeding this limit will lose precision during serialization.
   */
  total_staked: number;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
   * For Solana lamports or large values, ensure they stay within safe range.
   * Values exceeding this limit will lose precision during serialization.
   */
  reward_debt: number;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1 (9,007,199,254,740,991).
   * For Solana lamports or large values, ensure they stay within safe range.
   * Values exceeding this limit will lose precision during serialization.
   */
  last_update: number;
}
```

### Pattern 4: Manual Array and Option Handling

#### Before: Verbose Wrapper Types

```typescript
// types/inventory.ts
interface Inventory {
  // Manual optional handling
  equippedWeapon: PublicKey | null | undefined;
  equippedArmor: PublicKey | null | undefined;

  // Confusing array types
  items: Array<PublicKey>;

  // Fixed-size arrays are tricky
  quickSlots: [PublicKey, PublicKey, PublicKey, PublicKey]; // Tuple, not array!
}

// schemas/inventory.ts
const InventorySchema = borsh.struct([
  borsh.option(borsh.publicKey(), 'equipped_weapon'),
  borsh.option(borsh.publicKey(), 'equipped_armor'),
  borsh.vec(borsh.publicKey(), 'items'),
  // Fixed array requires manual handling
  borsh.array(borsh.publicKey(), 4, 'quick_slots'),
]);
```

#### After: Clean LUMOS Syntax

```lumos
#[solana]
#[account]
struct Inventory {
  equipped_weapon: Option<PublicKey>,
  equipped_armor: Option<PublicKey>,
  items: [PublicKey],
  quick_slots: [PublicKey; 4],
}
```

**Generated TypeScript:**

```typescript
export interface Inventory {
  equipped_weapon: PublicKey | null;
  equipped_armor: PublicKey | null;
  items: PublicKey[];
  quick_slots: PublicKey[];  // Length 4, enforced by schema
}

export const InventorySchema = borsh.struct([
  borsh.option(borsh.publicKey(), 'equipped_weapon'),
  borsh.option(borsh.publicKey(), 'equipped_armor'),
  borsh.vec(borsh.publicKey(), 'items'),
  borsh.array(borsh.publicKey(), 4, 'quick_slots'),
]);
```

### Pattern 5: Duplicate Constant Definitions

#### Before: Constants in Multiple Places

```typescript
// constants/sizes.ts
export const MAX_USERNAME_LENGTH = 20;
export const MAX_ITEMS = 50;
export const MAX_ACHIEVEMENTS = 100;

// types/player.ts
interface Player {
  username: string;  // Max 20 chars - hope this matches!
  items: PublicKey[];  // Max 50 - hope this matches!
}

// validation/player.ts
function validatePlayer(player: Player) {
  if (player.username.length > MAX_USERNAME_LENGTH) {
    throw new Error('Username too long');
  }
  if (player.items.length > MAX_ITEMS) {
    throw new Error('Too many items');
  }
}
```

#### After: Constraints in Schema

```lumos
#[solana]
#[account]
struct Player {
  #[max(20)]
  username: String,
  #[max(50)]
  items: [PublicKey],
  #[max(100)]
  achievements: [u32],
}
```

The constraints are:
1. Documented in the schema
2. Used for space calculation
3. Single source of truth

---

## Real-World Migration Examples

### Example 1: Gaming - Player Account

#### Original Manual Code (89 lines)

```typescript
// src/types/player.ts (35 lines)
import { PublicKey } from '@solana/web3.js';

export interface PlayerAccount {
  wallet: PublicKey;
  username: string;
  level: number;
  experience: number;
  health: number;
  mana: number;
  gold: number;
  gems: number;
  equippedItems: PublicKey[];
  inventoryItems: PublicKey[];
  questsCompleted: number;
  achievements: number[];
  createdAt: number;
  lastLogin: number;
  totalPlaytime: number;
}

// src/schemas/player.ts (40 lines)
import * as borsh from '@coral-xyz/borsh';

export const PlayerAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.str('username'),
  borsh.u16('level'),
  borsh.u64('experience'),
  borsh.u16('health'),
  borsh.u16('mana'),
  borsh.u64('gold'),
  borsh.u32('gems'),
  borsh.vec(borsh.publicKey(), 'equipped_items'),
  borsh.vec(borsh.publicKey(), 'inventory_items'),
  borsh.u32('quests_completed'),
  borsh.vec(borsh.u32(), 'achievements'),
  borsh.i64('created_at'),
  borsh.i64('last_login'),
  borsh.i64('total_playtime'),
]);

// Manual size calculation
export const PLAYER_ACCOUNT_SIZE = 32 + 24 + 2 + 8 + 2 + 2 + 8 + 4 +
  4 + (32 * 10) + 4 + (32 * 50) + 4 + 4 + (4 * 100) + 8 + 8 + 8;

// src/types/index.ts (14 lines)
export { PlayerAccount } from './player';
export { PlayerAccountSchema, PLAYER_ACCOUNT_SIZE } from '../schemas/player';
```

#### Migrated LUMOS Code (18 lines)

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
  health: u16,
  mana: u16,
  gold: u64,
  gems: u32,
  equipped_items: [PublicKey],
  inventory_items: [PublicKey],
  quests_completed: u32,
  achievements: [u32],
  created_at: i64,
  last_login: i64,
  total_playtime: i64,
}
```

**Reduction: 89 lines → 18 lines (80%)**

### Example 2: DeFi - Staking Pool

#### Original Manual Code (112 lines)

```typescript
// src/types/staking.ts (45 lines)
import { PublicKey } from '@solana/web3.js';

export interface StakingPool {
  authority: PublicKey;
  tokenMint: PublicKey;
  vault: PublicKey;
  totalStaked: number;
  totalStakers: number;
  rewardRatePerSecond: number;
  rewardTokenMint: PublicKey;
  rewardVault: PublicKey;
  startTime: number;
  endTime: number | null;
  lastUpdateTime: number;
  isActive: boolean;
  minStakeAmount: number;
  maxStakeAmount: number | null;
}

export interface Staker {
  owner: PublicKey;
  pool: PublicKey;
  stakedAmount: number;
  rewardDebt: number;
  lastStakeTime: number;
  lockEndTime: number | null;
}

// src/schemas/staking.ts (55 lines)
import * as borsh from '@coral-xyz/borsh';

export const StakingPoolSchema = borsh.struct([
  borsh.publicKey('authority'),
  borsh.publicKey('token_mint'),
  borsh.publicKey('vault'),
  borsh.u64('total_staked'),
  borsh.u32('total_stakers'),
  borsh.u64('reward_rate_per_second'),
  borsh.publicKey('reward_token_mint'),
  borsh.publicKey('reward_vault'),
  borsh.i64('start_time'),
  borsh.option(borsh.i64(), 'end_time'),
  borsh.i64('last_update_time'),
  borsh.bool('is_active'),
  borsh.u64('min_stake_amount'),
  borsh.option(borsh.u64(), 'max_stake_amount'),
]);

export const StakerSchema = borsh.struct([
  borsh.publicKey('owner'),
  borsh.publicKey('pool'),
  borsh.u64('staked_amount'),
  borsh.u64('reward_debt'),
  borsh.i64('last_stake_time'),
  borsh.option(borsh.i64(), 'lock_end_time'),
]);

// src/types/index.ts (12 lines)
export { StakingPool, Staker } from './staking';
export { StakingPoolSchema, StakerSchema } from '../schemas/staking';
```

#### Migrated LUMOS Code (28 lines)

```lumos
#[solana]
#[account]
struct StakingPool {
  #[key]
  authority: PublicKey,
  token_mint: PublicKey,
  vault: PublicKey,
  total_staked: u64,
  total_stakers: u32,
  reward_rate_per_second: u64,
  reward_token_mint: PublicKey,
  reward_vault: PublicKey,
  start_time: i64,
  end_time: Option<i64>,
  last_update_time: i64,
  is_active: bool,
  min_stake_amount: u64,
  max_stake_amount: Option<u64>,
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
  lock_end_time: Option<i64>,
}
```

**Reduction: 112 lines → 28 lines (75%)**

### Example 3: NFT Marketplace - Listing

#### Original Manual Code (78 lines)

```typescript
// src/types/marketplace.ts (32 lines)
import { PublicKey } from '@solana/web3.js';

export interface Listing {
  seller: PublicKey;
  nftMint: PublicKey;
  price: number;
  tokenMint: PublicKey;
  createdAt: number;
  expiresAt: number | null;
  isActive: boolean;
}

export enum ListingState {
  Active = 0,
  Sold = 1,
  Cancelled = 2,
  Expired = 3,
}

export interface Offer {
  buyer: PublicKey;
  listing: PublicKey;
  amount: number;
  expiresAt: number;
  state: OfferState;
}

export enum OfferState {
  Pending = 0,
  Accepted = 1,
  Rejected = 2,
  Expired = 3,
}

// src/schemas/marketplace.ts (38 lines)
import * as borsh from '@coral-xyz/borsh';

export const ListingSchema = borsh.struct([
  borsh.publicKey('seller'),
  borsh.publicKey('nft_mint'),
  borsh.u64('price'),
  borsh.publicKey('token_mint'),
  borsh.i64('created_at'),
  borsh.option(borsh.i64(), 'expires_at'),
  borsh.bool('is_active'),
]);

export const ListingStateSchema = borsh.u8();

export const OfferSchema = borsh.struct([
  borsh.publicKey('buyer'),
  borsh.publicKey('listing'),
  borsh.u64('amount'),
  borsh.i64('expires_at'),
  borsh.u8('state'),
]);

export const OfferStateSchema = borsh.u8();

// src/types/index.ts (8 lines)
export * from './marketplace';
export * from '../schemas/marketplace';
```

#### Migrated LUMOS Code (32 lines)

```lumos
#[solana]
#[account]
struct Listing {
  #[key]
  seller: PublicKey,
  nft_mint: PublicKey,
  price: u64,
  token_mint: PublicKey,
  created_at: i64,
  expires_at: Option<i64>,
  is_active: bool,
}

#[solana]
enum ListingState {
  Active,
  Sold,
  Cancelled,
  Expired,
}

#[solana]
#[account]
struct Offer {
  #[key]
  buyer: PublicKey,
  listing: PublicKey,
  amount: u64,
  expires_at: i64,
  state: OfferState,
}

#[solana]
enum OfferState {
  Pending,
  Accepted,
  Rejected,
  Expired,
}
```

**Reduction: 78 lines → 32 lines (59%)**

---

## Troubleshooting & Edge Cases

### Issue 1: u64 Values Exceed Safe Range

**Problem:**
```typescript
// Runtime error or silent precision loss
const balance = account.balance; // 10000000000000000000 (10^19)
const adjusted = balance + 1;    // Wrong result!
```

**Solutions:**

1. **Use u128 for very large values:**
```lumos
struct Account {
  balance: u128,  // Maps to TypeScript bigint
}
```

2. **Keep values in safe range:**
```typescript
// Check before arithmetic
if (balance > Number.MAX_SAFE_INTEGER) {
  throw new Error('Balance exceeds safe integer range');
}
```

3. **Use string for display:**
```typescript
const balanceDisplay = balance.toLocaleString();
```

### Issue 2: Existing On-Chain Data Compatibility

**Problem:** Will generated schemas deserialize existing accounts?

**Solution:** Yes! LUMOS generates identical Borsh layouts. Verify with:

```bash
# Compare schemas
lumos check schema.lumos --compare src/schemas/

# Test deserialization
lumos test schema.lumos --account <PUBKEY> --network devnet
```

**Manual verification:**

```typescript
// Fetch and deserialize existing account
const accountInfo = await connection.getAccountInfo(existingAccount);
const decoded = borsh.deserialize(GeneratedSchema, accountInfo.data);
console.log('Decoded successfully:', decoded);
```

### Issue 3: Field Naming Conventions

**Problem:** TypeScript uses camelCase, Rust uses snake_case.

**Solution:** LUMOS uses snake_case in schemas (Rust convention). The generated TypeScript maintains snake_case for Borsh compatibility:

```lumos
struct Player {
  total_score: u64,    // snake_case in schema
}
```

```typescript
// Generated TypeScript - keeps snake_case for serialization
export interface Player {
  total_score: number;  // Matches Rust field name
}
```

**If you need camelCase in your app:**

```typescript
// Create a mapping function
function toAppFormat(player: Player): AppPlayer {
  return {
    totalScore: player.total_score,
    // ... other fields
  };
}
```

### Issue 4: Circular Type References

**Problem:**
```lumos
// This won't work - circular reference
struct Node {
  value: u32,
  next: Node,  // Error: infinite size
}
```

**Solution:** Use PublicKey references:

```lumos
struct Node {
  value: u32,
  next: Option<PublicKey>,  // Reference to another Node account
}
```

### Issue 5: Generic Types

**Problem:** LUMOS doesn't support generic types directly.

**Solution:** Create concrete types:

```lumos
// Instead of Result<T, E>, create specific types
#[solana]
enum TransferResult {
  Success { amount: u64 },
  Error { code: u32, message: String },
}

// Instead of Vec<T> with different T, create specific arrays
struct Inventory {
  weapons: [PublicKey],
  armor: [PublicKey],
  consumables: [PublicKey],
}
```

### Issue 6: Migration During Active Development

**Problem:** Team is actively changing types during migration.

**Solution:** Incremental migration:

1. **Phase 1:** Add LUMOS alongside existing types
   ```
   src/
   ├── types/           # Keep existing
   ├── schemas/         # Keep existing
   └── schema.lumos     # Add new
   ```

2. **Phase 2:** Generate and compare
   ```bash
   lumos generate schema.lumos -o src/generated-new.ts
   diff src/types/index.ts src/generated-new.ts
   ```

3. **Phase 3:** Switch imports file by file
   ```typescript
   // Switch one file at a time
   import { Player } from '../generated';  // New
   import { Game } from '../types';         // Old (for now)
   ```

4. **Phase 4:** Remove old files after full migration

---

## Migration Checklist

Use this checklist to track your migration progress:

### Pre-Migration

- [ ] Audit all TypeScript interface files
- [ ] Audit all Borsh schema files
- [ ] Document all types to migrate
- [ ] Identify field types and constraints
- [ ] Check for circular dependencies
- [ ] Backup existing code (git branch)

### Schema Creation

- [ ] Create `schema.lumos` file
- [ ] Define all struct types
- [ ] Define all enum types
- [ ] Add `#[solana]` attributes
- [ ] Add `#[account]` for on-chain accounts
- [ ] Add `#[max(N)]` constraints for strings/arrays
- [ ] Add `#[key]` for primary key fields

### Validation

- [ ] Run `lumos validate schema.lumos`
- [ ] Fix any validation errors
- [ ] Verify type mappings are correct

### Generation

- [ ] Run `lumos generate schema.lumos -o src/generated.ts`
- [ ] Verify TypeScript compiles: `npx tsc --noEmit`
- [ ] Review generated interfaces
- [ ] Review generated schemas
- [ ] Check precision warnings are present

### Integration

- [ ] Update imports in all consuming files
- [ ] Remove old type imports
- [ ] Remove old schema imports
- [ ] Run existing tests
- [ ] Fix any type errors

### Testing

- [ ] Test deserialization of existing accounts
- [ ] Test serialization of new data
- [ ] Verify round-trip (serialize → deserialize)
- [ ] Run full test suite
- [ ] Test on devnet with real accounts

### Cleanup

- [ ] Remove old `types/` directory
- [ ] Remove old `schemas/` directory
- [ ] Remove manual re-export files
- [ ] Update `.gitignore` if needed
- [ ] Add `lumos generate` to build scripts

### Documentation

- [ ] Update README with LUMOS usage
- [ ] Document schema file location
- [ ] Add generation instructions
- [ ] Update contribution guidelines

---

## Next Steps

After completing your migration:

1. **Add to CI/CD:** Ensure schemas are validated on every PR
   ```yaml
   - run: lumos validate schema.lumos
   - run: lumos generate schema.lumos -o src/generated.ts
   - run: git diff --exit-code src/generated.ts
   ```

2. **Explore Advanced Features:**
   - [LUMOS + Anchor Integration](/guides/anchor-integration) - Generate complete Anchor programs
   - [LUMOS + web3.js Integration](/guides/web3js-integration) - Frontend development patterns
   - [LUMOS + Solana CLI Integration](/guides/solana-cli-integration) - Deployment workflows

3. **Use Case Guides:**
   - [Gaming Projects](/guides/use-cases/gaming) - Full gaming implementation
   - [NFT Marketplaces](/guides/use-cases/nft) - Metaplex-compatible NFTs
   - [DeFi Protocols](/guides/use-cases/defi) - Staking and vesting

---

## Summary

Migrating from manual TypeScript to LUMOS provides:

| Benefit | Impact |
|---------|--------|
| **70% less code** | Faster development, fewer bugs |
| **Single source of truth** | No more type drift |
| **Auto-generated warnings** | Catch precision issues early |
| **Type-safe enums** | Full discriminated union support |
| **Guaranteed compatibility** | Rust ↔ TypeScript always in sync |

**Start your migration today:**

```bash
# Install LUMOS
cargo install lumos-cli

# Create your schema
touch schema.lumos

# Generate TypeScript
lumos generate schema.lumos -o src/generated.ts
```

---

**Related Guides:**
- [LUMOS + web3.js Integration](/guides/web3js-integration)
- [LUMOS + Solana CLI Integration](/guides/solana-cli-integration)
- [Gaming Use Case](/guides/use-cases/gaming)
- [NFT Use Case](/guides/use-cases/nft)
- [DeFi Use Case](/guides/use-cases/defi)
