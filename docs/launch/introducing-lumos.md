# Introducing LUMOS: The Type-Safe Schema Language for Solana

**TL;DR:** LUMOS lets you define your data structures once and generate production-ready code for Rust, TypeScript, Python, Go, and Ruby—with guaranteed Borsh serialization compatibility. No more type drift. No more manual synchronization. Just write `.lumos` and ship.

---

## The Problem Every Solana Developer Knows

If you've built a full-stack Solana application, you've felt this pain:

```rust
// In your Anchor program (Rust)
#[account]
pub struct UserAccount {
    pub wallet: Pubkey,
    pub balance: u64,
    pub level: u16,
}
```

```typescript
// In your frontend (TypeScript)
interface UserAccount {
  wallet: PublicKey;
  balance: number;
  level: number;
}

const UserAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.u64('balance'),
  borsh.u16('level'),
]);
```

Two files. Same structure. Manual synchronization.

**What happens when you add a field to the Rust struct but forget to update TypeScript?**

Runtime deserialization errors. Silent data corruption. Hours of debugging.

**What happens when field order doesn't match?**

Borsh serialization fails. Your frontend reads garbage data. Users lose funds.

This isn't a theoretical problem. It's the #1 source of bugs in full-stack Solana development.

---

## The Solution: Write Once, Generate Everywhere

LUMOS is a schema language purpose-built for Solana. Define your types once:

```rust
#[solana]
#[account]
struct UserAccount {
    wallet: PublicKey,
    balance: u64,
    level: u16,
}
```

Run one command:

```bash
lumos generate schema.lumos --lang rust,typescript,python
```

Get production-ready code in every language:

**Rust (with Anchor integration):**
```rust
use anchor_lang::prelude::*;

#[account]
pub struct UserAccount {
    pub wallet: Pubkey,
    pub balance: u64,
    pub level: u16,
}
```

**TypeScript (with Borsh schemas):**
```typescript
import { PublicKey } from '@solana/web3.js';

export interface UserAccount {
  wallet: PublicKey;
  balance: number;
  level: number;
}

export const UserAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.u64('balance'),
  borsh.u16('level'),
]);
```

**Python (with borsh-construct):**
```python
@dataclass
class UserAccount:
    wallet: Pubkey
    balance: int
    level: int
```

One source of truth. Zero drift. Guaranteed compatibility.

---

## Why We Built LUMOS

We've been building on Solana since 2021. Every project hit the same wall: keeping types synchronized across Rust programs and TypeScript frontends.

We tried:
- **Manual synchronization** → Error-prone, doesn't scale
- **Code comments** → Gets stale instantly
- **Shared JSON schemas** → No Borsh support, no Anchor integration
- **Custom scripts** → Maintenance nightmare

None of these solutions worked. So we built LUMOS.

LUMOS isn't just a code generator. It's a **schema language** designed specifically for Solana's constraints:

- **Borsh-native**: Field order matters. LUMOS guarantees it.
- **Anchor-aware**: Detects `#[account]` and generates appropriate code
- **Multi-language**: One schema → Rust, TypeScript, Python, Go, Ruby
- **IDE-integrated**: Full LSP support for VS Code, IntelliJ, Neovim, Emacs, Sublime

---

## What Makes LUMOS Different

### 1. Context-Aware Code Generation

LUMOS understands Solana development patterns. It knows that `#[account]` structs need different treatment than regular structs:

```rust
#[solana]
#[account]
struct Config {
    admin: PublicKey,
}

#[solana]
struct Event {
    timestamp: i64,
    data: String,
}
```

Generated Rust automatically uses the right imports and derives:
- `Config` → Uses `anchor_lang::prelude::*`, no manual derives
- `Event` → Uses `AnchorSerialize`, `AnchorDeserialize`

No more derive conflicts. No more import errors.

### 2. Complete Enum Support

Real Solana programs need enums. LUMOS supports all three variant types:

```rust
#[solana]
enum GameInstruction {
    // Unit variants
    Pause,
    Resume,

    // Tuple variants
    UpdateScore(PublicKey, u64),

    // Struct variants
    Initialize {
        authority: PublicKey,
        max_players: u32,
    },
}
```

TypeScript gets discriminated unions with full type safety:

```typescript
type GameInstruction =
  | { kind: 'Pause' }
  | { kind: 'Resume' }
  | { kind: 'UpdateScore'; value: [PublicKey, number] }
  | { kind: 'Initialize'; authority: PublicKey; maxPlayers: number };
```

### 3. Advanced Type System

LUMOS supports the types you actually need:

- **Generics**: `struct Response<T> { data: T, timestamp: i64 }`
- **Fixed arrays**: `[u8; 32]` for hashes, `[PublicKey; 5]` for authority lists
- **Type aliases**: `type TokenAmount = u64`
- **Module system**: Rust-style `mod` declarations and imports
- **Deprecation**: `#[deprecated("Use new_field instead")]`

### 4. Schema Evolution

Schemas change. LUMOS helps you manage it:

```bash
# Check compatibility between versions
lumos check-compat v1.lumos v2.lumos

# Generate migration code
lumos migrate v1.lumos v2.lumos --output migrations/

# Visual diff
lumos diff v1.lumos v2.lumos
```

### 5. Security Built-In

LUMOS includes security tools out of the box:

- **Static analyzer**: Detects missing signer checks, unchecked arithmetic
- **Fuzzing support**: Generate fuzz targets for cargo-fuzz
- **Audit checklist**: Auto-generate security review checklists

```bash
lumos security analyze schema.lumos
lumos security fuzz schema.lumos --output fuzz/
lumos security audit schema.lumos
```

### 6. Complete Anchor Plugin

Generate entire Anchor programs from schemas:

```bash
lumos anchor generate schema.lumos --output programs/
```

This creates:
- Account structs with `#[account]` macro
- `LEN` constants for all types
- `#[derive(Accounts)]` instruction contexts
- Proper imports and `declare_id!()`

---

## IDE Support That Actually Works

We built a Language Server (LSP) so you get a real development experience:

- **Real-time diagnostics**: See errors as you type
- **Auto-completion**: All Solana types, primitives, attributes
- **Hover information**: Type definitions and documentation
- **Go to definition**: Navigate your schema files

**Supported editors:**
- VS Code (dedicated extension)
- IntelliJ IDEA / Rust Rover
- Neovim (with Tree-sitter)
- Emacs (lumos-mode)
- Sublime Text

Install the LSP:
```bash
cargo install lumos-lsp
```

---

## Five Languages, One Schema

LUMOS generates code for your entire stack:

| Language | Use Case | Serialization |
|----------|----------|---------------|
| **Rust** | Anchor programs, native Solana | Borsh |
| **TypeScript** | Frontend, Node.js backends | @coral-xyz/borsh |
| **Python** | Scripts, data analysis, Seahorse | borsh-construct |
| **Go** | High-performance backends | go-borsh |
| **Ruby** | Scripts, Rails integrations | borsh-rb |

```bash
# Generate all languages
lumos generate schema.lumos --lang rust,typescript,python,go,ruby
```

---

## Getting Started

### Install

```bash
# Via Cargo (Rust developers)
cargo install lumos-cli

# Via npm (no Rust required)
npm install -g @getlumos/cli

# Via Docker
docker pull ghcr.io/getlumos/lumos
```

### Create Your First Schema

```bash
lumos init my-project
cd my-project
```

Edit `schema.lumos`:

```rust
#[solana]
#[account]
struct Counter {
    authority: PublicKey,
    count: u64,
}

#[solana]
enum CounterInstruction {
    Initialize { authority: PublicKey },
    Increment,
    Decrement,
}
```

Generate code:

```bash
lumos generate schema.lumos
```

That's it. You now have synchronized Rust and TypeScript code.

### CI/CD Integration

Add to your GitHub Actions:

```yaml
- uses: getlumos/lumos-action@v1
  with:
    schema: 'schemas/**/*.lumos'
    fail-on-drift: true
```

This validates schemas on every PR and ensures generated code is always committed.

---

## The Numbers

- **300+ tests** passing (100% success rate)
- **5 languages** supported
- **6 IDE integrations** via LSP
- **0 vulnerabilities** (cargo audit clean)
- **<100ms** generation time for typical schemas

---

## What's Next

LUMOS is just getting started. Our roadmap includes:

**Era 1 (Current):** Complete DSL with all Solana patterns
- ✅ Multi-language generation
- ✅ Full IDE support
- ✅ Anchor plugin
- ✅ Security tools

**Era 2 (2026):** Evolution to workflow language
- Parser & runtime for executable LUMOS scripts
- Type-safe Solana operations
- Package ecosystem
- Deployment automation

We're building the TypeScript of Solana development—a language that handles schemas, scripts, and automation in one unified, type-safe experience.

---

## Join the Community

LUMOS is open source and MIT/Apache 2.0 licensed.

- **GitHub**: [github.com/getlumos/lumos](https://github.com/getlumos/lumos)
- **Documentation**: [lumos-lang.org](https://lumos-lang.org)
- **Discord**: Join our community for support and discussion
- **Twitter**: [@getlumos](https://twitter.com/getlumos)

**Star us on GitHub** if LUMOS solves a problem you've had. Your stars help other developers discover the project.

---

## Try It Now

```bash
cargo install lumos-cli
lumos init hello-lumos
cd hello-lumos
lumos generate schema.lumos
```

In 60 seconds, you'll have synchronized Rust and TypeScript code.

No more type drift. No more manual synchronization. Just write `.lumos` and ship.

**Welcome to type-safe Solana development.**

---

*LUMOS is built by developers who've felt the pain of type synchronization. We're committed to making Solana development faster, safer, and more enjoyable.*

*Have feedback? Open an issue or join our Discord. We read everything.*
