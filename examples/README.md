# LUMOS Example Schemas

This directory contains real-world example schemas demonstrating LUMOS capabilities and best practices.

## 📁 Available Examples

### 1. Gaming (`gaming/schema.lumos`)

**Use Case:** Blockchain gaming with player accounts, inventory, and leaderboards

**Features Demonstrated:**
- Mixed `#[account]` and non-account structs
- Array types for inventory management
- Optional fields for match opponents
- Public key references for player wallets

**Structures:**
- `PlayerAccount` - Main player data with level, XP, and equipped items
- `ItemDefinition` - Game item templates with rarity and stats
- `LeaderboardEntry` - Player rankings with scores
- `MatchResult` - Game match outcomes (non-account struct)

**Generated Code:**
- Rust: 4 structs with context-aware Anchor integration
- TypeScript: 4 interfaces with complete Borsh schemas

---

### 2. NFT Marketplace (`nft-marketplace/schema.lumos`)

**Use Case:** Decentralized NFT trading platform

**Features Demonstrated:**
- `Signature` type handling (maps to `String` in Rust, `string` in TypeScript)
- Optional fields for active listings
- Complex nested data structures
- Transaction receipt tracking

**Structures:**
- `Marketplace` - Platform configuration and fee structure
- `Listing` - Active NFT listings with pricing
- `NFTMetadata` - On-chain metadata storage
- `PurchaseReceipt` - Transaction history records

**Real-World Pattern:** Base58 signature encoding automatically handled

---

### 3. DeFi Staking (`defi-staking/schema.lumos`)

**Use Case:** Token staking protocol with rewards

**Features Demonstrated:**
- Time-based logic (timestamps)
- Reward calculation fields
- Pool configuration management
- Staker account tracking

**Structures:**
- `StakingPool` - Pool parameters and total stakes
- `StakerAccount` - Individual staker position and rewards
- `RewardEvent` - Reward distribution records

**Type Safety:** Timestamp handling ensures consistent u64 usage across languages

---

### 4. DAO Governance (`dao-governance/schema.lumos`)

**Use Case:** Decentralized autonomous organization voting

**Features Demonstrated:**
- All structs using `#[account]` (pure Anchor mode)
- Demonstrates context-aware import generation
- Vote tracking and proposal management
- Member role management

**Structures:**
- `DAOConfig` - Organization configuration
- `Proposal` - Governance proposals with voting periods
- `Vote` - Individual vote records
- `Member` - DAO membership data

**Import Strategy:** Automatically uses `anchor_lang::prelude::*` for entire module

---

### 5. Token Vesting (`token-vesting/schema.lumos`)

**Use Case:** Token release schedules for team/investors

**Features Demonstrated:**
- Time-based vesting calculations
- Beneficiary tracking
- Cliff and linear vesting support
- Release schedule management

**Structures:**
- `VestingSchedule` - Vesting parameters and timelines
- `Beneficiary` - Recipient information and claimed amounts

**Business Logic:** Shows how schemas support complex financial instruments

---

### 6. Enum Patterns (`enums/schema.lumos`)

**Use Case:** Comprehensive enum usage patterns for Solana

**Features Demonstrated:**
- **Unit enums** - State machines (`Active`, `Paused`, `Finished`)
- **Tuple enums** - Data-carrying variants (`PlayerJoined(PublicKey, u64)`)
- **Struct enums** - Solana instruction pattern (`Initialize { authority: PublicKey }`)
- Enum fields in structs
- Complex event types
- Mixed variant types

**Structures:**
- `GameState` - Unit enum for simple states
- `PlayerStatus` - Unit enum with 4 states
- `GameEvent` - Tuple enum for event logging
- `GameInstruction` - Struct enum (Anchor instruction pattern)
- `GameAccount` - Using enums as struct fields
- `ComplexEvent` - Mixed enum variant types

**Code Generation:**
- Rust: Native `enum` with proper derives
- TypeScript: Discriminated unions with `kind` field for type narrowing
- Borsh: Sequential discriminants (0, 1, 2...)

**Real-World Pattern:** Matches Solana/Anchor instruction enum convention

---

## 🚀 Using These Examples

### Generate Code from Example

```bash
# Navigate to LUMOS root directory
cd /path/to/lumos

# Generate Rust + TypeScript for gaming example
lumos generate examples/gaming/schema.lumos

# Generate for NFT marketplace
lumos generate examples/nft-marketplace/schema.lumos

# Generate for all examples
for dir in examples/*/; do
    lumos generate "${dir}schema.lumos"
done
```

### Validate Example Syntax

```bash
# Check if schema is valid
lumos validate examples/gaming/schema.lumos

# Validate all examples
for dir in examples/*/; do
    echo "Validating ${dir}..."
    lumos validate "${dir}schema.lumos"
done
```

### Testing Generated Code

Each example generates code that can be tested:

**Rust:**
```bash
# The generated Rust code is production-ready
# Copy to your Anchor project's src/ directory
cp examples/gaming/generated/rust/schema.rs \
   your-anchor-project/programs/your-program/src/state.rs

# Rust code compiles with Anchor or standalone Borsh
```

**TypeScript:**
```bash
# Generated TypeScript has Borsh schemas
# Copy to your frontend SDK
cp examples/gaming/generated/typescript/schema.ts \
   your-frontend/src/types/

# Use in your code:
# import { PlayerAccount, PlayerAccountBorshSchema } from './types/schema';
```

---

## 📋 Example Comparison Matrix

| Example | Structs | #[account] | Non-Account | Enums | Arrays | Optional | Signatures |
|---------|---------|------------|-------------|-------|--------|----------|------------|
| Gaming | 4 | 3 | 1 | 0 | Yes | Yes | No |
| NFT Marketplace | 4 | 4 | 0 | 0 | Yes | Yes | Yes |
| DeFi Staking | 3 | 3 | 0 | 0 | No | Yes | No |
| DAO Governance | 4 | 4 | 0 | 0 | No | No | No |
| Token Vesting | 2 | 2 | 0 | 0 | No | No | No |
| Enum Patterns | 5 | 1 | 0 | 4 | Yes | Yes | No |

---

## 🎓 Learning Path

**Beginner:** Start with `token-vesting` (simplest structure)
**Intermediate:** Try `gaming` or `defi-staking` (arrays and optional fields)
**Advanced:** Study `nft-marketplace` (signatures, complex types)
**Expert:** Master `enum-patterns` (all 3 enum variant types, discriminated unions)

---

## 🧪 Test Coverage

All examples are tested in the LUMOS test suite:

- **Parser Integration Tests:** 5 examples parsed successfully
- **Rust Generator Tests:** All examples generate valid Rust
- **TypeScript Generator Tests:** All examples generate valid TypeScript
- **E2E Compilation Tests:** Generated Rust code compiles with `cargo check`

Run tests:
```bash
cd packages/core
cargo test
```

---

## 📝 Creating Your Own Examples

Use these examples as templates for your own schemas:

1. **Copy closest match** - Find example similar to your use case
2. **Modify structures** - Adapt field types and names
3. **Add attributes** - Use `#[solana]`, `#[account]`, etc.
4. **Generate code** - Run `lumos generate your-schema.lumos`
5. **Integrate** - Copy generated code to your project

---

## 🔗 Related Documentation

- [LUMOS Syntax Guide](../docs/syntax.md) - Complete language reference
- [Type Mapping](../README.md#-type-mapping) - LUMOS ↔ Rust ↔ TypeScript types
- [Enum Design](../docs/enum-design.md) - Comprehensive enum documentation (500+ lines)
- [Execution Plan](../docs/execution-plan.md) - Development roadmap

---

## 💡 Tips & Best Practices

**Naming Conventions:**
- Use `PascalCase` for struct/enum names (`PlayerAccount`)
- Use `snake_case` for field names (`equipped_items`)
- Match Solana/Anchor conventions

**#[account] Usage:**
- Use for on-chain account storage
- Omit for client-side types (events, parameters)
- Anchor provides derives automatically

**Type Selection:**
- Use `u64` for balances, timestamps (not `u128` unless needed)
- Use `PublicKey`/`Pubkey` (both work) for addresses
- Use `[T]` for dynamic arrays, `Option<T>` for optional fields

**Performance:**
- Keep structs focused and minimal
- Avoid deep nesting (flattens serialization)
- Consider Borsh size limits for Solana transactions

---

## 🤝 Contributing Examples

Have a great real-world use case? Contribute an example!

**Requirements:**
- Represents real Solana development pattern
- Demonstrates unique LUMOS feature
- Includes clear documentation
- Passes all tests (`cargo test`)

Submit via pull request with:
- `examples/your-example/schema.lumos`
- Description in this README
- Test verification

---

**Last Updated:** 2025-11-18
**Examples:** 6 total (5 struct-based + 1 enum-focused)
**Test Coverage:** 100% (all examples tested)
