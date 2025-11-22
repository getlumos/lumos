# LUMOS Marketing Website - Complete Specification

## Vision

Create a **world-class marketing website** for LUMOS that rivals the best developer tool sites (Linear, Vercel, Stripe) while staying authentic to the technical Solana developer audience.

**Goal**: Convert visitors from "What is LUMOS?" to "I need to try this now" in under 30 seconds.

---

## Target Audience

- **Primary**: Solana developers (Rust + TypeScript experience)
- **Pain Point**: Manually maintaining Borsh serialization across Rust programs and TypeScript clients
- **Motivation**: Type safety, reduced bugs, faster development
- **Behavior**: Skeptical, code-first, values performance and correctness

---

## Success Metrics

- [ ] Lighthouse score 95+ (performance, accessibility, SEO, best practices)
- [ ] Time to Interactive < 1.5 seconds
- [ ] Bounce rate < 40%
- [ ] Average session duration > 2 minutes
- [ ] GitHub CTR (click-through rate) > 15%
- [ ] CLI installation instructions copied > 25%
- [ ] Mobile responsive (perfect on all devices)
- [ ] WCAG 2.1 AA accessibility compliance

---

## Technology Stack

### Core Framework
- **Next.js 14** (App Router, React Server Components)
- **TypeScript** (strict mode, dogfooding type safety)
- **Tailwind CSS** (utility-first, dark theme)
- **Framer Motion** (smooth animations, page transitions)

### Features & Integrations
- **Shiki** (syntax highlighting for `.lumos`, Rust, TypeScript)
- **Lucide React** (modern icon system)
- **Next SEO** (OpenGraph, Twitter Cards, structured data)
- **Vercel Analytics** (Core Web Vitals, user behavior)

### Deployment
- **Vercel** (edge functions, automatic CI/CD)
- **Custom domain**: lumos-lang.org
- **CDN**: Automatic via Vercel Edge Network

---

## Page Structure

### 1. Hero Section (Above the Fold)
**Purpose**: Hook developers in 5 seconds

**Elements**:
- **Headline**: "Type-Safe Schemas for Solana. Write Once. Zero Bugs."
- **Subheadline**: "Define data structures in `.lumos` syntax. Generate production-ready Rust + TypeScript with guaranteed Borsh compatibility."
- **Primary CTA**: "Get Started →" (docs.lumos-lang.org/quickstart)
- **Secondary CTA**: "View on GitHub ★" (with live star count)
- **Terminal Demo**: Animated terminal showing:
  ```bash
  $ cargo install lumos-cli
  $ lumos init my-project
  $ lumos generate schema.lumos
  ✓ Generated: src/generated.rs
  ✓ Generated: src/generated.ts
  ```

**Visual**: Split-screen code comparison (`.lumos` → Rust + TypeScript) with animated generation flow

**Trust Badges**:
- ✅ 74/74 Tests Passing
- 📦 Published on crates.io
- 🔒 Zero Vulnerabilities
- 🎨 VSCode Extension

---

### 2. Problem Statement (The Pain)
**Headline**: "Stop Writing Borsh Serialization Twice"

**3-Column Comparison**:

| ❌ The Old Way | ✨ The LUMOS Way | ✅ The Result |
|----------------|------------------|---------------|
| Manual Rust structs | Write schema once | 10x faster development |
| Manual TypeScript types | Auto-generate both languages | Zero serialization bugs |
| Hand-written Borsh schemas | Guaranteed synchronization | Perfect Anchor integration |
| Type mismatches = runtime errors | Catch errors at compile time | Production-ready code |
| Duplicate maintenance | Single source of truth | Sleep soundly 😴 |

---

### 3. Live Code Demo (Interactive)
**Headline**: "See It In Action"

**Features**:
- **Interactive Monaco Editor** (or static tabs if too complex)
- **Real-time generation**: Edit `.lumos` schema, see Rust + TS output live
- **Syntax highlighting**: All 3 languages (LUMOS, Rust, TypeScript)
- **Copy buttons**: One-click copy for each output
- **Example selector**: Dropdown with Gaming | NFT | DeFi | DAO schemas
- **Download button**: Export all files as `.zip`

**Default Example** (Gaming):
```rust
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    level: u16,
    experience: u64,
    inventory: [PublicKey],
}
```

---

### 4. Features Grid (Why LUMOS?)
**Headline**: "Everything You Need for Type-Safe Solana Development"

**6 Feature Cards** (3x2 grid, each with icon + description):

1. **⚡ Context-Aware Generation**
   - Detects `#[account]` for Anchor integration
   - Smart import management
   - Automatic derive macros

2. **🎯 Full Type Support**
   - Primitives: u8-u128, i8-i128, bool, String
   - Solana types: PublicKey, Signature
   - Complex: Vec<T>, Option<T>, enums

3. **🔧 Seamless Anchor Integration**
   - Zero manual derives
   - Drop-in replacement
   - Works with existing programs

4. **📝 TypeScript + Borsh**
   - Perfect type definitions
   - Automatic Borsh schemas
   - Web3.js compatible

5. **🚀 Production Ready**
   - 74/74 tests passing
   - Published on crates.io v0.1.1
   - Battle-tested in real projects

6. **🎨 VSCode Extension**
   - Syntax highlighting
   - IntelliSense auto-completion
   - Error diagnostics
   - Quick fixes

---

### 5. Before/After Comparison (Code Example)
**Headline**: "From This... To This"

**BEFORE (Manual Borsh)** - ~50 lines:
```rust
// Rust side
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Player {
    pub wallet: Pubkey,
    pub level: u16,
    pub xp: u64,
}
```

```typescript
// TypeScript side (separate file)
import * as borsh from '@coral-xyz/borsh';

export interface Player {
  wallet: PublicKey;
  level: number;
  xp: number;
}

// Manual Borsh schema - easy to make mistakes!
export const PlayerSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.u16('level'),
  borsh.u64('xp'), // Did you match field order? 🤞
]);
```

**AFTER (LUMOS)** - 7 lines:
```rust
#[solana]
#[account]
struct Player {
    wallet: PublicKey,
    level: u16,
    xp: u64,
}
```

**Caption**: "Run `lumos generate` and get production-ready Rust + TypeScript. Guaranteed compatible."

---

### 6. Use Cases (Who Is This For?)
**Headline**: "Built for the Solana Ecosystem"

**4 Persona Cards**:

1. **🎮 Game Developers**
   - Player accounts, inventory systems, match results
   - Real-time multiplayer state synchronization
   - Example: 53 types from awesome-lumos/gaming

2. **💰 DeFi Builders**
   - Staking pools, vesting schedules, AMM state
   - Type-safe across smart contracts + frontends
   - Example: Token vesting, liquidity pools

3. **🖼️ NFT Creators**
   - Metadata schemas, marketplace state
   - Collection management, royalty tracking
   - Example: NFT marketplace with 12 types

4. **🏛️ DAO Developers**
   - Governance proposals, voting records, treasury
   - Multi-sig workflows, member management
   - Example: DAO governance with 8 instruction types

---

### 7. Stats Section (Social Proof)
**Background**: Subtle gradient with glow effect

**4 Big Numbers** (animated count-up on scroll):
```
┌─────────────┬─────────────┬─────────────┬─────────────┐
│   74/74     │    v0.1.1   │      2      │    100%     │
│Tests Passing│Latest Stable│  Languages  │ Type Safety │
└─────────────┴─────────────┴─────────────┴─────────────┘
```

---

### 8. Ecosystem Overview
**Headline**: "Complete Developer Experience"

**4 Ecosystem Cards** (links to each resource):

1. **📦 CLI Tool** (crates.io)
   - `cargo install lumos-cli`
   - Generate, validate, init, check commands
   - [View on crates.io →](https://crates.io/crates/lumos-cli)

2. **📚 Documentation** (docs.lumos-lang.org)
   - Quickstart guide, API reference
   - Migration guides, best practices
   - [Read the docs →](https://docs.lumos-lang.org)

3. **🔌 VSCode Extension** (Marketplace)
   - Syntax highlighting, snippets
   - Real-time validation, quick fixes
   - [Install extension →](https://marketplace.visualstudio.com/items?itemName=lumos.lumos-vscode)

4. **🌟 Examples** (awesome-lumos)
   - 5 production-ready templates
   - Gaming, DeFi, NFT, DAO, Vesting
   - [Browse examples →](https://github.com/getlumos/awesome-lumos)

---

### 9. Getting Started (Final CTA)
**Background**: Bold gradient (purple → blue → cyan)

**Headline**: "Start Building in 5 Minutes"

**3-Step Visual Process**:
```
1️⃣ Install             2️⃣ Create              3️⃣ Generate
━━━━━━━━━━━━━━━        ━━━━━━━━━━━━━━━       ━━━━━━━━━━━━━━━
$ cargo install        $ lumos init          $ lumos generate
  lumos-cli             my-project            schema.lumos

  30 seconds            30 seconds            Instant ⚡
```

**Large CTA Button**: "Get Started Now →"
**Secondary Link**: "Or view the quickstart guide"

---

### 10. Footer
**Background**: Darkest shade (#0A0F1C)

**4 Columns**:

**Resources**
- Documentation
- Quickstart Guide
- API Reference
- Migration Guide

**Community**
- GitHub ⭐
- Report an Issue
- Contributing
- Changelog

**Examples**
- awesome-lumos
- Gaming Examples
- DeFi Examples
- NFT Examples

**Legal**
- License (MIT)
- Security Policy
- Code of Conduct

**Bottom Bar**:
```
Copyright © 2025 LUMOS  •  Built with ❤️ for the Solana ecosystem
```

---

## Design System

### Color Palette (Dark Theme)
```css
/* Primary Brand Colors */
--lumos-purple: #7C3AED;      /* Primary CTA, headings */
--lumos-blue: #3B82F6;        /* Secondary accents */
--lumos-cyan: #06B6D4;        /* Highlights, hover states */

/* Backgrounds */
--bg-darkest: #0A0F1C;        /* Footer, deep sections */
--bg-dark: #0F172A;           /* Main background */
--bg-card: #1E293B;           /* Card backgrounds */
--bg-elevated: #334155;       /* Elevated elements */

/* Text */
--text-primary: #FFFFFF;      /* Headings */
--text-secondary: #CBD5E1;    /* Body text */
--text-muted: #94A3B8;        /* Captions, labels */
```

### Typography
- **Headings**: Inter, 700-900 weight
- **Body**: Inter, 400-500 weight
- **Code**: JetBrains Mono / Fira Code, 400-500 weight

### Animation Principles
- **Timing**: 150-300ms (fast, snappy)
- **Easing**: ease-out for entrances, ease-in for exits
- **Distance**: Subtle movements (10-20px max)
- **Scroll animations**: Intersection Observer

---

## Technical Requirements

### Performance Targets
- [ ] First Contentful Paint (FCP) < 1.0s
- [ ] Time to Interactive (TTI) < 1.5s
- [ ] Largest Contentful Paint (LCP) < 2.0s
- [ ] Cumulative Layout Shift (CLS) < 0.1
- [ ] Total bundle size < 200KB (gzipped)

### Accessibility
- [ ] Semantic HTML5 elements
- [ ] ARIA labels where needed
- [ ] Keyboard navigation
- [ ] Color contrast ≥ 4.5:1 (WCAG AA)
- [ ] Screen reader tested
- [ ] Reduced motion preference respected

---

## Development Phases

### Phase 1: Foundation (Week 1)
- Set up Next.js 14 project
- Configure Tailwind CSS
- Design system implementation
- Basic page structure

### Phase 2: Content (Week 2)
- Hero section with animated terminal
- Problem/solution comparison
- Features grid
- Before/after code comparison

### Phase 3: Interactive Features (Week 3)
- Live code demo
- Syntax highlighting
- Copy-to-clipboard
- Example selector

### Phase 4: Polish & Optimization (Week 4)
- Animations
- Performance optimization
- SEO implementation
- Accessibility audit

### Phase 5: Deployment (Week 5)
- Vercel deployment
- Custom domain configuration
- Analytics integration
- Final Lighthouse audit

---

## Design Inspiration

**Study these for visual excellence**:
- [Linear.app](https://linear.app) - Smooth animations, dark theme, professional
- [Vercel.com](https://vercel.com) - Gradients, code showcases, fast
- [Stripe.com](https://stripe.com) - Clean, trustworthy, interactive
- [Rust-lang.org](https://rust-lang.org) - Technical precision, clear CTAs
- [TypeScript homepage](https://typescriptlang.org) - Code comparisons, playgrounds

---

## Next Steps

1. Create new repository: `getlumos/lumos-website`
2. Set up Next.js project: `npx create-next-app@latest`
3. Design review and approval
4. Content review and finalization
5. Development (follow phases 1-5)
6. Deploy to Vercel
7. Point lumos-lang.org to deployment

---

**Priority**: High
**Complexity**: Medium-High
**Impact**: Critical for ecosystem growth
**Timeline**: 3-4 weeks to production
