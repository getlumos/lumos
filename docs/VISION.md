# LUMOS Vision: From Schema DSL to Programming Language

**Last Updated**: November 22, 2025

---

## The Journey So Far

LUMOS began with a focused mission: eliminate the pain of manually writing Borsh serialization code for Solana programs. Write schemas once in `.lumos` syntax, generate production-ready Rust and TypeScript with guaranteed compatibility.

**We achieved this goal.** LUMOS v0.1.1 is production-ready, battle-tested, and serving real developers.

But along the way, our community showed us something bigger.

---

## The Bigger Vision

Developers today face **workflow fragmentation**:

- Bash scripts for small tasks
- Python for automation
- TypeScript for build processes
- GitHub Actions for CI/CD
- Custom scripts for blockchain operations
- Cloud Functions for async jobs
- Makefiles/Justfiles for orchestration

**5-8 different tools** just to automate something end-to-end.

No unified, typed, programmable workflow engine exists for modern developers—especially in the crypto/Web3 space.

**LUMOS can solve this.**

---

## Where We're Going

> **⚠️ Important Note on Scope & Timeline**
>
> This vision document outlines **long-term aspirations** for LUMOS beyond its core mission as a schema language.
>
> **Current Focus (v0.1.x - v1.x):**
> - Schema language remains the **primary focus**
> - All improvements, features, and releases will center on schema generation
> - Community examples, documentation, and tooling for schemas
> - Stability and production-readiness for schema use cases
>
> **Future Exploration (v2.0+, 2026+):**
> - The workflow programming language vision is **exploratory**
> - Development will only begin after v1.0 is mature and stable
> - Community feedback will guide whether/when to pursue this direction
> - The schema DSL will always remain a core, standalone feature
>
> **No Breaking Changes:** The transition from schema DSL to programming language (if pursued) will be gradual, opt-in, and backwards-compatible.

### The Evolution

```
LUMOS v0.1.x (Today)
    ↓
Schema DSL for Solana
    ↓
LUMOS 1.0 (2026)
    ↓
Mature, Stable Schema Language
    ↓
LUMOS 2.0 (Future - TBD)
    ↓
Typed Workflow Programming Language (Exploratory)
```

### Phase 1: Schema Language ✅ (Complete)

- **v0.1.1 shipped** - 108 tests passing, zero warnings
- Production-ready schema generation (Rust + TypeScript)
- VSCode extension with diagnostics and autocomplete
- 5 community examples (NFT, DeFi, DAO, Gaming, Vesting)
- Interactive playground at docs.lumos-lang.org

**Status**: Mission accomplished. Schema language is stable and ready.

---

### Phase 2: Language Foundation (Future)

**Vertical Expansion** - Building deep, powerful core:

#### 1. Core Language
- Parser, AST, and evaluator/VM
- Variables, functions, and modules
- Comprehensive error system with diagnostics
- Scheme-inspired functional design

#### 2. Type System
- Gradual typing (TypeScript-inspired philosophy)
- Solana-native primitive types (PublicKey, Account, Lamports, Instruction)
- Type inference for common patterns
- Load types from Anchor IDLs automatically
- Compile-time validation of workflows

#### 3. Compiler & IR
- LUMOS → Solana CLI commands
- LUMOS → Anchor CLI operations
- LUMOS → Solv automation scripts
- LUMOS → TOML/YAML/JSON configs
- Multi-target code generation

#### 4. Language Server Protocol (LSP)
- Expand VSCode extension with deep language support
- Multi-editor support (IntelliJ, Neovim, Emacs)
- Autocomplete from IDLs and schemas
- Inline type information and documentation
- Real-time error checking

#### 5. Runtime Engine
- Execute workflows locally or in the cloud
- Native RPC integration
- Jito bundle execution
- Transaction builder helpers
- Sandbox/dry-run mode for safety

**Goal**: Transform LUMOS from schema DSL into a real programming language specialized for developer workflows.

---

### Phase 3: Ecosystem Growth (Future)

**Horizontal Expansion** - Growing the platform:

#### 1. Multi-Chain Support
- Extend beyond Solana to Sui, Aptos, Starknet
- EVM-compatible chains (Neon, Base, Aurora)
- Unified workflow primitives across chains
- Cross-chain orchestration

#### 2. DevOps Automation
- Docker orchestration workflows
- Kubernetes deployment automation
- GitHub Actions generation
- CI/CD pipeline templates
- Cloud deployment (Fly.io, AWS, Netlify)

#### 3. Cloud Execution Platform
- LUMOS Cloud for running workflows
- Scheduled jobs (cron-like for crypto operations)
- Monitoring and logging dashboards
- Secrets management
- Execution guarantees and reliability

#### 4. Package Ecosystem
- LPM (Lumos Package Manager)
- Community-contributed workflow libraries
- Template marketplace for common patterns
- Plugin registry for extensions

#### 5. Infrastructure as Code
- Manage RPC provider configurations
- Automate validator infrastructure
- On-chain program deployment pipelines
- Configuration provisioning and versioning

**Goal**: Make LUMOS the go-to platform for any developer workflow, starting with crypto but expanding beyond.

---

## The ENDGAME

**LUMOS will become the typed workflow language for developer automation.**

Think of it as:
- **"TypeScript of developer workflows"** - Type-safe scripting with excellent DX
- **"Terraform for crypto operations"** - Declarative infrastructure and workflows
- **"Makefile/Justfile evolved"** - Programmable automation done right

### What This Means

**Write workflows once, run anywhere:**
- Local development (instant feedback)
- Cloud execution (scale and reliability)
- On-chain automation (Solana, multi-chain)

**Type-safe by default:**
- Catch errors at compile time, not runtime
- IntelliSense and autocomplete everywhere
- Refactor with confidence

**Solana-native primitives:**
- PublicKey, Account, Instruction types built-in
- Load program types from Anchor IDLs
- Transaction building made simple

**Extensible and composable:**
- Scheme-inspired macro system
- Community packages and templates
- Plugin ecosystem for integrations

---

## Our Philosophy

### 1. Vertical Before Horizontal

We build **deep foundations** before expanding wide:
- Type system before multi-chain support
- Runtime before cloud platform
- Core language before ecosystem

**Why?** Deep technical layers create a moat that ensures LUMOS remains best-in-class.

### 2. Community-First

LUMOS is built **with and for developers**:
- Open source core (always)
- Community-driven roadmap
- Real-world examples and templates
- Active feedback loops

### 3. Type Safety

**Catch errors early:**
- Compile-time validation prevents runtime failures
- Strong types reduce bugs
- IDE support makes development fast

### 4. Elegant Simplicity

Inspired by Scheme's minimalism:
- Simple syntax, powerful semantics
- Functional programming patterns
- Composability over complexity
- Expressive macros for abstraction

---

## What Sets LUMOS Apart

### The Intersection of Five Domains

LUMOS uniquely combines:

1. **Programming language design** - Real compiler, type system, runtime
2. **Workflow automation** - Purpose-built for developer operations
3. **Cloud orchestration** - Local development, cloud execution
4. **Crypto engineering** - Solana-native, blockchain-first
5. **Developer experience** - Type safety, LSP, great tooling

**Most tools excel at one.** LUMOS excels at all five.

### Competitive Positioning

| Tool | Strengths | Weaknesses |
|------|-----------|------------|
| **TypeScript scripts** | Familiar, flexible | Too general, verbose, no workflow primitives |
| **Python automation** | Easy syntax | Weak typing, not domain-specific |
| **Terraform/HCL** | Great for infra | Not blockchain-native, static |
| **Makefile/Justfile** | Simple tasks | Not a real language, limited expressiveness |
| **LUMOS** | Typed, workflow-first, crypto-native, extensible | New (requires learning) |

**LUMOS fills the gap** that existing tools cannot.

---

## Current Status vs. Vision

### Today (v0.1.1)
```rust
// .lumos schema
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    level: u16,
    experience: u64,
}
```

Generates Rust + TypeScript with Borsh serialization.

### Tomorrow (LUMOS 2.0 Vision)
```scheme
;; LUMOS workflow language
(workflow deploy-game
  (let program (anchor build "game-program"))
  (let keypair (solana keygen "deploy-key.json"))

  (when (not (deployed? program))
    (anchor deploy program keypair))

  (jito bundle
    (anchor call program "initialize" {admin: keypair})
    (anchor call program "create-player" {wallet: (env "PLAYER_WALLET")}))

  (log "Game deployed successfully!"))
```

Type-safe, executable, composable workflows.

---

## Roadmap Alignment

This vision **complements** our existing roadmap:

- **ROADMAP.md** - Near-term schema language evolution (Phase 5-6, Q2-Q3 2026)
- **VISION.md** - Long-term programming language direction (2026+)

The schema DSL continues to improve while we build the language foundation in parallel.

**No breaking changes.** The transition will be gradual and opt-in.

---

## Get Involved

LUMOS is open source and community-driven. Here's how you can help:

### For Developers
- **Try LUMOS** - Use it in your Solana projects
- **Contribute code** - Check GitHub issues for good first issues
- **Build examples** - Add to awesome-lumos repository
- **Report bugs** - Help us improve quality

### For Designers
- **Improve docs** - Better tutorials, diagrams, examples
- **Create content** - Blog posts, videos, workshops

### For Enthusiasts
- **Spread the word** - Share LUMOS with other developers
- **Join discussions** - Discord, GitHub Discussions, Reddit
- **Provide feedback** - Tell us what you need

### Community Links
- **GitHub**: https://github.com/getlumos/lumos
- **Discord**: [Join our server] (coming soon)
- **Twitter/X**: [@getlumos](https://twitter.com/getlumos)
- **Documentation**: https://docs.lumos-lang.org

---

## Frequently Asked Questions

### Will the schema DSL go away?
**No.** The schema DSL (current LUMOS) remains fully supported. The language evolution is additive, not replacement.

### When will LUMOS 2.0 ship?
We're focused on building the right foundation, not rushing to arbitrary deadlines. Follow our progress on GitHub.

### Is this still for Solana only?
LUMOS starts with Solana because that's where the pain is greatest. But the vision is multi-chain and beyond crypto (DevOps, cloud automation).

### Will LUMOS stay open source?
**Yes.** The core language, compiler, runtime, and standard library will always be open source. We may monetize cloud services and premium templates, following the Terraform/TypeScript model.

### How can I stay updated?
- Watch the GitHub repo for releases
- Join Discord for real-time updates
- Follow [@getlumos](https://twitter.com/getlumos) on X/Twitter

---

## Closing Thoughts

Developer workflows are **too important** to be fragmented across a dozen tools.

LUMOS started as a schema DSL. But the need for a typed, workflow-first programming language is clear.

**We're building it.**

Join us on this journey. Let's make developer automation elegant, type-safe, and joyful.

---

**The best way to predict the future is to build it.**

Let's build LUMOS together.

---

*This is a living document. As we learn and grow, this vision will evolve. Last updated: November 22, 2025.*
