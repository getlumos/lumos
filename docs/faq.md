# Frequently Asked Questions

> **Quick answers to common LUMOS questions**

Find answers to the most frequently asked questions about LUMOS, the type-safe schema language for Solana development.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Schema Design](#schema-design)
3. [Code Generation](#code-generation)
4. [Anchor Integration](#anchor-integration)
5. [TypeScript & Frontend](#typescript--frontend)
6. [Migration](#migration)
7. [Troubleshooting](#troubleshooting)
8. [CLI Reference](#cli-reference)

---

## Getting Started

### How do I install LUMOS?

**Option 1: Cargo (Recommended)**
```bash
cargo install lumos-cli
lumos --version
```

**Option 2: npm (No Rust required)**
```bash
npm install -g @getlumos/cli
lumos --version
```

**Option 3: From source**
```bash
git clone https://github.com/getlumos/lumos
cd lumos
cargo install --path packages/cli
```

---

### Do I need Rust installed to use LUMOS?

**No!** You have two options:

1. **npm package** - Install via `npm install -g @getlumos/cli` (uses WebAssembly, no Rust needed)
2. **Docker** - Use the official Docker image for CI/CD pipelines

However, if you're building Anchor programs, you'll need Rust for `anchor build`.

---

### What's the quickest way to try LUMOS?

```bash
# 1. Install
cargo install lumos-cli

# 2. Create a sample schema
cat > schema.lumos << 'EOF'
#[solana]
#[account]
struct Counter {
    authority: PublicKey,
    count: u64,
}
EOF

# 3. Generate code
lumos generate schema.lumos

# 4. Check output
cat generated.rs
cat generated.ts
```

---

### Which editors have LUMOS support?

| Editor | Extension | Features |
|--------|-----------|----------|
| **VSCode** | [vscode-lumos](https://marketplace.visualstudio.com/items?itemName=getlumos.vscode-lumos) | Syntax, IntelliSense, diagnostics |
| **Neovim** | [nvim-lumos](https://github.com/getlumos/nvim-lumos) | Tree-sitter, LSP |
| **IntelliJ/Rust Rover** | [intellij-lumos](https://github.com/getlumos/intellij-lumos) | LSP integration |
| **Emacs** | [lumos-mode](https://github.com/getlumos/lumos-mode) | Syntax, LSP |
| **Sublime Text** | [sublime-lumos](https://github.com/getlumos/sublime-lumos) | Syntax, snippets |

All editors use the same LSP server (`lumos-lsp`) for consistent features.

---

### What files does LUMOS generate?

By default, LUMOS generates:

| File | Contents |
|------|----------|
| `generated.rs` | Rust structs with Borsh/Anchor derives |
| `generated.ts` | TypeScript interfaces + Borsh schemas |

With `--lang` flag, you can also generate:
- Python (`generated.py`)
- Go (`generated.go`)
- Ruby (`generated.rb`)

---

### How is LUMOS different from Anchor's IDL?

| Feature | Anchor IDL | LUMOS |
|---------|------------|-------|
| Source of truth | Generated from Rust | Schema file |
| TypeScript types | Manual or generated | Generated with precision warnings |
| Space calculation | Manual | Automatic (`lumos anchor space`) |
| Multi-language | TypeScript only | Rust, TS, Python, Go, Ruby |
| Validation | Runtime | Compile-time |

LUMOS can generate Anchor IDL: `lumos anchor idl schema.lumos`

---

### Can I use LUMOS without Anchor?

**Yes!** LUMOS supports pure Borsh serialization:

```lumos
#[solana]
struct MyData {
    name: String,
    value: u64,
}
```

Without `#[account]`, LUMOS generates:
```rust
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MyData {
    pub name: String,
    pub value: u64,
}
```

---

### Where should I put my schema file?

Recommended project structure:

```
my-project/
├── schemas/
│   └── schema.lumos      # Your schema
├── programs/
│   └── my_program/
│       └── src/
│           └── lib.rs    # Generated + handlers
├── app/
│   └── src/
│       └── types.ts      # Generated TypeScript
└── Anchor.toml
```

---

## Schema Design

### What's the difference between `#[account]` and regular structs?

**`#[account]` structs** are on-chain Solana accounts:

```lumos
#[solana]
#[account]
struct Vault {
    owner: PublicKey,
    balance: u64,
}
```

Generates:
```rust
use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub balance: u64,
}

impl Vault {
    pub const LEN: usize = 8 + 32 + 8; // Auto-calculated!
}
```

**Regular structs** are data types (not stored on-chain directly):

```lumos
#[solana]
struct TransferParams {
    amount: u64,
    memo: String,
}
```

Generates:
```rust
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct TransferParams {
    pub amount: u64,
    pub memo: String,
}
```

---

### What types does LUMOS support?

**Primitive Types:**

| LUMOS | Rust | TypeScript | Size |
|-------|------|------------|------|
| `u8`, `i8` | `u8`, `i8` | `number` | 1 byte |
| `u16`, `i16` | `u16`, `i16` | `number` | 2 bytes |
| `u32`, `i32` | `u32`, `i32` | `number` | 4 bytes |
| `u64`, `i64` | `u64`, `i64` | `number` ⚠️ | 8 bytes |
| `u128`, `i128` | `u128`, `i128` | `bigint` | 16 bytes |
| `bool` | `bool` | `boolean` | 1 byte |
| `String` | `String` | `string` | 4 + len |

**Solana Types:**

| LUMOS | Rust | TypeScript | Size |
|-------|------|------------|------|
| `PublicKey` | `Pubkey` | `PublicKey` | 32 bytes |
| `Signature` | `Signature` | `Uint8Array` | 64 bytes |

**Complex Types:**

| LUMOS | Rust | TypeScript |
|-------|------|------------|
| `[T]` | `Vec<T>` | `T[]` |
| `[T; N]` | `[T; N]` | `T[]` (length N) |
| `Option<T>` | `Option<T>` | `T \| null` |

---

### How do I handle u64 values in TypeScript?

JavaScript's `number` type loses precision above 2^53-1 (9,007,199,254,740,991).

**LUMOS auto-generates warnings:**

```typescript
export interface Vault {
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1.
   * For Solana lamports or large values, ensure they stay within safe range.
   */
  balance: number;
}
```

**Solutions:**

1. **Use u128 for large values** (maps to `bigint`):
   ```lumos
   struct Vault {
       balance: u128,  // → bigint in TypeScript
   }
   ```

2. **Check range before arithmetic:**
   ```typescript
   if (balance > Number.MAX_SAFE_INTEGER) {
       throw new Error('Value exceeds safe range');
   }
   ```

---

### How do I create enums?

LUMOS supports three enum variant types:

**Unit variants** (no data):
```lumos
#[solana]
enum Status {
    Active,
    Paused,
    Closed,
}
```

**Tuple variants** (positional data):
```lumos
#[solana]
enum Event {
    Transfer(PublicKey, u64),
    Mint(u64),
}
```

**Struct variants** (named fields):
```lumos
#[solana]
enum GameResult {
    Victory { score: u64, time: i64 },
    Defeat { score: u64 },
    Draw,
}
```

**TypeScript generates discriminated unions:**
```typescript
type GameResult =
  | { kind: 'Victory'; score: number; time: number }
  | { kind: 'Defeat'; score: number }
  | { kind: 'Draw' };

// Type narrowing works!
if (result.kind === 'Victory') {
    console.log(result.score); // TypeScript knows score exists
}
```

---

### How do I set string/array length limits?

Use the `#[max(N)]` attribute:

```lumos
#[solana]
#[account]
struct Profile {
    #[max(32)]
    username: String,      // Max 32 characters

    #[max(100)]
    bio: String,           // Max 100 characters

    #[max(10)]
    friends: [PublicKey],  // Max 10 friends
}
```

**Space calculation includes limits:**
```
username: 4 (length prefix) + 32 (max chars) = 36 bytes
bio: 4 + 100 = 104 bytes
friends: 4 (length prefix) + 32 * 10 = 324 bytes
```

---

### How do I deprecate a field?

Use the `#[deprecated]` attribute:

```lumos
#[solana]
#[account]
struct User {
    owner: PublicKey,

    #[deprecated("Use email_v2 instead")]
    email: String,

    email_v2: Option<String>,
}
```

**Output during generation:**
```
warning: User.email: Use email_v2 instead
```

---

### How do I mark PDA seed fields?

Use the `#[key]` attribute to document PDA seeds:

```lumos
#[solana]
#[account]
struct PlayerVault {
    #[key]  // Used in PDA: seeds = [b"vault", player.as_ref()]
    player: PublicKey,

    #[key]  // Used in PDA: seeds = [..., game_id.to_le_bytes()]
    game_id: u64,

    balance: u64,
}
```

This helps tooling understand account relationships and assists with client-side PDA derivation.

---

### Can I reference other types in my schema?

**Yes!** Types can reference each other:

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
    stats: Stats,        // Nested struct
    class: CharacterClass,  // Enum reference
}

#[solana]
enum CharacterClass {
    Warrior,
    Mage,
    Archer,
}
```

**Note:** All referenced types must be defined in the same schema (or imported via modules).

---

### Can I use generic types?

LUMOS doesn't support user-defined generics, but provides built-in generic types:

```lumos
// ✅ Supported
items: [PublicKey],      // Vec<PublicKey>
maybe_value: Option<u64>, // Option<u64>
fixed_data: [u8; 32],    // [u8; 32]

// ❌ Not supported
data: MyGeneric<T>,      // Custom generics
```

**Workaround:** Create concrete types:
```lumos
// Instead of Result<T, E>
#[solana]
enum TransferResult {
    Success { amount: u64 },
    Error { code: u32 },
}
```

---

## Code Generation

### How do I generate code?

**Basic generation:**
```bash
lumos generate schema.lumos
```

**With options:**
```bash
# Specify output directory
lumos generate schema.lumos -o src/generated/

# Generate specific languages
lumos generate schema.lumos --lang rust,typescript

# Watch mode (auto-regenerate on save)
lumos generate schema.lumos --watch

# Preview without writing files
lumos generate schema.lumos --dry-run
```

---

### Will regenerating overwrite my code?

**Yes!** Generated files are completely overwritten.

**Best practice:** Keep business logic separate:

```
src/
├── generated.rs    # ← Overwritten by LUMOS (don't edit!)
├── handlers.rs     # ← Your code (safe)
└── lib.rs          # ← Imports both
```

```rust
// lib.rs
mod generated;
mod handlers;

pub use generated::*;
```

---

### How do I preview changes before generating?

```bash
# Show what would be generated
lumos generate schema.lumos --dry-run

# Show diff against existing files
lumos generate schema.lumos --dry-run --show-diff

# Backup existing files before overwriting
lumos generate schema.lumos --backup
```

---

### How do I use watch mode?

```bash
# Basic watch
lumos generate schema.lumos --watch

# With verbose logging
lumos generate schema.lumos --watch --verbose

# Custom debounce (milliseconds)
lumos generate schema.lumos --watch --debounce 500
```

Watch mode automatically regenerates when you save the schema file.

---

### How do I generate multiple languages?

```bash
# Generate Rust and TypeScript (default)
lumos generate schema.lumos

# Generate specific languages
lumos generate schema.lumos --lang rust,typescript,python

# Generate all supported languages
lumos generate schema.lumos --lang rust,typescript,python,go,ruby
```

---

### How do I integrate LUMOS in CI/CD?

**GitHub Actions:**
```yaml
name: Validate Schema
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Validate and Generate
        uses: getlumos/lumos-action@v1
        with:
          schema: 'schemas/**/*.lumos'

      - name: Check for drift
        run: git diff --exit-code
```

**Manual CI:**
```bash
# Install
cargo install lumos-cli

# Validate
lumos validate schema.lumos

# Generate and check for uncommitted changes
lumos generate schema.lumos
git diff --exit-code
```

---

### How do I check if generated code is up-to-date?

```bash
# Check if regeneration would change anything
lumos check schema.lumos

# Returns exit code 0 if up-to-date, 1 if changes needed
```

Useful in CI to catch uncommitted generation changes.

---

### Can I split my schema across multiple files?

**Yes!** Use imports:

```lumos
// types/player.lumos
#[solana]
#[account]
struct Player {
    owner: PublicKey,
    stats: Stats,
}

// types/stats.lumos
#[solana]
struct Stats {
    level: u16,
    experience: u64,
}

// main.lumos
import "./types/player.lumos";
import "./types/stats.lumos";
```

---

## Anchor Integration

### How do I generate a complete Anchor program?

```bash
lumos anchor generate schema.lumos \
    --name my_program \
    --address YOUR_PROGRAM_ID \
    --typescript \
    --output ./
```

**Generates:**
```
./
├── programs/my_program/src/lib.rs  # Complete Anchor program
├── target/idl/my_program.json      # Anchor IDL
└── app/src/types.ts                # TypeScript types
```

---

### How do I calculate account space?

```bash
# Human-readable output
lumos anchor space schema.lumos

# Output:
# Vault: 49 bytes (8 discriminator + 41 data)
#   - owner: 32 bytes (PublicKey)
#   - balance: 8 bytes (u64)
#   - bump: 1 byte (u8)
```

```bash
# Rust constants (copy-paste ready)
lumos anchor space schema.lumos --format rust

# Output:
# impl Vault {
#     pub const LEN: usize = 49;
# }
```

---

### Why is my code using Anchor imports instead of Borsh?

LUMOS uses **context-aware generation**:

- If **any** struct has `#[account]` → entire module uses Anchor imports
- If **no** structs have `#[account]` → uses Borsh imports

```lumos
// This schema...
#[solana]
#[account]
struct Vault { ... }

#[solana]
struct Params { ... }  // No #[account], but still gets Anchor imports
```

```rust
// Generates Anchor imports for BOTH types
use anchor_lang::prelude::*;

#[account]
pub struct Vault { ... }

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Params { ... }
```

---

### How do I generate Anchor IDL?

```bash
# Generate IDL only
lumos anchor idl schema.lumos \
    --output target/idl/my_program.json \
    --address YOUR_PROGRAM_ID \
    --pretty
```

---

### How do I use instruction contexts?

Mark instruction contexts with `#[instruction]`:

```lumos
#[solana]
#[instruction]
struct Initialize {
    #[anchor(init, payer = owner, space = 8 + 64, seeds = [b"vault"], bump)]
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
```

**Generates:**
```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = owner, space = 8 + 64, seeds = [b"vault"], bump)]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

---

### What Anchor attributes are supported?

| Attribute | Example |
|-----------|---------|
| `init` | `#[anchor(init)]` |
| `mut` | `#[anchor(mut)]` |
| `payer` | `#[anchor(payer = user)]` |
| `space` | `#[anchor(space = 8 + 64)]` |
| `seeds` | `#[anchor(seeds = [b"vault", user.key().as_ref()])]` |
| `bump` | `#[anchor(bump)]` or `#[anchor(bump = vault.bump)]` |
| `has_one` | `#[anchor(has_one = owner)]` |
| `constraint` | `#[anchor(constraint = "amount > 0")]` |
| `close` | `#[anchor(close = owner)]` |

See [Anchor Integration Guide](/guides/anchor-integration) for full reference.

---

### How do I handle PDAs in instruction contexts?

```lumos
#[solana]
#[instruction]
struct Deposit {
    #[anchor(
        mut,
        seeds = [b"vault", owner.key().as_ref()],
        bump = vault.bump
    )]
    vault: Vault,

    #[anchor(mut)]
    owner: Signer,

    system_program: SystemProgram,
}
```

---

## TypeScript & Frontend

### How do I deserialize on-chain accounts?

```typescript
import { Connection, PublicKey } from '@solana/web3.js';
import { VaultSchema } from './generated';
import * as borsh from '@coral-xyz/borsh';

const connection = new Connection('https://api.devnet.solana.com');
const vaultPubkey = new PublicKey('...');

// Fetch account
const accountInfo = await connection.getAccountInfo(vaultPubkey);

// Skip 8-byte Anchor discriminator
const data = accountInfo.data.slice(8);

// Deserialize
const vault = VaultSchema.decode(data);
console.log(vault.balance);
```

---

### How do I handle enum variants in TypeScript?

LUMOS generates **discriminated unions** for type-safe enums:

```typescript
// Generated type
type GameState =
  | { kind: 'Waiting' }
  | { kind: 'Active'; players: number }
  | { kind: 'Finished'; winner: PublicKey };

// Type narrowing works!
function handleState(state: GameState) {
    switch (state.kind) {
        case 'Waiting':
            console.log('Waiting for players...');
            break;
        case 'Active':
            console.log(`${state.players} players`);
            break;
        case 'Finished':
            console.log(`Winner: ${state.winner}`);
            break;
    }
}
```

---

### What's the Anchor discriminator?

The first 8 bytes of every Anchor account, derived from the account name:

```typescript
// Skip discriminator when deserializing
const data = accountInfo.data.slice(8);
const account = Schema.decode(data);
```

**Why it exists:** Anchor uses the discriminator to identify account types and prevent type confusion attacks.

---

### How do I build transactions with generated types?

```typescript
import { Program, AnchorProvider } from '@coral-xyz/anchor';
import { PublicKey, SystemProgram } from '@solana/web3.js';

// Using Anchor
const program = new Program(idl, programId, provider);

await program.methods
    .initialize()
    .accounts({
        vault: vaultPda,
        owner: wallet.publicKey,
        systemProgram: SystemProgram.programId,
    })
    .rpc();
```

---

### How do I use generated types with React?

```typescript
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { Vault, VaultSchema } from './generated';

function useVault(pubkey: PublicKey): Vault | null {
    const { connection } = useConnection();
    const [vault, setVault] = useState<Vault | null>(null);

    useEffect(() => {
        connection.getAccountInfo(pubkey).then((info) => {
            if (info) {
                const data = info.data.slice(8);
                setVault(VaultSchema.decode(data));
            }
        });
    }, [pubkey]);

    return vault;
}
```

See [web3.js Integration Guide](/guides/web3js-integration) for complete patterns.

---

### How do I subscribe to account changes?

```typescript
const subscriptionId = connection.onAccountChange(
    vaultPubkey,
    (accountInfo) => {
        const data = accountInfo.data.slice(8);
        const vault = VaultSchema.decode(data);
        console.log('Vault updated:', vault.balance);
    }
);

// Later: unsubscribe
connection.removeAccountChangeListener(subscriptionId);
```

---

## Migration

### How do I add LUMOS to an existing Anchor project?

**Three strategies:**

| Strategy | Risk | Effort | Best For |
|----------|------|--------|----------|
| **IDL-Only** | Low | 1 hour | Stable programs |
| **Gradual** | Medium | 1-2 days | Active development |
| **Full** | Higher | 2-3 days | Major rewrites |

**IDL-Only (Recommended start):**
```bash
# 1. Create schema matching existing structs
# 2. Generate IDL only
lumos anchor idl schema.lumos -o target/idl/program.json

# 3. Verify it matches
diff target/idl/program.json target/idl/program.json.bak
```

See [Migration: Anchor → LUMOS](/guides/migration-anchor) for detailed guide.

---

### How do I migrate from manual TypeScript types?

```bash
# Before: 67 lines across 3 files
# After: 15 lines in schema.lumos
```

**Steps:**
1. Audit existing interfaces and schemas
2. Create `schema.lumos` with matching types
3. Generate: `lumos generate schema.lumos`
4. Update imports in your code
5. Delete manual type files

See [Migration: TypeScript → LUMOS](/guides/migration-typescript) for complete guide.

---

### Will schema changes break on-chain data?

**Depends on the change:**

| Change | Safe? | Notes |
|--------|-------|-------|
| Add field at end | ✅ | Old data still deserializes |
| Add enum variant at end | ✅ | Old data still deserializes |
| Change field type (same size) | ⚠️ | May work, test carefully |
| Reorder fields | ❌ | Breaks serialization |
| Remove field | ❌ | Breaks serialization |
| Change field type (different size) | ❌ | Breaks serialization |

---

### How do I verify on-chain compatibility?

```bash
# Compare schemas
lumos check-compat old-schema.lumos new-schema.lumos

# Test deserialization
lumos test schema.lumos --account PUBKEY --network devnet
```

**Manual verification:**
```typescript
// Fetch existing account
const info = await connection.getAccountInfo(existingAccount);
// Deserialize with new schema
const account = NewSchema.decode(info.data.slice(8));
// Verify all fields populated correctly
```

---

### How do I handle breaking changes?

**Option 1: New account type**
```lumos
#[solana]
#[account]
struct VaultV2 {
    // New structure
}
```

**Option 2: Migration instruction**
```rust
pub fn migrate_vault(ctx: Context<MigrateVault>) -> Result<()> {
    let old = &ctx.accounts.old_vault;
    let new = &mut ctx.accounts.new_vault;

    new.owner = old.owner;
    new.balance = old.balance;
    new.new_field = default_value;

    Ok(())
}
```

---

## Troubleshooting

### Error: "Unknown type 'X' referenced"

A type is referenced but not defined:

```lumos
// ❌ Error - Player not defined
#[solana]
struct Team {
    members: [Player],  // Player doesn't exist!
}

// ✅ Fix - Define Player first
#[solana]
struct Player {
    name: String,
}

#[solana]
struct Team {
    members: [Player],
}
```

---

### Error: "Path traversal detected"

LUMOS prevents writing outside the project:

```bash
# ❌ Rejected - security risk
lumos generate schema.lumos -o ../../etc/passwd

# ❌ Rejected - escaping project
lumos generate schema.lumos -o ../other-project/

# ✅ Allowed
lumos generate schema.lumos -o ./src/generated/
```

---

### Watch mode not detecting changes?

1. **Ensure file is saved** (not just modified in buffer)
2. **Check debounce setting:**
   ```bash
   lumos generate schema.lumos --watch --debounce 100
   ```
3. **Run with verbose logging:**
   ```bash
   lumos generate schema.lumos --watch --verbose
   ```
4. **Check file permissions** on the schema file

---

### Generated Rust code has compilation errors?

1. **Validate schema first:**
   ```bash
   lumos validate schema.lumos
   ```

2. **Check for undefined types:**
   ```bash
   lumos validate schema.lumos --verbose
   ```

3. **Verify Anchor attributes syntax:**
   ```lumos
   // ❌ Wrong
   #[anchor(init payer = owner)]

   // ✅ Correct
   #[anchor(init, payer = owner)]
   ```

4. **Check type compatibility:**
   - Ensure all referenced types are defined
   - Verify enum variants have correct syntax

---

### TypeScript has type errors after generation?

1. **Check imports:**
   ```typescript
   import { PublicKey } from '@solana/web3.js';
   import * as borsh from '@coral-xyz/borsh';
   ```

2. **Verify package versions:**
   ```bash
   npm install @solana/web3.js@latest @coral-xyz/borsh@latest
   ```

3. **Regenerate types:**
   ```bash
   lumos generate schema.lumos --lang typescript
   ```

---

### Account deserialization fails?

1. **Check discriminator handling:**
   ```typescript
   // Skip 8-byte Anchor discriminator
   const data = accountInfo.data.slice(8);
   ```

2. **Verify account exists:**
   ```typescript
   if (!accountInfo) {
       throw new Error('Account not found');
   }
   ```

3. **Check data length:**
   ```typescript
   console.log('Expected:', ACCOUNT_SIZE);
   console.log('Actual:', accountInfo.data.length);
   ```

---

### Enums not deserializing correctly?

1. **Check variant order matches on-chain data:**
   ```lumos
   // Variant indices: Active=0, Paused=1, Finished=2
   enum Status {
       Active,    // 0
       Paused,    // 1
       Finished,  // 2
   }
   ```

2. **Verify discriminant byte:**
   ```typescript
   console.log('Discriminant:', data[0]);
   ```

3. **Check for struct vs tuple variants:**
   ```lumos
   // Tuple variant
   Transfer(PublicKey, u64)

   // Struct variant (different serialization!)
   Transfer { to: PublicKey, amount: u64 }
   ```

---

### "Circular dependency" error?

LUMOS doesn't support self-referential types:

```lumos
// ❌ Error - infinite size
struct Node {
    value: u32,
    next: Node,  // Can't contain itself!
}

// ✅ Fix - use PublicKey reference
struct Node {
    value: u32,
    next: Option<PublicKey>,  // Reference to another account
}
```

---

## CLI Reference

### What's the difference between `validate` and `check`?

| Command | Purpose | Speed |
|---------|---------|-------|
| `validate` | Syntax check only | Fast |
| `check` | Verify generated code matches schema | Slower |

```bash
# Quick syntax validation
lumos validate schema.lumos

# Full check (regenerates and compares)
lumos check schema.lumos
```

---

### How do I compare two schema versions?

```bash
lumos diff old-schema.lumos new-schema.lumos

# Output:
# + Added: UserV2.email_verified (bool)
# - Removed: User.legacy_field
# ~ Modified: User.name max length 20 → 32
```

---

### How do I see all available commands?

```bash
# List all commands
lumos --help

# Get help for specific command
lumos generate --help
lumos anchor --help
lumos anchor generate --help
```

---

### What does each command do?

| Command | Purpose |
|---------|---------|
| `generate` | Generate code from schema |
| `validate` | Check schema syntax |
| `init` | Create new project with template |
| `check` | Verify generated code is up-to-date |
| `diff` | Compare two schemas |
| `anchor generate` | Generate complete Anchor program |
| `anchor idl` | Generate Anchor IDL |
| `anchor space` | Calculate account sizes |
| `security analyze` | Check for vulnerabilities |
| `fuzz generate` | Generate fuzzing harness |

---

### How do I enable verbose output?

```bash
# Verbose mode
lumos generate schema.lumos --verbose

# Debug mode (maximum detail)
RUST_LOG=debug lumos generate schema.lumos
```

---

### How do I check the installed version?

```bash
lumos --version
# lumos-cli 0.1.1
```

---

## Related Documentation

- **Integration Guides:**
  - [LUMOS + Anchor Integration](/guides/anchor-integration)
  - [LUMOS + Solana CLI Integration](/guides/solana-cli-integration)
  - [LUMOS + web3.js Integration](/guides/web3js-integration)

- **Migration Guides:**
  - [Migration: TypeScript → LUMOS](/guides/migration-typescript)
  - [Migration: Anchor → LUMOS](/guides/migration-anchor)

- **Use Case Guides:**
  - [Gaming Projects](/guides/use-cases/gaming)
  - [NFT Marketplaces](/guides/use-cases/nft)
  - [DeFi Protocols](/guides/use-cases/defi)

---

**Still have questions?**
- [GitHub Issues](https://github.com/getlumos/lumos/issues) - Report bugs or request features
- [GitHub Discussions](https://github.com/getlumos/lumos/discussions) - Ask the community
