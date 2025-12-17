# Reddit Post: r/solana

---

## Title

We built LUMOS: A schema language that generates Rust + TypeScript from a single source (with Anchor integration)

---

## Post Content

Hey r/solana,

After years of manually keeping Rust structs and TypeScript interfaces in sync, we built something to fix it permanently.

**LUMOS** is a schema language specifically designed for Solana development. You define your types once:

```rust
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    balance: u64,
    level: u16,
}
```

Then generate perfectly synchronized code:

```bash
lumos generate schema.lumos
```

This outputs:
- **Rust** with correct Anchor imports and derives
- **TypeScript** interfaces with Borsh schemas
- **Python, Go, Ruby** if you need them

### Why This Matters

If you've built a full-stack Solana app, you've probably:

1. Added a field to your Anchor program
2. Forgotten to update the TypeScript interface
3. Spent hours debugging why deserialization failed

LUMOS eliminates this entire class of bugs. One schema, multiple outputs, guaranteed compatibility.

### Features

- **Anchor-aware**: Detects `#[account]` and generates appropriate code (no derive conflicts)
- **Full enum support**: Unit, tuple, and struct variants
- **IDE integration**: LSP for VS Code, IntelliJ, Neovim, Emacs, Sublime
- **Schema evolution**: Diff, compatibility checking, migration generation
- **Security tools**: Static analysis, fuzzing support

### Quick Start

```bash
cargo install lumos-cli
lumos init my-project
cd my-project
lumos generate schema.lumos
```

Or without Rust:

```bash
npm install -g @getlumos/cli
```

### Links

- GitHub: https://github.com/getlumos/lumos
- Docs: https://lumos-lang.org
- 300+ tests passing, published to crates.io

### Feedback Welcome

We built this to solve our own pain. Would love to hear:

1. What type synchronization approaches have you tried?
2. What features would make this more useful for your projects?
3. Any edge cases we should handle?

Happy to answer questions!

---

## Post Guidelines

- Post on weekday afternoon (highest r/solana engagement)
- Monitor and respond to all comments for first 24 hours
- Don't be defensive about criticism—acknowledge and learn
- Offer to help if anyone has specific use cases
- Follow up with interesting questions/suggestions as GitHub issues
