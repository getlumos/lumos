# LUMOS Extraction Tool - Real-World Examples

**Companion to:** extraction-tool-design.md
**Purpose:** Practical examples showing extraction tool in action

---

## Real-World Example: Migrating Anchor Gaming Project

### Starting Point: Existing Solana Game

**Project structure:**
```
solana-rpg/
├── programs/
│   └── rpg/
│       └── src/
│           ├── lib.rs
│           └── state.rs  (300 lines, 8 structs)
└── app/
    └── src/
        └── types.ts  (200 lines, manual types)
```

**Current pain:** Type drift between Rust and TypeScript

---

### Step 1: Analyze Existing Code

**Command:**
```bash
cd solana-rpg
lumos extract programs/rpg/src/state.rs --dry-run
```

**Output:**
```
🔍 Analyzing programs/rpg/src/state.rs...

File: programs/rpg/src/state.rs (300 lines)

Found types:

  ✓ EXTRACTABLE (8 types):
    1. PlayerAccount (#[account]) - 12 fields
    2. CharacterStats (#[account]) - 8 fields
    3. Inventory (#[account]) - 5 fields
    4. Guild (#[account]) - 10 fields
    5. Quest (BorshSerialize) - 6 fields
    6. Item (BorshSerialize) - 8 fields
    7. GameEvent (enum, 5 variants)
    8. CharacterClass (enum, 4 variants)

  ⊘ SKIPPED (2 types):
    - InternalCache (no derives)
    - TempData (lifetime parameter - unsupported)

  ⚠️ WARNINGS:
    - PlayerAccount::equipped_items uses [Item; 10] (fixed array)
      → Will convert to Vec<Item> (verify compatibility)

Summary:
  Total types: 10
  Extractable: 8
  Would generate: ~250 lines of LUMOS

Estimated LUMOS schema size: Similar to current Rust (simplified syntax)

Run without --dry-run to perform extraction:
  lumos extract programs/rpg/src/state.rs --output lumos/schema.lumos
```

---

### Step 2: Extract with Interactive Mode

**Command:**
```bash
lumos extract programs/rpg/src/state.rs --interactive
```

**Interactive session:**

```
🔍 Analyzing programs/rpg/src/state.rs...

Found 8 extractable types.

──────────────────────────────────────────────
 EXTRACTION CONFIGURATION
──────────────────────────────────────────────

Output file: [schema.lumos]
> lumos/game-types.lumos

Preserve Rust doc comments? [Y/n]
> y

Add extraction metadata (source files, timestamps)? [y/N]
> y

Group types by: [file/module/type]
> type

──────────────────────────────────────────────
 TYPE SELECTION
──────────────────────────────────────────────

Extract all 8 compatible types? [Y/n]
> n

  1. [✓] PlayerAccount (#[account])
  2. [✓] CharacterStats (#[account])
  3. [✓] Inventory (#[account])
  4. [✓] Guild (#[account])
  5. [✓] Quest (BorshSerialize)
  6. [✓] Item (BorshSerialize)
  7. [✓] GameEvent (enum)
  8. [✓] CharacterClass (enum)

Toggle selection (1-8), or [a]ll, [n]one, [c]ontinue:
> c

──────────────────────────────────────────────
 WARNINGS
──────────────────────────────────────────────

⚠️  PlayerAccount::equipped_items
  Current type: [Item; 10] (fixed-size array)
  LUMOS equivalent: Vec<Item> (dynamic array)

  This changes behavior! Fixed arrays → dynamic arrays.

  Options:
    [1] Convert to Vec<Item> (recommended)
    [2] Skip this field
    [3] Add manual TODO comment
    [4] Abort extraction

> 1

✓ Will convert to Vec<Item>

──────────────────────────────────────────────
 EXTRACTING
──────────────────────────────────────────────

  [1/8] PlayerAccount ✓
  [2/8] CharacterStats ✓
  [3/8] Inventory ✓
  [4/8] Guild ✓
  [5/8] Quest ✓
  [6/8] Item ✓
  [7/8] GameEvent ✓
  [8/8] CharacterClass ✓

✓ Extraction complete!

──────────────────────────────────────────────
 SUMMARY
──────────────────────────────────────────────

Output file: lumos/game-types.lumos
Types extracted: 8
Lines generated: 245
Warnings resolved: 1
Skipped types: 2 (InternalCache, TempData)

Next steps:
  1. Review: cat lumos/game-types.lumos
  2. Validate: lumos validate lumos/game-types.lumos
  3. Generate code: lumos generate lumos/game-types.lumos
  4. Test: cargo test && npm test

Happy coding! ✨
```

---

### Step 3: Review Generated Schema

**File:** `lumos/game-types.lumos`

```rust
// ═══════════════════════════════════════════════════════════
// LUMOS Schema for Solana RPG
// ═══════════════════════════════════════════════════════════
// Extracted from: programs/rpg/src/state.rs
// Generated: 2025-01-18 10:30:00 UTC
// LUMOS version: 0.2.0
// Types extracted: 8
// ═══════════════════════════════════════════════════════════

// ───────────────────────────────────────────────────────────
// Account Types
// ───────────────────────────────────────────────────────────

// Player account storing all character data
// Updated on every game action
// Original: programs/rpg/src/state.rs:15
#[solana]
#[account]
struct PlayerAccount {
    // Player's wallet address (owner)
    owner: PublicKey,

    // Character name (max 32 chars)
    name: String,

    // Current character level (1-100)
    level: u16,

    // Total experience points
    experience: u64,

    // Current health points
    health: u32,

    // Maximum health points
    max_health: u32,

    // Character class
    class: CharacterClass,

    // Character statistics
    stats: CharacterStats,

    // Equipped items
    // Note: Converted from [Item; 10] to dynamic array
    equipped_items: [Item],

    // Current guild (None if not in guild)
    guild: Option<PublicKey>,

    // Last login timestamp
    last_login: i64,

    // Total playtime in seconds
    playtime: u64,
}

// Character statistics and attributes
// Determines combat effectiveness
// Original: programs/rpg/src/state.rs:45
#[solana]
#[account]
struct CharacterStats {
    // Strength (affects physical damage)
    strength: u16,

    // Dexterity (affects speed and critical chance)
    dexterity: u16,

    // Intelligence (affects magic damage)
    intelligence: u16,

    // Vitality (affects health)
    vitality: u16,

    // Luck (affects loot drops)
    luck: u16,

    // Total stat points to allocate
    available_points: u16,

    // Armor rating
    armor: u32,

    // Magic resistance
    magic_resist: u32,
}

// Player inventory storing items and currency
// Original: programs/rpg/src/state.rs:70
#[solana]
#[account]
struct Inventory {
    // Owner's wallet
    owner: PublicKey,

    // Gold currency
    gold: u64,

    // Gems (premium currency)
    gems: u32,

    // Item slots (max 100 items)
    items: [Item],

    // Total inventory weight
    weight: u32,
}

// Guild account for player groups
// Original: programs/rpg/src/state.rs:90
#[solana]
#[account]
struct Guild {
    // Guild leader's wallet
    leader: PublicKey,

    // Guild name
    name: String,

    // Guild description
    description: String,

    // Member wallet addresses
    members: [PublicKey],

    // Maximum members allowed
    max_members: u32,

    // Guild level
    level: u16,

    // Guild experience
    experience: u64,

    // Guild treasury (gold)
    treasury: u64,

    // Creation timestamp
    created_at: i64,

    // Is guild recruiting?
    recruiting: bool,
}

// ───────────────────────────────────────────────────────────
// Data Types
// ───────────────────────────────────────────────────────────

// Quest definition and progress
// Original: programs/rpg/src/state.rs:125
#[solana]
struct Quest {
    // Quest ID
    id: u32,

    // Quest name
    name: String,

    // Quest description
    description: String,

    // Experience reward
    exp_reward: u64,

    // Gold reward
    gold_reward: u64,

    // Is quest completed?
    completed: bool,
}

// In-game item
// Original: programs/rpg/src/state.rs:145
#[solana]
struct Item {
    // Item ID
    id: u32,

    // Item name
    name: String,

    // Item description
    description: String,

    // Item rarity (1=common, 5=legendary)
    rarity: u8,

    // Base value in gold
    value: u64,

    // Item weight
    weight: u32,

    // Is item stackable?
    stackable: bool,

    // Stack quantity (if stackable)
    quantity: u32,
}

// ───────────────────────────────────────────────────────────
// Enumerations
// ───────────────────────────────────────────────────────────

// Game events for logging and notifications
// Original: programs/rpg/src/state.rs:170
#[solana]
enum GameEvent {
    // Player created new character
    PlayerCreated(PublicKey),

    // Player leveled up
    LevelUp {
        player: PublicKey,
        new_level: u16,
    },

    // Item acquired
    ItemAcquired {
        player: PublicKey,
        item: Item,
    },

    // Quest completed
    QuestCompleted {
        player: PublicKey,
        quest_id: u32,
    },

    // Guild joined
    GuildJoined {
        player: PublicKey,
        guild: PublicKey,
    },
}

// Character classes
// Original: programs/rpg/src/state.rs:200
#[solana]
enum CharacterClass {
    // Melee fighter (high strength)
    Warrior,

    // Ranged attacker (high dexterity)
    Archer,

    // Magic caster (high intelligence)
    Mage,

    // Support class (balanced stats)
    Cleric,
}
```

---

### Step 4: Validate Extracted Schema

**Command:**
```bash
lumos validate lumos/game-types.lumos
```

**Output:**
```
✓ Validating lumos/game-types.lumos...

  Syntax check ✓
  Type definitions ✓
  Field types ✓
  Enum variants ✓
  Dependencies ✓

✓ Schema is valid!

Summary:
  Structs: 6
  Enums: 2
  Total fields: 68
  No errors, no warnings

Ready to generate code:
  lumos generate lumos/game-types.lumos
```

---

### Step 5: Generate Code

**Command:**
```bash
lumos generate lumos/game-types.lumos \
  --rust programs/rpg/src/types.rs \
  --typescript app/src/types.ts
```

**Output:**
```
✓ Generating from lumos/game-types.lumos...

  Rust generator ✓
    → programs/rpg/src/types.rs (420 lines)

  TypeScript generator ✓
    → app/src/types.ts (380 lines)

✓ Generation complete!

Files generated:
  programs/rpg/src/types.rs  (420 lines, 12.3 KB)
  app/src/types.ts           (380 lines, 10.8 KB)

Next steps:
  1. Update imports in lib.rs to use generated types
  2. Update TypeScript imports to use generated types
  3. Run tests: cargo test && npm test
  4. Commit changes: git add . && git commit -m "Migrate to LUMOS"
```

---

### Step 6: Update Rust Imports

**Before:** `programs/rpg/src/lib.rs`
```rust
mod state;

use state::{
    PlayerAccount,
    CharacterStats,
    Inventory,
    // ... etc
};
```

**After:** `programs/rpg/src/lib.rs`
```rust
mod types;  // Generated by LUMOS

use types::{
    PlayerAccount,
    CharacterStats,
    Inventory,
    // ... all types from LUMOS
};
```

**Delete old file:**
```bash
rm programs/rpg/src/state.rs
```

---

### Step 7: Update TypeScript Imports

**Before:** `app/src/rpg-client.ts`
```typescript
// Manual types
interface PlayerAccount {
    owner: PublicKey;
    name: string;
    // ... hope these match Rust!
}

const playerSchema = borsh.struct([
    borsh.publicKey('owner'),
    borsh.string('name'),
    // ... manual schema maintenance
]);
```

**After:** `app/src/rpg-client.ts`
```typescript
// Generated types (guaranteed to match Rust!)
import {
    PlayerAccount,
    playerAccountBorshSchema
} from './types';

// Use generated schema
const player = borsh.deserialize(
    playerAccountBorshSchema,
    accountData
);
```

**Delete old file:**
```bash
rm app/src/types.ts
```

---

### Step 8: Test Migration

**Command:**
```bash
# Test Rust compilation
cd programs/rpg
cargo test

# Test TypeScript types
cd ../../app
npm run type-check
npm test

# Integration tests
npm run test:integration
```

**Result:**
```
✓ All tests passing!

Rust: 45/45 tests passed
TypeScript: 32/32 tests passed
Integration: 12/12 tests passed

Migration successful! 🎉
```

---

## Before vs After Comparison

### Before LUMOS (Manual Duplication)

**Development workflow:**
```
1. Write Rust struct
2. Manually copy to TypeScript
3. Create Borsh schema manually
4. Test and find type mismatches
5. Fix mismatches
6. Repeat for every change
```

**Type drift example:**
```rust
// Rust (updated)
pub struct Player {
    pub wallet: Pubkey,
    pub score: u64,
    pub level: u32,  // Changed from u16
}
```

```typescript
// TypeScript (forgot to update!)
interface Player {
    wallet: PublicKey;
    score: number;
    level: number;  // Still thinks it's u16!
}
```

**Result:** Runtime deserialization errors! 💥

---

### After LUMOS (Single Source of Truth)

**Development workflow:**
```
1. Edit .lumos schema
2. Run: lumos generate
3. Use generated types
4. Types guaranteed to match!
```

**Type update example:**
```rust
// Update once
struct Player {
    wallet: PublicKey,
    score: u64,
    level: u32,  // Change here
}
```

```bash
lumos generate schema.lumos
```

**Result:**
- Rust: `level: u32` ✓
- TypeScript: `level: number` (u32) ✓
- Borsh schema: Updated automatically ✓
- **No type drift possible!** ✅

---

## Time Savings Analysis

### Manual Approach (Before)

**Initial development:**
- Write Rust types: 2 hours
- Write TypeScript types: 1.5 hours
- Create Borsh schemas: 1 hour
- Debug type mismatches: 2 hours
- **Total: 6.5 hours**

**Each change:**
- Update Rust: 10 minutes
- Update TypeScript: 10 minutes
- Update Borsh schema: 5 minutes
- Test and debug: 20 minutes
- **Total: 45 minutes per change**

**For 20 changes during development:**
- Initial: 6.5 hours
- Changes: 20 × 45 min = 15 hours
- **Total: 21.5 hours**

---

### LUMOS Approach (After)

**Initial setup:**
- Write LUMOS schema: 1.5 hours
- Run extraction tool: 5 minutes
- Generate code: 1 minute
- Test: 30 minutes
- **Total: 2 hours**

**Each change:**
- Update LUMOS schema: 5 minutes
- Regenerate: 1 minute
- Test: 5 minutes
- **Total: 11 minutes per change**

**For 20 changes during development:**
- Initial: 2 hours
- Changes: 20 × 11 min = 3.7 hours
- **Total: 5.7 hours**

---

### Savings

**Time saved:**
- 21.5 hours - 5.7 hours = **15.8 hours saved!**
- That's **73% reduction** in type maintenance time
- Equivalent to **2 full work days**

**Plus:**
- ✅ Zero type drift bugs
- ✅ Guaranteed serialization compatibility
- ✅ Easier onboarding for new developers
- ✅ Confidence in refactoring

---

## Extraction Tool Value Proposition

### For Individual Developers

**Time savings:**
- Initial migration: 15 minutes (vs 4+ hours manual)
- Ongoing changes: 70% faster
- Bug prevention: Hours saved debugging type mismatches

### For Teams

**Consistency:**
- Everyone uses same schema
- Code reviews focus on logic, not type sync
- New members productive faster

### For Open Source Projects

**Adoption:**
- Easy migration path from existing code
- Lower barrier to entry
- More contributors willing to help

---

## Conclusion

**The extraction tool makes LUMOS adoption:**
- ⚡ **Fast** - Minutes instead of hours
- 🎯 **Accurate** - Automated conversion
- 🔒 **Safe** - Validation before generation
- 🤝 **Interactive** - Guided process for edge cases

**Result:** Existing Solana projects can adopt LUMOS with minimal friction, immediately gaining benefits of type safety and eliminating type drift.

---

**Status:** Design complete, ready for implementation
**Priority:** Critical for ecosystem adoption
**Estimated impact:** 10x easier migration for existing projects
