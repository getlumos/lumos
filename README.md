<div align="center">

<pre>
██╗     ██╗   ██╗███╗   ███╗ ██████╗ ███████╗
██║     ██║   ██║████╗ ████║██╔═══██╗██╔════╝
██║     ██║   ██║██╔████╔██║██║   ██║███████╗
██║     ██║   ██║██║╚██╔╝██║██║   ██║╚════██║
███████╗╚██████╔╝██║ ╚═╝ ██║╚██████╔╝███████║
╚══════╝ ╚═════╝ ╚═╝     ╚═╝ ╚═════╝ ╚══════╝
</pre>

# LUMOS

> **Write once. Deploy Everywhere.**

**Illuminate your Solana development with type-safe cross-language code generation**

*One schema to rule them all • TypeScript ↔ Rust synchronization • Borsh serialization • Anchor integration • Zero type drift • Production-ready code generation*

[![Crates.io](https://img.shields.io/crates/v/lumos-core?label=lumos-core)](https://crates.io/crates/lumos-core)
[![Crates.io](https://img.shields.io/crates/v/lumos-cli?label=lumos-cli)](https://crates.io/crates/lumos-cli)
[![Crates.io](https://img.shields.io/crates/v/lumos-lsp?label=lumos-lsp)](https://crates.io/crates/lumos-lsp)
[![npm](https://img.shields.io/npm/v/@getlumos/cli?label=npm)](https://www.npmjs.com/package/@getlumos/cli)
[![Cargo Downloads](https://img.shields.io/crates/d/lumos-cli?label=cargo%20installs)](https://crates.io/crates/lumos-cli)
[![npm Downloads](https://img.shields.io/npm/dm/@getlumos/cli?label=npm%20installs)](https://www.npmjs.com/package/@getlumos/cli)
[![Docker](https://img.shields.io/badge/Docker-ghcr.io-blue.svg?logo=docker)](https://github.com/getlumos/lumos/pkgs/container/lumos)
[![GitHub Action](https://img.shields.io/badge/Marketplace-LUMOS%20Generate-purple.svg?logo=github)](https://github.com/marketplace/actions/lumos-generate)
[![CI](https://img.shields.io/github/actions/workflow/status/getlumos/lumos/ci.yml?branch=main&label=CI%20(main)&logo=github)](https://github.com/getlumos/lumos/actions)
[![CI](https://img.shields.io/github/actions/workflow/status/getlumos/lumos/ci.yml?branch=dev&label=CI%20(dev)&logo=github)](https://github.com/getlumos/lumos/actions)
[![Coverage](https://codecov.io/gh/getlumos/lumos/branch/dev/graph/badge.svg)](https://codecov.io/gh/getlumos/lumos)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Made for Solana](https://img.shields.io/badge/Made%20for-Solana-9945FF.svg)](https://solana.com)
[![Phase 1 Complete](https://img.shields.io/badge/Phase%201-Complete-success.svg)](#-roadmap)
[![Phase 2 Complete](https://img.shields.io/badge/Phase%202-Complete-success.svg)](#-roadmap)
[![Phase 3.1 Complete](https://img.shields.io/badge/Phase%203.1-Complete-success.svg)](#-roadmap)
[![Phase 3.2 Complete](https://img.shields.io/badge/Phase%203.2-Complete-success.svg)](#-roadmap)
[![Tests Passing](https://img.shields.io/badge/Tests-64%2F64%20passing-brightgreen.svg)](#-test-suite)
[![VSCode Extension](https://img.shields.io/badge/VSCode-Extension%20Ready-blue.svg)](https://github.com/getlumos/vscode-lumos)

</div>

---

## 🌟 What is LUMOS?

LUMOS is a **powerful code generation framework** that bridges TypeScript and Rust, eliminating the pain of maintaining duplicate type definitions across your full-stack Solana applications. Write your data structures **once** in LUMOS syntax, and automatically generate perfectly synchronized code for both languages with guaranteed Borsh serialization compatibility.

**Stop writing the same types twice. Start building faster.**

---

## 🎥 Quick Preview

### Input: Single LUMOS Schema

```rust
#[solana]
#[account]
struct UserAccount {
    wallet: PublicKey,
    balance: u64,
    level: u16,
    equipped_items: [PublicKey],
}
```

### Output: Production-Ready Code

<table>
<tr>
<td width="50%">

**Rust (Anchor Program)**
```rust
use anchor_lang::prelude::*;

#[account]
pub struct UserAccount {
    pub wallet: Pubkey,
    pub balance: u64,
    pub level: u16,
    pub equipped_items: Vec<Pubkey>,
}
```

</td>
<td width="50%">

**TypeScript (Frontend SDK)**
```typescript
import { PublicKey } from '@solana/web3.js';
import { publicKey, u64, u16, vec } from '@coral-xyz/borsh';

export interface UserAccount {
  wallet: PublicKey;
  balance: number;
  level: number;
  equipped_items: PublicKey[];
}

export const UserAccountBorshSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.u64('balance'),
  borsh.u16('level'),
  borsh.vec(borsh.publicKey(), 'equipped_items'),
]);
```

</td>
</tr>
</table>

**Result:** Guaranteed type safety, zero manual synchronization, instant Borsh compatibility.

---

## 📚 Table of Contents

- [What is LUMOS?](#-what-is-lumos)
- [Quick Preview](#-quick-preview)
- [The Problem](#-the-problem)
- [The Solution](#-the-solution)
- [Key Features](#-key-features)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Architecture](#️-architecture)
- [Examples](#-examples)
- [Type Mapping](#-type-mapping)
- [Roadmap](#-roadmap)
- [Long-term Vision](docs/VISION.md)
- [Tech Stack](#️-tech-stack)
- [Test Suite](#-test-suite)
- [Contributing](#-contributing)
- [License](#-license)
- [Credits](#-credits)

---

## 🎯 The Problem

Building full-stack Solana applications requires maintaining **identical type definitions in two languages**. This manual synchronization is error-prone, time-consuming, and a major source of bugs.

### The Pain Points

<table>
<tr>
<th width="50%">❌ Without LUMOS</th>
<th width="50%">✅ With LUMOS</th>
</tr>
<tr>
<td valign="top">

**Manual Duplication**
```rust
// programs/src/state.rs
#[derive(BorshSerialize, BorshDeserialize)]
pub struct GameState {
    pub player: Pubkey,
    pub score: u64,
    pub level: u16,
}
```

```typescript
// app/src/types.ts
interface GameState {
  player: PublicKey;
  score: number;
  level: number;
}
```

**Problems:**
- 🔴 Manual synchronization required
- 🔴 Type mismatches cause runtime errors
- 🔴 Refactoring breaks in multiple places
- 🔴 No single source of truth
- 🔴 Borsh schema written manually
- 🔴 Field order must match exactly

</td>
<td valign="top">

**Single Source of Truth**
```rust
#[solana]
struct GameState {
    player: PublicKey,
    score: u64,
    level: u16,
}
```

**Benefits:**
- ✅ Define once, generate everywhere
- ✅ Types always in sync
- ✅ Refactor in one place
- ✅ Single schema file
- ✅ Borsh auto-generated
- ✅ Field order guaranteed

Run `lumos build` → Both implementations generated!

</td>
</tr>
</table>

### Real-World Consequences

| Issue | Impact | Frequency |
|-------|--------|-----------|
| **Type Drift** | Frontend expects `u64`, contract sends `u128` → deserialization fails | Every refactor |
| **Field Order Mismatch** | Borsh requires exact order → data corruption | Hard to debug |
| **Missing Fields** | Contract adds field, frontend doesn't know → crashes | Every update |
| **Version Skew** | Contract v2 deployed, frontend still uses v1 types → incompatible | Every deployment |

**LUMOS eliminates all of these issues.**

---

## 💡 The Solution

LUMOS provides a **custom domain-specific language (DSL)** with a powerful code generator that bridges TypeScript and Rust seamlessly.

### How It Works

```
   .lumos Schema File
          ↓
   ┌──────────────┐
   │    Parser    │  ← syn-based Rust parser
   │  (AST Gen)   │
   └──────┬───────┘
          ↓
   ┌──────────────┐
   │  Transform   │  ← AST → IR conversion
   └──────┬───────┘
          ↓
   ┌──────────────┐
   │      IR      │  ← Language-agnostic representation
   │ (Intermediate)│
   └──────┬───────┘
          ↓
   ┌──────────────┬──────────────┐
   ↓              ↓              ↓
┌─────────┐  ┌─────────┐  ┌─────────┐
│  Rust   │  │TypeScript│  │ Future  │
│Generator│  │Generator │  │  (C++,  │
│         │  │         │  │Python)  │
└────┬────┘  └────┬────┘  └─────────┘
     ↓            ↓
  .rs files   .ts files
```

### Core Capabilities

1. **Context-Aware Generation**
   - Detects Anchor usage → uses `anchor_lang::prelude::*`
   - Pure Borsh modules → uses `borsh::{BorshSerialize, BorshDeserialize}`
   - Mixed modules → smart import resolution

2. **Intelligent Derive Management**
   - `#[account]` structs → No manual derives (Anchor provides them)
   - Non-account structs → Appropriate derives based on context
   - Prevents derive conflicts automatically

3. **Type Safety Guarantee**
   - Complete bidirectional type mapping
   - Borsh schema auto-generation
   - Field order preservation
   - Optional types, vectors, and complex types supported

---

## ✨ Key Features

### 🎯 **Single Source of Truth**
Define your data structures once in `.lumos` syntax. LUMOS generates production-ready code for both Rust and TypeScript with guaranteed synchronization.

### 🔐 **100% Type Safety**
Complete type mapping ensures your Rust structs and TypeScript interfaces are always compatible. No more runtime deserialization errors.

### ⚓ **Anchor Framework Integration**
First-class support for Anchor programs. LUMOS understands `#[account]` attributes and generates appropriate code without derive conflicts.

### 📦 **Borsh Serialization Compatibility**
Automatic Borsh schema generation for both languages. Field order, type sizes, and serialization format guaranteed to match.

### 🧠 **Context-Aware Code Generation**
Intelligent analysis of your schemas determines the optimal imports, derives, and patterns for each target language.

### 🧩 **Extensible Architecture**
IR-based design makes adding new target languages straightforward. Future support planned for C++, Python, and more.

### ✅ **Production Ready**
- **50/50 tests passing** (100% success rate)
- E2E tests with actual Rust compilation verification
- Battle-tested on 5 real-world example schemas
- Clean, idiomatic code generation

### 🚀 **Developer Experience**
- Familiar Rust-style syntax (`#[attribute]` annotations)
- Clear error messages
- Fast compilation
- Zero runtime dependencies

---

## 📦 Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo package manager

### Install from crates.io (Recommended)

```bash
# Install the CLI
cargo install lumos-cli

# Verify installation
lumos --version
# lumos-cli 0.1.0

# Or add as library dependency
cargo add lumos-core
```

**Published Packages:**
- 📦 [lumos-core](https://crates.io/crates/lumos-core) - Core library (parser, generators, IR)
- 🔧 [lumos-cli](https://crates.io/crates/lumos-cli) - Command-line interface

### Install from Source

```bash
# Clone the repository
git clone https://github.com/getlumos/lumos.git
cd lumos

# Build the CLI
cargo build --release

# The binary will be available at: target/release/lumos
./target/release/lumos --help
```

### Install via npm (No Rust Required)

```bash
# Install globally
npm install -g @getlumos/cli

# Or use with npx
npx @getlumos/cli generate schema.lumos
```

### Use with Docker

```bash
# Run directly (no installation needed)
docker run --rm -v $(pwd):/workspace ghcr.io/getlumos/lumos generate schema.lumos

# Or pull first for offline use
docker pull ghcr.io/getlumos/lumos:latest
docker run --rm -v $(pwd):/workspace ghcr.io/getlumos/lumos generate schema.lumos -o generated/
```

**Docker in CI/CD (GitHub Actions):**

```yaml
- name: Generate code from schema
  uses: docker://ghcr.io/getlumos/lumos:latest
  with:
    args: generate schemas/*.lumos -o src/generated/
```

### Run Tests

```bash
cd packages/core
cargo test --all-features --workspace

# All 64 tests should pass ✅
```

---

## 🎓 Quick Start

### 1. Initialize a New Project

```bash
# Create a new LUMOS project
lumos init my-game

# Output:
#    Creating project: my-game
#     Created my-game/schema.lumos
#     Created my-game/lumos.toml
#     Created my-game/README.md
#    Finished project initialized
```

This creates:
- `schema.lumos` - Example schema file
- `lumos.toml` - Configuration file
- `README.md` - Quick start guide

### 2. Edit Your Schema

Open `schema.lumos` and define your data structures:

```rust
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    level: u16,
    experience: u64,
    equipped_items: [PublicKey],
}

#[solana]
struct MatchResult {
    player: PublicKey,
    opponent: Option<PublicKey>,
    score: u64,
    timestamp: i64,
}
```

### 3. Generate Code

```bash
# Generate Rust and TypeScript code
lumos generate schema.lumos

# Output:
#     Reading schema.lumos
#     Parsing schema
#  Generating Rust code
#       Wrote ./generated.rs
#  Generating TypeScript code
#       Wrote ./generated.ts
#    Finished generated 2 type definitions
```

### 4. Use Generated Code

**In your Rust program:**
```rust
// Import generated types
use crate::generated::*;

// Use in your Anchor program
#[program]
pub mod my_game {
    use super::*;

    pub fn create_player(ctx: Context<CreatePlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.level = 1;
        player.experience = 0;
        Ok(())
    }
}
```

**In your TypeScript app:**
```typescript
import { PlayerAccount, PlayerAccountSchema } from './generated';

// Deserialize on-chain data
const player = PlayerAccountSchema.deserialize(buffer);
console.log(`Level: ${player.level}, XP: ${player.experience}`);
```

### 5. Development Workflow

```bash
# Validate schema syntax
lumos validate schema.lumos

# Check if generated code is up-to-date
lumos check schema.lumos

# Watch for changes and auto-regenerate
lumos generate schema.lumos --watch
```

### 6. Iterate with Confidence

Update your `.lumos` schema, run `lumos generate`, and both codebases stay in sync automatically. No manual synchronization needed!

### 7. CI/CD Integration

Automate schema validation and code generation in your CI/CD pipeline with the [official GitHub Action](https://github.com/marketplace/actions/lumos-generate):

```yaml
- uses: actions/checkout@v4
- uses: getlumos/lumos-action@v1
  with:
    schema: 'schemas/**/*.lumos'
    fail-on-drift: true  # Ensure generated code is committed
```

**Features:**
- ✅ Auto-install LUMOS CLI
- ✅ Validate schemas on every PR
- ✅ Detect drift between schemas and generated code
- ✅ Post PR comments with diff summaries

[View full documentation →](https://github.com/getlumos/lumos-action#readme)

### 8. IDE Support

Get a powerful development experience with the LUMOS Language Server (LSP):

```bash
cargo install lumos-lsp
```

**Features:**
- 🔴 Real-time diagnostics (syntax errors, undefined types)
- ⚡ Auto-completion (Solana types, primitives, attributes)
- 📖 Hover information (type definitions, documentation)
- 🎨 Code formatting (coming soon)

**Supported Editors:**
- **VS Code**: Install [LUMOS extension](https://github.com/getlumos/vscode-lumos) (auto-configures LSP)
- **Neovim**: Use `nvim-lspconfig` with `lumos-lsp` binary
- **Emacs**: Configure `lsp-mode` with `lumos-lsp`
- **Sublime Text**: Install LSP package + configure `lumos-lsp`
- **IntelliJ/Rust Rover**: Coming soon

[View LSP documentation →](packages/lsp/README.md)

---

## 🏗️ Architecture

### Design Philosophy

LUMOS uses an **Intermediate Representation (IR)** architecture to decouple parsing from code generation. This enables:
- Language-agnostic schema representation
- Easy addition of new target languages
- Consistent transformations and optimizations
- Better testing and validation

### Pipeline Flow

```
┌─────────────────────────────────────────────────────────────┐
│                      LUMOS Pipeline                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  1. PARSER (syn-based)                                       │
│     Input: .lumos file                                       │
│     Output: AST (Abstract Syntax Tree)                       │
│     ├─ Attribute parsing (#[solana], #[account])            │
│     ├─ Struct definitions                                    │
│     ├─ Field types and annotations                           │
│     └─ Validation and error reporting                        │
│                                                               │
│  2. TRANSFORMER                                              │
│     Input: AST                                               │
│     Output: IR (Intermediate Representation)                 │
│     ├─ Type normalization                                    │
│     ├─ Semantic analysis                                     │
│     └─ Language-agnostic representation                      │
│                                                               │
│  3. CODE GENERATORS                                          │
│     Input: IR                                                │
│     Output: Target language code                             │
│                                                               │
│     ┌────────────────────┐      ┌────────────────────┐      │
│     │   Rust Generator   │      │TypeScript Generator│      │
│     ├────────────────────┤      ├────────────────────┤      │
│     │• Context detection │      │• Interface gen     │      │
│     │• Import management │      │• Borsh schema      │      │
│     │• Derive selection  │      │• Type mapping      │      │
│     │• Anchor support    │      │• SDK helpers       │      │
│     └────────────────────┘      └────────────────────┘      │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

| Component | Responsibility | Lines of Code |
|-----------|---------------|---------------|
| **parser.rs** | Parse `.lumos` syntax → AST | ~200 |
| **ast.rs** | AST data structures | ~150 |
| **transform.rs** | AST → IR transformation | ~180 |
| **ir.rs** | Intermediate representation | ~120 |
| **generators/rust.rs** | Rust code generation | ~340 |
| **generators/typescript.rs** | TypeScript code generation | ~387 |

### Design Decisions

#### 1. Rust-Style Syntax
**Why?** Solana developers primarily use Rust. Familiar `#[attribute]` syntax reduces learning curve and feels natural.

#### 2. Context-Aware Generation
**Why?** Mixed modules (some with `#[account]`, some without) require different imports. Smart detection prevents compile errors.

Example:
```rust
#[solana]
#[account]
struct Config { ... }  // Uses anchor_lang::prelude::*

#[solana]
struct Event { ... }   // Uses AnchorSerialize/AnchorDeserialize
```

#### 3. IR-Based Architecture
**Why?** Decouples parsing from generation. Adding Python support? Just write a new generator that consumes the IR.

#### 4. No Manual Derives for #[account]
**Why?** Anchor's `#[account]` macro already provides derives. Adding manual derives causes conflicts:
```rust
#[derive(BorshSerialize)] // ❌ CONFLICT!
#[account]                 // Already provides BorshSerialize
struct Foo { ... }
```

---

## 📋 Examples

LUMOS includes **5 real-world example schemas** covering common Solana use cases. All examples have been tested and generate valid, compilable code.

### 1. Gaming Platform
**File:** `examples/gaming/schema.lumos`

```rust
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    level: u16,
    experience: u64,
    equipped_items: [PublicKey],
}

#[solana]
#[account]
struct GameSession {
    players: [PublicKey],
    start_time: i64,
    active: bool,
}

#[solana]
struct MatchResult {
    player: PublicKey,
    opponent: Option<PublicKey>,
    score: u64,
}
```

**Use Case:** On-chain game state management with player progression, sessions, and match results.

### 2. NFT Marketplace
**File:** `examples/nft-marketplace/schema.lumos`

```rust
#[solana]
#[account]
struct Listing {
    nft_mint: PublicKey,
    seller: PublicKey,
    price: u64,
    active: bool,
}

#[solana]
struct PurchaseReceipt {
    buyer: PublicKey,
    nft_mint: PublicKey,
    price: u64,
    transaction_signature: Signature,
}
```

**Use Case:** NFT marketplace with listings and purchase tracking. Demonstrates `Signature` type mapping to `String` (base58).

### 3. DeFi Staking Protocol
**File:** `examples/defi-staking/schema.lumos`

```rust
#[solana]
#[account]
struct StakeAccount {
    owner: PublicKey,
    amount: u64,
    staked_at: i64,
    reward_rate: u16,
}

#[solana]
struct RewardClaim {
    staker: PublicKey,
    amount: u64,
    claimed_at: i64,
}
```

**Use Case:** Staking protocol with reward calculations and claim tracking.

### 4. DAO Governance
**File:** `examples/dao-governance/schema.lumos`

```rust
#[solana]
#[account]
struct Proposal {
    id: u64,
    proposer: PublicKey,
    description: String,
    votes_for: u64,
    votes_against: u64,
    deadline: i64,
    executed: bool,
}

#[solana]
struct Vote {
    voter: PublicKey,
    proposal_id: u64,
    vote_weight: u64,
    in_favor: bool,
}
```

**Use Case:** DAO governance system with proposals and voting. Shows `String` type support.

### 5. Token Vesting
**File:** `examples/token-vesting/schema.lumos`

```rust
#[solana]
#[account]
struct VestingSchedule {
    beneficiary: PublicKey,
    total_amount: u64,
    released_amount: u64,
    start_time: i64,
    cliff_duration: i64,
    vesting_duration: i64,
}

#[solana]
struct Release {
    beneficiary: PublicKey,
    amount: u64,
    released_at: i64,
}
```

**Use Case:** Token vesting with time-locked releases. Demonstrates complex time-based logic.

---

## 🔄 Type Mapping

LUMOS provides complete bidirectional type mapping between `.lumos` syntax, Rust, and TypeScript.

### Primitive Types

| LUMOS | Rust | TypeScript | Borsh (Rust) | Borsh (TS) |
|-------|------|------------|--------------|------------|
| `u8` | `u8` | `number` | - | `borsh.u8` |
| `u16` | `u16` | `number` | - | `borsh.u16` |
| `u32` | `u32` | `number` | - | `borsh.u32` |
| `u64` | `u64` | `number` | - | `borsh.u64` |
| `u128` | `u128` | `bigint` | - | `borsh.u128` |
| `i8` | `i8` | `number` | - | `borsh.i8` |
| `i16` | `i16` | `number` | - | `borsh.i16` |
| `i32` | `i32` | `number` | - | `borsh.i32` |
| `i64` | `i64` | `number` | - | `borsh.i64` |
| `i128` | `i128` | `bigint` | - | `borsh.i128` |
| `bool` | `bool` | `boolean` | - | `borsh.bool` |

### Solana-Specific Types

| LUMOS | Rust | TypeScript | Borsh (TS) |
|-------|------|------------|------------|
| `PublicKey` | `Pubkey` | `PublicKey` | `borsh.publicKey` |
| `Signature` | `String` | `string` | `borsh.string` |

### Complex Types

| LUMOS | Rust | TypeScript | Borsh (TS) |
|-------|------|------------|------------|
| `String` | `String` | `string` | `borsh.string` |
| `[T]` | `Vec<T>` | `T[]` | `borsh.vec(...)` |
| `Option<T>` | `Option<T>` | `T \| undefined` | `borsh.option(...)` |

### Type Mapping Examples

```rust
#[solana]
struct Example {
    id: u64,                      // → Rust: u64, TS: number
    wallet: PublicKey,            // → Rust: Pubkey, TS: PublicKey
    name: String,                 // → Rust: String, TS: string
    tags: [String],               // → Rust: Vec<String>, TS: string[]
    metadata: Option<String>,     // → Rust: Option<String>, TS: string | undefined
    large_number: u128,           // → Rust: u128, TS: bigint
}
```

**Generated TypeScript Borsh Schema:**
```typescript
export const ExampleBorshSchema = borsh.struct([
  borsh.u64('id'),
  borsh.publicKey('wallet'),
  borsh.string('name'),
  borsh.vec(borsh.string(), 'tags'),
  borsh.option(borsh.string(), 'metadata'),
  borsh.u128('large_number'),
]);
```

---

## 🚀 Roadmap

> 📍 **Looking for our future plans?** See the detailed [ROADMAP.md](ROADMAP.md) for Phase 4+, including VSCode extension polish, community examples, and ecosystem expansion.

> 🔮 **Curious about our long-term vision?** Check out [docs/VISION.md](docs/VISION.md) - LUMOS is evolving from a schema DSL into a full typed workflow programming language for developer automation.

### Phase 1: Core TypeScript ↔ Rust Codegen ✅ **COMPLETED**

**Status:** 🎉 **100% Complete (2025-01-17)**

- ✅ Project setup and architecture
- ✅ Custom `.lumos` parser using syn
- ✅ Rust code generator with Borsh serialization
- ✅ TypeScript code generator with Borsh schemas
- ✅ Context-aware generation (Anchor/Borsh detection)
- ✅ Smart derive management
- ✅ Complete type mapping system
- ✅ 50/50 tests passing (100% success rate)
- ✅ E2E compilation tests
- ✅ 5 real-world examples

**Metrics:**
- **50 tests** (26 unit + 24 integration/E2E)
- **100% pass rate**
- **5 example schemas** all compile successfully
- **1,400+ lines** of core generation logic

---

### Phase 2: CLI & Developer Tools ✅ **COMPLETED**

**Status:** 🎉 **100% Complete (2025-01-17)**

Core CLI functionality to make LUMOS usable in real projects:

- ✅ **File I/O System**
  - Read `.lumos` files from disk
  - Write generated code to filesystem
  - Output directory customization

- ✅ **CLI Tool (`lumos` command)**
  - `lumos init [project]` - Initialize new project with templates
  - `lumos generate <schema>` - Generate Rust + TypeScript code
  - `lumos validate <schema>` - Validate schema syntax
  - `lumos check <schema>` - Verify generated code is up-to-date
  - `lumos diff <schema1> <schema2>` - Compare schemas and show differences
  - `lumos generate --watch` - Watch mode for auto-regeneration
  - `lumos --version` - Version information
  - `lumos --help` - Comprehensive help

- ✅ **Configuration System**
  - `lumos.toml` configuration file
  - Output directory customization
  - Project initialization templates

- ✅ **Developer Experience**
  - Professional cargo-style colored output
  - Clear status messages and progress indicators
  - Helpful error messages with context
  - File watching with debouncing

**Metrics:**
- ✅ Working `lumos` CLI executable
- ✅ Can generate real Solana projects from scratch
- ✅ All 4 CLI commands fully functional
- ✅ Tested with 5 real-world example schemas
- ✅ Watch mode with file system monitoring

**Success Criteria:**
- ✅ Working `lumos` CLI executable
- ✅ Can generate real Solana projects from scratch
- ⏳ Published to crates.io (pending)
- ⏳ Documentation website live (pending)

---

### Phase 3.1: Enum Support ✅ **COMPLETED**

**Status:** 🎉 **100% Complete (2025-11-17)**

Full support for Rust-style enums with three variant types:

- ✅ **Unit Variants** - Simple state machines (`Active`, `Paused`, `Finished`)
- ✅ **Tuple Variants** - Data-carrying variants (`PlayerJoined(PublicKey, u64)`)
- ✅ **Struct Variants** - Named fields (`Initialize { authority: PublicKey }`)

**Implementation:**

- ✅ **AST & Parser** (Week 1)
  - Complete enum syntax design with 8 comprehensive patterns
  - AST support for all 3 enum variant types
  - Full parser implementation
  - 5 new parser tests passing
  - 500+ lines design documentation

- ✅ **IR & Transform** (Week 2)
  - Enum-based TypeDefinition IR architecture
  - EnumDefinition and EnumVariantDefinition types
  - Complete AST→IR transform for all variants
  - 3 new transform tests passing
  - All generators updated for new IR structure

- ✅ **Code Generation** (Week 3)
  - Rust native enum generator with context-aware derives
  - TypeScript discriminated unions with `kind` field
  - Borsh schema support for enums
  - Enum-specific unit tests
  - E2E compilation tests with enums

- ✅ **Documentation & Polish** (Week 4)
  - Real-world Solana instruction pattern validation
  - Performance optimization
  - Complete documentation updates
  - Example schemas with enum patterns

**Metrics:**
- ✅ **64/64 tests passing** (100% success rate)
- ✅ All 3 enum variant types supported
- ✅ E2E compilation tests pass
- ✅ Context-aware derives working
- ✅ TypeScript discriminated unions with type safety

**Example:**

```rust
#[solana]
enum GameInstruction {
    Initialize {
        authority: PublicKey,
        max_players: u32,
    },
    UpdateScore {
        player: PublicKey,
        new_score: u64,
    },
}
```

---

### Phase 3.2: VSCode Extension ✅ **COMPLETED**

**Status:** 🎉 **100% Complete (2025-11-18)**

Professional VSCode extension for enhanced `.lumos` development experience:

- ✅ **Syntax Highlighting**
  - TextMate grammar with 26 rules
  - Support for attributes, keywords, types, strings, numbers, comments
  - Solana-specific type highlighting (`PublicKey`, `Signature`)
  - Context-aware highlighting for `#[solana]`, `#[account]`, enums

- ✅ **Code Snippets** (13 snippets)
  - `struct` - Basic struct template
  - `account` - Solana account struct
  - `enum-unit` - Unit variant enum
  - `enum-tuple` - Tuple variant enum
  - `enum-struct` - Struct variant enum
  - `enum-mixed` - Mixed variant enum
  - Field type snippets (`pubkey`, `u64`, `string`, `vec`, `option`)

- ✅ **Commands**
  - `LUMOS: Generate Code` - Generate Rust + TypeScript from current file
  - `LUMOS: Validate Schema` - Validate current schema syntax

- ✅ **Auto-Generation**
  - Generate code on save (configurable)
  - Setting: `lumos.autoGenerateOnSave`

- ✅ **Professional Branding**
  - "Radiant Precision" icon design (inspired by Frieren's magic circles)
  - Purple and gold color scheme
  - Geometric patterns symbolizing code generation

**Metrics:**
- ✅ Extension package: 17.77 KB
- ✅ 26 syntax highlighting rules
- ✅ 13 productivity snippets
- ✅ 2 commands + auto-generation
- ✅ Professional icon and branding
- ✅ Ready for VSCode marketplace publishing

**Repository:** [getlumos/vscode-lumos](https://github.com/getlumos/vscode-lumos)

---

### Phase 3.3: Advanced Features 📋 **FUTURE** (Months 7-12)

Powerful features for complex use cases:

- [ ] **PDA (Program Derived Address) Helpers**
  - `#[pda]` attribute support
  - Seed derivation generation
  - TypeScript PDA finding helpers

- [ ] **Anchor Instruction Generation**
  - Generate instruction handlers
  - CPI helper functions
  - Account validation macros

- [ ] **Validation & Constraints**
  - `#[validate]` attributes
  - Range constraints (`min`, `max`)
  - Custom validation functions

- [ ] **Migration Tools**
  - Version compatibility checker
  - Schema diff tool
  - Breaking change detector
  - Migration script generator

**Success Criteria:**
- [ ] PDA generation tested with Anchor
- [ ] Migration tools handle v1 → v2 schemas
- [ ] Validation constraints in 10+ test cases

---

### Phase 4: Ecosystem Expansion 🌍 **VISION** (Year 2+)

**Multi-Language Support:**
- [ ] **TypeScript ↔ C++** generator
- [ ] **TypeScript ↔ Python** generator
- [ ] **TypeScript ↔ Go** generator

**Plugin Architecture:**
- [ ] Community generator SDK
- [ ] Custom transformer plugins
- [ ] Template marketplace

**Advanced Solana Features:**
- [ ] Zero-Knowledge proof type support
- [ ] ZK circuit generation helpers
- [ ] Integration with ZK libraries (Light Protocol, etc.)

**Tooling:**
- [ ] IntelliJ IDEA plugin
- [ ] Neovim/Vim plugin
- [ ] Language server protocol (LSP) support

---

## 🛠️ Tech Stack

### Core Technologies

| Technology | Purpose | Version |
|------------|---------|---------|
| **Rust** | Core language | 1.70+ |
| **syn** | Rust parser | 2.0 |
| **quote** | Code generation | 1.0 |
| **proc-macro2** | Token manipulation | 1.0 |
| **serde** | Serialization | 1.0 |
| **serde_json** | JSON support | 1.0 |
| **toml** | Config files | 0.8 |
| **anyhow** | Error handling | 1.0 |
| **thiserror** | Error macros | 1.0 |

### Development Tools

| Tool | Purpose |
|------|---------|
| **cargo** | Build system & package manager |
| **cargo test** | Test runner |
| **rustfmt** | Code formatting |
| **clippy** | Linting |
| **tempfile** | E2E test infrastructure |

### Dependencies (packages/core/Cargo.toml)

```toml
[dependencies]
syn = "2.0"           # Rust parser for .lumos syntax
quote = "1.0"         # Quasi-quoting for code generation
proc-macro2 = "1.0"   # Token stream manipulation
serde = "1.0"         # Serialization framework
serde_json = "1.0"    # JSON support
toml = "0.8"          # TOML config parsing
anyhow = "1.0"        # Flexible error handling
thiserror = "1.0"     # Derive macro for error types

[dev-dependencies]
tempfile = "3.8"      # Temporary file creation for E2E tests
```

---

## ✅ Test Suite

LUMOS has comprehensive test coverage ensuring code quality and reliability.

### Test Statistics

**Total Tests:** 50/50 passing (100% success rate)

| Test Category | Count | Purpose |
|---------------|-------|---------|
| **Unit Tests** | 26 | Core functionality (parser, generators, transform) |
| **Parser Integration** | 5 | Real-world schema parsing |
| **Rust Generator Integration** | 5 | Rust code generation validation |
| **TypeScript Generator Integration** | 6 | TypeScript code generation validation |
| **E2E Compilation** | 8 | Actual Rust compilation with `cargo check` |

### Running Tests

```bash
cd packages/core

# Run all tests
cargo test

# Run specific test suites
cargo test --lib                  # Unit tests only
cargo test --test integration_test    # Parser integration
cargo test --test test_e2e            # E2E compilation tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_gaming_example
```

### Test Coverage

**Parser Tests:**
- ✅ Parse structs with `#[solana]` and `#[account]` attributes
- ✅ Parse all primitive types (u8, u16, u32, u64, u128, i8-i128, bool)
- ✅ Parse Solana types (PublicKey, Signature)
- ✅ Parse complex types (String, Vec, Option)
- ✅ Parse all 5 example schemas

**Generator Tests:**
- ✅ Rust: Context-aware import generation
- ✅ Rust: Smart derive selection
- ✅ Rust: Anchor `#[account]` handling
- ✅ Rust: Mixed module support
- ✅ TypeScript: Interface generation
- ✅ TypeScript: Borsh schema generation
- ✅ TypeScript: Type mapping correctness

**E2E Tests:**
- ✅ Generated Rust code compiles with `cargo check`
- ✅ All 5 examples compile successfully
- ✅ No import conflicts
- ✅ No derive conflicts
- ✅ Proper Anchor integration

### Example Test Output

```
running 50 tests
test ast::tests::test_struct_creation ... ok
test parser::tests::test_parse_basic_struct ... ok
test parser::tests::test_parse_with_attributes ... ok
test generators::rust::tests::test_context_detection ... ok
test generators::rust::tests::test_derive_selection ... ok
test generators::typescript::tests::test_interface_gen ... ok
test generators::typescript::tests::test_borsh_schema ... ok
test integration_test::test_parse_gaming_example ... ok
test integration_test::test_parse_nft_marketplace ... ok
test test_e2e::test_gaming_example_compiles ... ok
test test_e2e::test_dao_governance_compiles ... ok

test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured
```

---

## 🤝 Contributing

LUMOS is in active early development and we welcome contributions from the community!

### How to Contribute

1. **Fork the Repository**
   ```bash
   git fork https://github.com/getlumos/lumos.git
   ```

2. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Your Changes**
   - Follow existing code style (use `rustfmt`)
   - Add tests for new functionality
   - Update documentation as needed

4. **Run Tests**
   ```bash
   cd packages/core
   cargo test
   cargo fmt --check
   cargo clippy
   ```

5. **Submit a Pull Request**
   - Provide clear description of changes
   - Reference any related issues
   - Ensure all tests pass

### Development Setup

```bash
# Clone the repository
git clone https://github.com/getlumos/lumos.git
cd lumos

# Build the project
cd packages/core
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Areas We Need Help

- 🐛 **Bug Reports** - Found an issue? Open a GitHub issue
- 📝 **Documentation** - Improve guides, examples, and API docs
- ✨ **Features** - Implement items from the roadmap
- 🧪 **Testing** - Add more test cases and edge case coverage
- 🎨 **Examples** - Create more real-world example schemas
- 🌍 **Community** - Share LUMOS with the Solana community

### Contributing Guidelines

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guidelines on:
- Code style and conventions
- Commit message format
- Pull request process
- Issue reporting
- Community conduct

---

## 📄 License

LUMOS is dual-licensed under your choice of:

- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

This follows the same licensing model as the Rust programming language.

### Why Dual License?

- **Flexibility** - Use the license that works best for your project
- **Compatibility** - MIT for maximum permissiveness, Apache 2.0 for patent protection
- **Rust Ecosystem Standard** - Consistent with Rust community conventions

You may choose either license when using LUMOS in your projects.

---

## 🙏 Credits

### Created By

**RECTOR** ([@rz1989s](https://github.com/rz1989s))
Senior Developer & Founder of [getlumos](https://github.com/getlumos)

### Organization

Built with dedication at **getlumos** - Empowering developers with innovative tools.

### Acknowledgments

- **Solana Foundation** - For building an incredible blockchain ecosystem
- **Anchor Team** - For the excellent Solana development framework
- **Rust Community** - For `syn`, `quote`, and amazing tooling
- **Borsh Team** - For the efficient serialization format

### Built For

The **Solana developer community** - developers building the future of decentralized applications.

---

## 🌐 Resources

### Official Documentation

- **LUMOS Docs** - Coming soon
- **Vision (Vertical)** - [docs/VISION.md](docs/VISION.md) - ENDGAME: Workflow language for Solana
- **Future (Horizontal)** - [docs/FUTURE.md](docs/FUTURE.md) - Beyond: Multichain, DevOps, automation
- **Roadmap** - [ROADMAP.md](ROADMAP.md) - Development phases and timeline
- **Migration Guide** - [docs/MIGRATION.md](docs/MIGRATION.md) - Version upgrade instructions
- **Execution Plan** - [docs/execution-plan.md](docs/execution-plan.md)
- **Project Context** - [CLAUDE.md](CLAUDE.md)

### Solana Ecosystem

- **Solana Docs** - https://docs.solana.com/
- **Anchor Framework** - https://www.anchor-lang.com/
- **Borsh Specification** - https://borsh.io/

### Rust Resources

- **syn Crate** - https://docs.rs/syn/
- **quote Crate** - https://docs.rs/quote/
- **Rust Book** - https://doc.rust-lang.org/book/

---

<div align="center">

**Status:** ✅ Published on crates.io - Production Ready
**Version:** 0.1.0
**Released:** November 18, 2025

---

**Built with ❤️ for the Solana community**

⭐ **Star this repo** if you find LUMOS useful!

[Report Bug](https://github.com/getlumos/lumos/issues) • [Request Feature](https://github.com/getlumos/lumos/issues) • [Discussions](https://github.com/getlumos/lumos/discussions)

</div>
