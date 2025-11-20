# LUMOS Landing Page - lovable.dev Prompt

Build a modern, sleek landing page for **LUMOS** - a type-safe schema language for Solana development.

---

## Brand Identity

**Name**: LUMOS
**Tagline**: "Type-Safe Schemas for Solana"
**Description**: Schema language that generates production-ready Rust and TypeScript with guaranteed Borsh serialization compatibility

**Color Scheme**:
- Primary: Deep Purple (#7C3AED) - represents magic/light
- Secondary: Electric Blue (#3B82F6) - tech/innovation
- Accent: Bright Cyan (#06B6D4) - highlights
- Background: Dark (#0F172A) with subtle gradient
- Text: White (#FFFFFF) and Light Gray (#CBD5E1)

**Vibe**: Modern, technical, magical, professional

---

## Page Structure

### 1. Hero Section (Above the Fold)
**Background**: Dark gradient (navy to black) with subtle grid pattern or animated particles

**Content**:
- Large, bold headline: **"Write Once. Type-Safe Everywhere."**
- Subheadline: "Define Solana schemas in `.lumos` syntax. Get production-ready Rust + TypeScript with perfect Borsh compatibility."
- Two prominent CTAs:
  1. Primary button: "Get Started →" (links to docs.lumos-lang.org/getting-started/quick-start)
  2. Secondary button: "View on GitHub" (links to github.com/getlumos/lumos)
- Badge row showing: "Published on crates.io" | "64/64 Tests Passing" | "Zero Dependencies" | "VSCode Extension Available"

**Visual Element**:
- Split-screen code comparison showing:
  - Left: `.lumos` schema (10 lines, clean)
  - Right: Generated Rust + TypeScript (30+ lines)
- With animation showing generation flow (.lumos → Rust + TS)

---

### 2. Problem Statement (Why LUMOS?)
**Headline**: "Stop Writing Borsh Serialization Twice"

**3-Column Layout**:

**Column 1: ❌ The Old Way**
- Manual Rust structs
- Manual TypeScript interfaces
- Hand-written Borsh schemas
- Type mismatches = runtime errors
- Duplicate maintenance burden

**Column 2: ✨ The LUMOS Way**
- Write schema once in `.lumos`
- Generate both languages automatically
- Guaranteed type synchronization
- Catch errors at compile time
- Single source of truth

**Column 3: ✅ The Result**
- 10x faster development
- Zero serialization bugs
- Perfect Anchor integration
- Production-ready code
- Sleep soundly at night

---

### 3. Live Code Demo (Interactive)
**Headline**: "See It In Action"

**Interactive code editor** (Monaco or CodeMirror):
- Default example: Gaming schema (PlayerAccount with level, xp, items)
- User can edit the `.lumos` schema
- Live preview shows generated Rust and TypeScript side-by-side
- Syntax highlighting for all 3 languages
- Copy button for each output
- Dropdown to switch between examples: Gaming | NFT | DeFi | DAO

**Note**: If interactive is too complex, use static code blocks with tabs

---

### 4. Features Grid
**Headline**: "Everything You Need"

**6 Feature Cards** (2 rows x 3 columns):

1. **⚡ Context-Aware Generation**
   - Detects Anchor vs pure Borsh
   - Automatic import management
   - Smart derive macros

2. **🎯 Full Type Support**
   - Primitives (u8-u128, i8-i128, bool)
   - Solana types (PublicKey, Signature)
   - Complex types (Vec, Option, enums)

3. **🔧 Seamless Anchor Integration**
   - #[account] attribute support
   - Zero manual derives
   - Drop-in replacement

4. **📝 TypeScript + Borsh**
   - Perfect type definitions
   - Automatic Borsh schemas
   - Web3.js compatible

5. **🚀 Production Ready**
   - 64/64 tests passing
   - Published on crates.io
   - Zero known vulnerabilities

6. **🎨 VSCode Extension**
   - Syntax highlighting
   - Auto-completion
   - Error diagnostics
   - Quick fixes

---

### 5. Code Comparison (Before/After)
**Headline**: "From This... To This"

**Side-by-side comparison**:

**Before (Manual Borsh)**:
```rust
// Rust
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Player {
    pub wallet: Pubkey,
    pub level: u16,
    pub xp: u64,
}

// Then separately write TypeScript...
```

```typescript
// TypeScript
import * as borsh from '@coral-xyz/borsh';

export interface Player {
  wallet: PublicKey;
  level: number;
  xp: number;
}

// Manually create Borsh schema... easy to make mistakes!
export const PlayerSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.u16('level'),
  borsh.u64('xp'), // Did you match field order? 🤞
]);
```

**After (LUMOS)**:
```lumos
#[solana]
#[account]
struct Player {
    wallet: PublicKey,
    level: u16,
    xp: u64,
}
```

**Caption**: "One schema. Two languages. Zero mistakes. Run `lumos generate` and you're done."

---

### 6. Stats Section
**Background**: Slightly lighter dark with accent glow

**4 Big Numbers** (horizontal):
1. **64/64** - Tests Passing
2. **0.1.0** - Latest Version
3. **2** - Languages Supported
4. **100%** - Type Safety

---

### 7. Use Cases (Who Is This For?)
**Headline**: "Built For Solana Developers"

**3 Persona Cards**:

1. **🎮 Game Developers**
   - Define player accounts, inventory, match results
   - Generate once, use in program + client
   - Example: Gaming schemas for RPGs and competitive games

2. **💰 DeFi Builders**
   - Staking pools, vesting schedules, token accounts
   - Type-safe across smart contracts and frontends
   - Example: Automated market maker schemas

3. **🏛️ DAO Creators**
   - Governance proposals, voting records, treasury
   - Synchronized types for on-chain + off-chain
   - Example: Multi-sig governance structures

---

### 8. Ecosystem
**Headline**: "Complete Ecosystem"

**4 Cards with icons and links**:

1. **📦 Core Library** (crates.io)
   - Install: `cargo install lumos-cli`
   - Link: crates.io/crates/lumos-cli

2. **📚 Documentation** (docs.lumos-lang.org)
   - Quickstart, guides, API reference
   - Link: docs.lumos-lang.org

3. **🔌 VSCode Extension** (Marketplace)
   - Syntax highlighting, auto-complete, diagnostics
   - Link: marketplace.visualstudio.com/items?itemName=lumos.lumos-vscode

4. **🌟 Examples** (awesome-lumos)
   - Real-world schemas: NFT, DeFi, Gaming, DAO
   - Link: github.com/getlumos/awesome-lumos

---

### 9. Getting Started (Final CTA)
**Background**: Gradient accent (purple to blue)

**Headline**: "Ready to Build?"

**3-Step Process**:
1. **Install** - `cargo install lumos-cli` (30 seconds)
2. **Create** - `lumos init my-project` (30 seconds)
3. **Generate** - `lumos generate schema.lumos` (instant)

**Large CTA Button**: "Start in 5 Minutes →" (links to quickstart)

---

### 10. Footer
**Background**: Darkest (#0A0F1C)

**3 Columns**:

**Column 1: Resources**
- Documentation
- Quickstart Guide
- API Reference
- Examples

**Column 2: Community**
- GitHub
- Discord (if exists)
- Twitter (if exists)
- Report an Issue

**Column 3: Project**
- About LUMOS
- Changelog
- Contributing
- License (MIT)

**Bottom**:
- Copyright © 2025 LUMOS
- "Built with ❤️ for the Solana ecosystem"

---

## Technical Requirements

1. **Framework**: React with TypeScript (or Next.js for better SEO)
2. **Styling**: Tailwind CSS
3. **Animations**: Framer Motion for smooth transitions
4. **Icons**: Lucide React or Heroicons
5. **Code Blocks**: Shiki or Prism for syntax highlighting
6. **Responsive**: Mobile-first, works perfectly on all screen sizes
7. **Performance**: Lighthouse score 90+
8. **Accessibility**: WCAG 2.1 AA compliant

---

## Design Inspiration
- Linear.app (clean, modern, dark theme)
- Vercel.com (gradients, code demos)
- Rust-lang.org (technical, professional)
- TypeScript homepage (split-screen code comparisons)

---

## Copy Tone
- **Technical but accessible** - Assume Solana developer audience
- **Confident, not arrogant** - "Built for production" not "the best ever"
- **Problem-focused** - Lead with pain points, then solution
- **Action-oriented** - Clear CTAs, easy next steps

---

## Must-Have Elements
✅ Dark theme (primary)
✅ Code syntax highlighting
✅ Responsive design
✅ Fast loading (<2s)
✅ Clear CTAs throughout
✅ GitHub link prominent
✅ Installation commands copy-able
✅ Examples visible above fold

---

## Optional Nice-to-Haves
- Animated code generation flow
- Interactive schema playground
- Video demo embed
- Testimonials (add later)
- Blog link (add later)
- Newsletter signup (add later)

---

## After lovable.dev Builds It

We'll:
1. Review and refine
2. Export code to new repo: `getlumos/lumos-website`
3. Deploy to Vercel/Netlify
4. Point lumos-lang.org to deployment
5. Add to navigation from docs site

Let me know when you're ready to paste this into lovable.dev!
