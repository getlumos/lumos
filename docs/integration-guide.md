# LUMOS Integration Guide for Existing Projects

**Purpose:** Step-by-step guide for integrating LUMOS into existing Solana projects

**Last Updated:** 2025-11-18

---

## Overview

LUMOS currently works **one-way**:

```
.lumos files → LUMOS Generator → Rust + TypeScript code
```

This guide covers strategies for migrating existing Solana projects to LUMOS.

---

## Table of Contents

1. [When to Use LUMOS](#when-to-use-lumos)
2. [Integration Strategies](#integration-strategies)
3. [Step-by-Step Migration](#step-by-step-migration)
4. [Type Mapping Reference](#type-mapping-reference)
5. [Common Patterns](#common-patterns)
6. [Troubleshooting](#troubleshooting)
7. [Future: Reverse Engineering](#future-reverse-engineering)

---

## When to Use LUMOS

### ✅ Good Fit

- **Type drift problems:** Rust and TypeScript types don't match
- **Frequent schema changes:** Types change often during development
- **New features:** Adding new account structures
- **Borsh serialization bugs:** Manual schema maintenance errors
- **Multiple clients:** Different languages accessing same accounts

### ❌ Not Ideal (Yet)

- **Legacy codebase freeze:** No active development
- **Custom serialization:** Not using Borsh
- **Non-Solana projects:** LUMOS is Solana-specific
- **Complex generic types:** LUMOS doesn't support generics yet

---

## Integration Strategies

### Strategy 1: Gradual Migration (Recommended)

**Best for:** Large existing projects, risk-averse teams

**Approach:**
- Start with new features
- Migrate high-churn types
- Keep old code working
- Gradual replacement

**Timeline:** 2-4 weeks for medium projects

**Pros:**
- ✅ Low risk
- ✅ Immediate value from new types
- ✅ Test LUMOS before full commitment

**Cons:**
- ❌ Temporary code duplication
- ❌ Longer migration period

---

### Strategy 2: Full Migration (Clean Slate)

**Best for:** Small projects, refactoring opportunities

**Approach:**
- Convert all types at once
- Delete old type definitions
- Update all imports
- Comprehensive testing

**Timeline:** 1-3 days for small projects

**Pros:**
- ✅ Clean codebase
- ✅ Immediate full benefits
- ✅ No legacy code

**Cons:**
- ❌ Higher risk
- ❌ Requires thorough testing
- ❌ Potential downtime

---

### Strategy 3: Hybrid (Feature Flags)

**Best for:** Testing LUMOS, maintaining flexibility

**Approach:**
- Use Cargo features to toggle
- Parallel type definitions
- Test both paths
- Switch when confident

**Timeline:** 1 week evaluation + 1 week migration

**Pros:**
- ✅ Safe evaluation
- ✅ Easy rollback
- ✅ Confidence building

**Cons:**
- ❌ Maintenance overhead
- ❌ More complex build process

---

## Step-by-Step Migration

### Example Project: "Solana NFT Marketplace"

**Starting point:**
```
nft-marketplace/
├── programs/marketplace/src/
│   ├── lib.rs (500 lines)
│   └── state.rs (15 structs, 300 lines)
└── app/src/
    ├── index.ts
    └── types.ts (15 interfaces, 200 lines)
```

---

### Phase 1: Setup (15 minutes)

#### 1.1 Install LUMOS

```bash
cargo install lumos-cli
```

#### 1.2 Create LUMOS Directory

```bash
cd nft-marketplace
mkdir lumos
touch lumos/.gitkeep
```

#### 1.3 Create Configuration

```bash
cat > .lumos.toml <<EOF
[generation]
rust_output = "programs/marketplace/src/generated.rs"
typescript_output = "app/src/generated.ts"

[watch]
enabled = false  # Enable later when ready
input = "lumos/*.lumos"
EOF
```

#### 1.4 Add to .gitignore

```bash
cat >> .gitignore <<EOF

# LUMOS generated files (committed but marked for clarity)
**/generated.rs
**/generated.ts
EOF
```

**Note:** You can commit generated files (recommended) or regenerate in CI.

---

### Phase 2: First Migration (30 minutes)

#### 2.1 Choose Starting Type

**Select a simple, frequently changing type:**

```rust
// programs/marketplace/src/state.rs
#[account]
pub struct Listing {
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub price: u64,
    pub is_active: bool,
}
```

#### 2.2 Convert to LUMOS

**Create:** `lumos/schema.lumos`

```lumos
// NFT Marketplace Types
// Migrated from programs/marketplace/src/state.rs

#[solana]
#[account]
struct Listing {
    seller: PublicKey,
    nft_mint: PublicKey,
    price: u64,
    is_active: bool,
}
```

#### 2.3 Generate Code

```bash
lumos generate lumos/schema.lumos \
  --rust programs/marketplace/src/generated.rs \
  --typescript app/src/generated.ts
```

**Output:**
```
✓ Generated programs/marketplace/src/generated.rs
✓ Generated app/src/generated.ts
```

#### 2.4 Update Imports (Rust)

```rust
// programs/marketplace/src/lib.rs

// Add new module
mod generated;

// Keep old for now
mod state;

// Use generated type
use crate::generated::Listing as GeneratedListing;
use crate::state::Listing as OldListing;

// TODO: Switch to GeneratedListing everywhere
```

#### 2.5 Test Generation

```bash
# Verify Rust compiles
cd programs/marketplace
cargo check

# Verify TypeScript types
cd ../../app
npm run type-check  # or tsc --noEmit
```

---

### Phase 3: Incremental Migration (1-2 weeks)

#### 3.1 Week 1: High-Priority Types

**Migrate types that:**
- Change frequently
- Cause type drift issues
- Are newly created

**Example migration priority:**

| Type | Priority | Reason |
|------|----------|--------|
| `Listing` | HIGH | Changes often, central to app |
| `Offer` | HIGH | New feature, perfect for LUMOS |
| `UserProfile` | MEDIUM | Stable but good candidate |
| `Config` | LOW | Rarely changes |
| `InternalState` | SKIP | Internal only, no client sync |

**Add to schema.lumos:**

```lumos
#[solana]
#[account]
struct Listing {
    seller: PublicKey,
    nft_mint: PublicKey,
    price: u64,
    is_active: bool,
}

#[solana]
#[account]
struct Offer {
    buyer: PublicKey,
    listing: PublicKey,
    amount: u64,
    expiry: i64,
}

#[solana]
#[account]
struct UserProfile {
    owner: PublicKey,
    username: String,
    listings_count: u32,
}
```

**Regenerate:**
```bash
lumos generate lumos/schema.lumos
```

#### 3.2 Update Code Gradually

**Pattern: Alias old → new**

```rust
// programs/marketplace/src/lib.rs

mod generated;
mod state;  // Still has some old types

// Migrate one type at a time
pub use generated::Listing;
pub use generated::Offer;
pub use state::{Config, InternalState};  // Not migrated yet

// Code using Listing now uses generated version!
```

**Verify:**
```bash
cargo test
```

#### 3.3 Week 2: Complete Migration

**Add remaining types to LUMOS:**

```lumos
// Complete schema.lumos

#[solana]
#[account]
struct Listing { ... }

#[solana]
#[account]
struct Offer { ... }

#[solana]
#[account]
struct UserProfile { ... }

#[solana]
struct Config {
    marketplace_fee: u16,
    authority: PublicKey,
}

// Note: InternalState not migrated (no client sync needed)
```

---

### Phase 4: Cleanup (1-2 hours)

#### 4.1 Remove Old Type Definitions

**Delete migrated types from state.rs:**

```rust
// programs/marketplace/src/state.rs

// BEFORE: 15 structs
#[account]
pub struct Listing { ... }  // ← Delete

#[account]
pub struct Offer { ... }  // ← Delete

// ... delete all migrated types

// AFTER: Only types not in LUMOS remain
pub struct InternalState { ... }  // Keep (internal only)
```

#### 4.2 Simplify Imports

```rust
// programs/marketplace/src/lib.rs

mod generated;
mod state;  // Only for internal types now

// Use everything from generated
pub use generated::*;

// Only specific internal types from state
use state::InternalState;
```

#### 4.3 Update TypeScript

**Delete old types.ts:**

```bash
rm app/src/types.ts
```

**Update imports:**

```typescript
// app/src/marketplace.ts

// Before
import { Listing, Offer } from './types';

// After
import { Listing, Offer } from './generated';
```

#### 4.4 Final Testing

```bash
# Full test suite
cd programs/marketplace
cargo test-bpf

# Client tests
cd ../../app
npm test

# Integration tests
npm run test:integration
```

---

### Phase 5: Automation (30 minutes)

#### 5.1 Enable Watch Mode (Optional)

**Update .lumos.toml:**

```toml
[watch]
enabled = true
input = "lumos/*.lumos"
```

**Or use VSCode extension:**
- Install LUMOS VSCode extension
- Enable "auto-generate on save"
- Edit `.lumos` → auto-regenerates Rust/TypeScript!

#### 5.2 Add to Build Scripts

**package.json:**

```json
{
  "scripts": {
    "lumos:generate": "lumos generate lumos/schema.lumos",
    "prebuild": "npm run lumos:generate",
    "dev": "npm run lumos:generate && npm run start"
  }
}
```

**Cargo build script (optional):**

```rust
// programs/marketplace/build.rs
use std::process::Command;

fn main() {
    // Regenerate LUMOS types before build
    Command::new("lumos")
        .args(&["generate", "lumos/schema.lumos"])
        .output()
        .expect("Failed to generate LUMOS types");

    println!("cargo:rerun-if-changed=lumos/schema.lumos");
}
```

---

## Type Mapping Reference

### Rust → LUMOS

| Rust | LUMOS | Notes |
|------|-------|-------|
| `Pubkey` | `PublicKey` | Solana address type |
| `u8`, `u16`, `u32`, `u64`, `u128` | Same | Unsigned integers |
| `i8`, `i16`, `i32`, `i64`, `i128` | Same | Signed integers |
| `bool` | `bool` | Boolean |
| `String` | `String` | Dynamic string |
| `Vec<T>` | `[T]` | Dynamic array |
| `Option<T>` | `Option<T>` | Optional value |
| `[T; N]` | Not supported yet | Fixed-size array |

### Complex Examples

**Rust:**
```rust
#[account]
pub struct ComplexAccount {
    pub owner: Pubkey,
    pub balances: Vec<u64>,
    pub metadata: Option<String>,
    pub tags: Vec<String>,
}
```

**LUMOS:**
```lumos
#[solana]
#[account]
struct ComplexAccount {
    owner: PublicKey,
    balances: [u64],
    metadata: Option<String>,
    tags: [String],
}
```

---

## Common Patterns

### Pattern 1: Coexistence

**Use both old and generated types during migration:**

```rust
mod generated;
mod state;

use generated::NewType;
use state::OldType;

pub fn migration_function(
    old: &OldType,
    new: &mut NewType,
) {
    // Convert old to new
    new.field = old.field;
}
```

### Pattern 2: Type Aliases

**Maintain compatibility with existing code:**

```rust
// Public API uses alias
pub use generated::PlayerAccount as Player;

// Old code works unchanged
pub fn process_player(player: &Player) { ... }
```

### Pattern 3: Wrapper Structs

**Add functionality to generated types:**

```rust
use crate::generated::PlayerAccount;

pub struct PlayerWrapper {
    inner: PlayerAccount,
}

impl PlayerWrapper {
    pub fn new(account: PlayerAccount) -> Self {
        Self { inner: account }
    }

    pub fn calculate_power(&self) -> u64 {
        self.inner.level as u64 * self.inner.experience
    }
}
```

---

## Troubleshooting

### Issue: Generated Code Doesn't Compile

**Symptom:** Rust compilation errors after generation

**Solutions:**

1. **Check type mapping:**
   ```bash
   # Verify LUMOS types are correct
   lumos validate lumos/schema.lumos
   ```

2. **Regenerate:**
   ```bash
   rm programs/*/src/generated.rs
   lumos generate lumos/schema.lumos
   ```

3. **Check for name conflicts:**
   ```rust
   // If you have both:
   mod generated;
   mod state;

   use generated::Account;  // Conflict!
   use state::Account;      // Conflict!

   // Solution: Use qualified paths
   use generated::Account as GenAccount;
   ```

### Issue: Type Drift Still Occurring

**Symptom:** Rust and TypeScript types don't match

**Cause:** Not using generated types everywhere

**Solution:**

```bash
# Search for old type usage
rg "use crate::state::" programs/

# Replace with generated
# use crate::state::Account → use crate::generated::Account
```

### Issue: Borsh Serialization Errors

**Symptom:** Runtime deserialization failures

**Cause:** Field order mismatch

**Solution:**

LUMOS generates correct field order. Ensure:
1. Using LUMOS-generated Borsh schemas
2. Not manually creating schemas
3. All fields in correct order

---

## Future: Reverse Engineering

### Planned Feature: `lumos extract`

**Command to extract from existing Rust:**

```bash
# Extract types from Rust file
lumos extract programs/game/src/state.rs --output lumos/schema.lumos

# Preview extraction
lumos extract src/state.rs --dry-run
```

**Would analyze:**
```rust
#[account]
pub struct Player {
    pub wallet: Pubkey,
    pub score: u64,
}
```

**And generate:**
```lumos
#[solana]
#[account]
struct Player {
    wallet: PublicKey,
    score: u64,
}
```

**Status:** Planned for Phase 3.3+

**Tracking:** https://github.com/RECTOR-LABS/lumos/issues/TBD

---

## Migration Checklist

### Pre-Migration

- [ ] Install LUMOS CLI
- [ ] Create `lumos/` directory
- [ ] Create `.lumos.toml` config
- [ ] Identify types to migrate
- [ ] Prioritize migration order

### During Migration

- [ ] Convert types to LUMOS syntax
- [ ] Generate code
- [ ] Update imports gradually
- [ ] Test after each type migration
- [ ] Run full test suite frequently

### Post-Migration

- [ ] Delete old type definitions
- [ ] Clean up imports
- [ ] Enable watch mode (optional)
- [ ] Add to build scripts
- [ ] Update documentation
- [ ] Train team on LUMOS workflow

---

## Best Practices

### 1. Start Small

Migrate 1-2 types first, verify everything works, then scale up.

### 2. Test Frequently

Run tests after each type migration to catch issues early.

### 3. Commit Generated Code

Commit `generated.rs` and `generated.ts` to Git for build reproducibility.

### 4. Document Schema

Add comments to `.lumos` files explaining types:

```lumos
// Player account stores user game state
// Updated: 2025-01-18
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,  // Owner's wallet address
    score: u64,         // Current game score
}
```

### 5. Version Schema Changes

Use Git tags or version comments:

```lumos
// Schema version: v1.2.0
// Last breaking change: 2025-01-15
```

---

## Support

**Issues:** https://github.com/RECTOR-LABS/lumos/issues
**Documentation:** https://github.com/RECTOR-LABS/lumos/docs
**Examples:** https://github.com/RECTOR-LABS/lumos/examples

---

**Last Updated:** 2025-11-18
**Version:** 1.0.0
