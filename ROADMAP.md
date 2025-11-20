# LUMOS Roadmap

**Vision**: Become the standard schema language for type-safe Solana development

**Last Updated**: November 20, 2025

---

## Current Status

**Phase 4.1 Complete** - VSCode extension published and docs.lumos-lang.org live!

- VSCode extension v0.5.0 published to marketplace
- Documentation site deployed with SSL
- All core features shipped and stable
- Ready for Phase 4.2: Community Examples

---

## Phase 4: Ecosystem Growth (Q1 2026)

### 4.2 Community Examples

**Goal**: Proven patterns for real-world Solana use cases

- [ ] NFT marketplace schema (mint, transfer, listing)
- [ ] DeFi staking program (stake, unstake, rewards)
- [ ] DAO governance structure (proposals, voting)
- [ ] Gaming inventory system (items, trades, upgrades)
- [ ] Token vesting contract (cliff, linear vesting)

**Success Metric**: 5 complete full-stack examples in awesome-lumos

### 4.3 Developer Experience

**Goal**: Low barrier to adoption, excellent documentation

- [ ] Interactive playground on lumos-lang.org
- [ ] Video tutorial series (YouTube)
- [ ] Migration guide from manual Borsh serialization
- [ ] Performance benchmarks vs manual implementations
- [ ] API reference documentation with examples
- [ ] "LUMOS in 5 minutes" quickstart guide

**Success Metric**: Sub-5 minute onboarding for new users

---

## Phase 5: Advanced Features (Q2 2026)

### 5.1 Schema Evolution

**Goal**: Support schema changes without breaking deployed programs

- [ ] Schema versioning syntax (`#[version = "1.0.0"]`)
- [ ] Automatic migration code generation
- [ ] Backward compatibility validation
- [ ] Deprecation warnings for old schemas
- [ ] Schema diff tool (`lumos diff v1.lumos v2.lumos`)

**Success Metric**: Zero-downtime schema upgrades

### 5.2 IDE Integration

**Goal**: Multi-editor support beyond VSCode

- [ ] Language Server Protocol (LSP) implementation
- [ ] IntelliJ IDEA / Rust Rover plugin
- [ ] Neovim plugin with Tree-sitter grammar
- [ ] Emacs mode
- [ ] Sublime Text package

**Success Metric**: LSP used by 3+ editors

### 5.3 Advanced Type System

**Goal**: Express complex Solana program constraints

- [ ] Custom derive macros support
- [ ] Const generics for fixed-size arrays
- [ ] Type aliases and imports
- [ ] Nested module support
- [ ] Generic struct/enum definitions

**Success Metric**: Support 95% of Anchor program patterns

---

## Phase 6: Ecosystem Integration (Q3 2026)

### 6.1 Framework Integration

- [ ] Anchor framework plugin
- [ ] Seahorse integration
- [ ] Native Solana program support
- [ ] Metaplex standard compatibility

### 6.2 Tooling Ecosystem

- [ ] Cargo subcommand (`cargo lumos generate`)
- [ ] GitHub Action for CI/CD
- [ ] Pre-commit hook for schema validation
- [ ] npm package for JavaScript projects

### 6.3 Security & Validation

- [ ] Static analysis for common vulnerabilities
- [ ] Account size overflow detection
- [ ] Security audit checklist generator
- [ ] Fuzzing support for generated code

---

## Future Considerations (Beyond 2026)

**Ideas under exploration** (not committed):

- Code generation for other languages (Python, C++)
- On-chain schema registry for program introspection
- Automated documentation generation from schemas
- GraphQL API generation from LUMOS schemas
- Cross-chain support (EVM, Cosmos, etc.)
- WASM target for browser-based tooling

---

## Completed Phases

### Phase 4.1: VSCode Extension Polish ✅ (Completed Nov 2025)

- [x] Published extension to VS Marketplace (v0.1.0 - v0.5.0)
- [x] Added error diagnostics with red squiggles
- [x] Implemented auto-completion for Solana types (PublicKey, Signature, etc.)
- [x] Added format-on-save support
- [x] Created quick fix suggestions for common errors
- [x] Deployed documentation site at docs.lumos-lang.org with SSL

**Result**: Full-featured VSCode extension with professional DX

### Phase 3.3: Production Polish ✅ (Completed Nov 2025)

- [x] All 64 tests passing
- [x] Zero clippy warnings, zero rustfmt violations
- [x] Security audit clean (0 vulnerabilities)
- [x] Published to crates.io (lumos-core, lumos-cli)
- [x] Organization migrated to getlumos
- [x] Homepage updated to lumos-lang.org
- [x] Comprehensive documentation
- [x] CI/CD pipeline with GitHub Actions
- [x] VSCode extension created (syntax highlighting, snippets)

### Phase 3.2: Enum Support ✅ (Completed Nov 2025)

- [x] Unit, Tuple, and Struct enum variants
- [x] Rust enum generation with sequential discriminants
- [x] TypeScript discriminated unions with `kind` field
- [x] Borsh `rustEnum()` integration

### Phase 3.1: Context-Aware Generation ✅ (Completed Nov 2025)

- [x] Anchor vs pure Borsh detection
- [x] Automatic import management
- [x] Smart derive macro handling

---

## Contributing

See an opportunity to help? Check our [Contributing Guide](CONTRIBUTING.md) or:

1. **Developers**: Claim an issue, submit a PR
2. **Content Creators**: Write tutorials, create videos
3. **Example Authors**: Build real-world schemas for awesome-lumos
4. **Community**: Test features, report bugs, suggest improvements

---

## How to Provide Feedback

- **Feature Requests**: Open a GitHub issue with label `enhancement`
- **Bug Reports**: Open a GitHub issue with label `bug`
- **Discussions**: Use GitHub Discussions for questions and ideas
- **Direct Contact**: Twitter [@getlumos](https://twitter.com/getlumos)

---

**This roadmap is a living document** - priorities may shift based on community feedback and ecosystem needs.

**Last Updated**: November 20, 2025
