# LUMOS

**Illuminate your Solana development**

LUMOS is a powerful code generation framework that bridges TypeScript and Rust, eliminating the pain of maintaining duplicate type definitions across your full-stack Solana applications.

## ✨ Vision

Stop writing the same types twice. Define your data structures once in LUMOS, and automatically generate:
- ✅ Rust structs with Borsh serialization (for Anchor programs)
- ✅ TypeScript interfaces and SDK code (for your frontend)
- ✅ Perfectly synchronized types across your entire stack

## 🎯 Problem

Building full-stack Solana applications requires maintaining identical type definitions in two languages:

```rust
// Rust (Anchor program)
#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserAccount {
    pub id: u64,
    pub wallet: Pubkey,
    pub balance: u64,
}
```

```typescript
// TypeScript (Frontend)
interface UserAccount {
  id: number;
  wallet: PublicKey;
  balance: number;
}
```

**The pain:**
- 🔴 Manual synchronization is error-prone
- 🔴 Type mismatches cause runtime bugs
- 🔴 Refactoring requires changes in multiple places
- 🔴 No single source of truth

## 💡 Solution

LUMOS provides a single schema that generates both languages:

```toml
# lumos.toml
[schema]
name = "UserAccount"
solana = true

[[schema.fields]]
name = "id"
type = "u64"

[[schema.fields]]
name = "wallet"
type = "PublicKey"

[[schema.fields]]
name = "balance"
type = "u64"
```

Run `lumos build` and get both Rust and TypeScript implementations, guaranteed to stay in sync.

## 🚀 Roadmap

### Phase 1: Core TypeScript ↔ Rust Codegen (Months 1-3)
- [x] Project setup and architecture
- [ ] TOML schema parser
- [ ] Rust code generator (with Borsh serialization)
- [ ] TypeScript code generator
- [ ] CLI tool (`lumos init`, `lumos build`)
- [ ] Basic examples and documentation
- [ ] Solana Foundation grant application

### Phase 2: DSL Features (Months 4-6)
- [ ] Custom `.lumos` syntax (prettier than TOML)
- [ ] PDA (Program Derived Address) helpers
- [ ] Anchor instruction generation
- [ ] VSCode extension with syntax highlighting

### Phase 3: ZK Primitives (Months 7-12)
- [ ] Zero-Knowledge proof type support
- [ ] ZK circuit generation helpers
- [ ] Integration with ZK libraries

### Future: Multi-Language Platform
- [ ] TypeScript ↔ Go support
- [ ] TypeScript ↔ Python support
- [ ] Plugin architecture for community generators

## 🏗️ Architecture

```
LUMOS Core
    │
    ├─ Schema Parser (TOML → IR)
    ├─ Intermediate Representation (IR)
    └─ Code Generators
        ├─ Rust Generator (Anchor-compatible)
        └─ TypeScript Generator
```

**Designed for extensibility:** Adding new language targets is straightforward via the plugin architecture.

## 🛠️ Tech Stack

- **Language:** Rust
- **Parser:** `syn` + `toml`
- **Code Generation:** `quote`
- **CLI:** `clap` v4
- **Testing:** `cargo test`

## 📦 Installation

> **Note:** LUMOS is currently in early development. Installation instructions will be available when we release v0.1.0.

```bash
# Coming soon
cargo install lumos-cli
```

## 🎓 Quick Start

```bash
# Initialize a new LUMOS project
lumos init my-project

# Build your schemas
cd my-project
lumos build

# Output:
#   - ./generated/rust/lib.rs
#   - ./generated/typescript/types.ts
```

## 🤝 Contributing

LUMOS is in active early development. We welcome contributions!

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

This follows the same licensing as the Rust programming language.

## 🙏 Credits

Created by [RECTOR](https://github.com/rz1989s) at [RECTOR-LABS](https://github.com/RECTOR-LABS).

Built for the Solana developer community with ❤️

---

**Status:** 🚧 Early Development - Not production-ready

**Version:** 0.1.0-alpha

**Target Release:** Q2 2025
