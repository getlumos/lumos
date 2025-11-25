# LUMOS Marketing Website - World-Class Specification

## Vision

Create a **world-class marketing website** for LUMOS that rivals the best developer tool sites (Linear, Vercel, Stripe, Raycast, Warp) while staying authentic to the technical Solana developer audience.

**Mission:** Convert visitors from "What is LUMOS?" to "Holy shit, I need this now" in under 30 seconds.

**Philosophy:** Not minimum viable. Not good enough. **World-class.** Every pixel, every animation, every interaction matters.

---

## Target Audience

- **Primary**: Solana developers (Rust + TypeScript experience)
- **Pain Point**: Manually maintaining Borsh serialization across Rust programs and TypeScript clients
- **Motivation**: Type safety, reduced bugs, faster development
- **Behavior**: Skeptical, code-first, values performance and correctness
- **Expectation**: Professional, fast, no bullshit

---

## Success Metrics

### Performance Targets (World-Class)

| Metric | Target | How We Achieve It |
|--------|--------|-------------------|
| Lighthouse | **100/100** | Astro SSG, optimized assets, zero bloat |
| First Contentful Paint | **<0.8s** | Inline critical CSS, preload fonts, edge CDN |
| Largest Contentful Paint | **<1.2s** | Optimized images, lazy load, priority hints |
| Time to Interactive | **<1.5s** | Minimal JS (islands), code splitting, defer |
| Cumulative Layout Shift | **<0.05** | Reserved space, no layout jumps |
| Total Bundle Size | **<100KB** | Tree-shaking, islands architecture, compression |
| Mobile Speed Index | **<1.5s** | Optimized for 3G networks |

### User Behavior Targets

- ✅ Bounce rate < 35% (industry standard: 40-60%)
- ✅ Average session duration > 3 minutes
- ✅ GitHub CTR (click-through rate) > 20%
- ✅ CLI installation instructions copied > 30%
- ✅ Live demo interaction rate > 40%
- ✅ Mobile traffic engagement = desktop (no degradation)

### Quality Standards

- ✅ WCAG 2.1 **AAA** compliance (not just AA)
- ✅ Mobile responsive perfection (every breakpoint)
- ✅ Cross-browser tested (Chrome, Firefox, Safari, Edge)
- ✅ Screen reader tested (NVDA, VoiceOver, JAWS)
- ✅ Keyboard navigation with visible focus states
- ✅ Reduced motion mode respected

---

## Technology Stack

### Core Framework

**Astro 5.6** - The right tool for world-class marketing sites

**Why Astro over Next.js:**
- Zero JavaScript by default (Next.js ships ~150KB, Astro <50KB)
- Lighthouse 100/100 trivial to achieve
- Island Architecture = best of both worlds (static + interactive)
- View Transitions API (native SPA feel, zero framework overhead)
- Consistency with docs-lumos (same tech stack)
- Faster builds, simpler deployment

**Architecture:**
```astro
---
// Static sections = pure HTML (instant load)
---
<Hero />
<ProblemStatement />
<FeaturesGrid />

<!-- Interactive islands = React only where needed -->
<LiveCodeDemo client:visible />
<BeforeAfterSlider client:idle />
<StatsCounter client:visible />
```

### Languages & Styling

- **TypeScript** (strict mode, dogfooding type safety)
- **Tailwind CSS** (utility-first, dark theme, custom config)
- **React** (islands only, for complex interactions)

### Animation & Interaction Libraries

**Framer Motion** (v11+)
- Sophisticated component animations
- Spring physics for natural motion
- Gesture detection (drag, pan, hover)
- Layout animations (automatic)

**GSAP** (GreenSock) + ScrollTrigger
- Advanced timeline animations
- Scroll-linked animations
- Timeline control (play, pause, reverse)
- Performance-optimized (60fps guaranteed)

**Lenis** (v1.1+)
- Butter-smooth scroll with momentum
- Easing curve tuned for developer tools
- Virtual scroll for infinite performance

**Rough Notation**
- Hand-drawn emphasis effects
- Underline, circle, highlight, strike-through
- Animated reveal on scroll

**Optional (if budget allows):**
- **Three.js** (3D hero background, particle systems)
- **Spline** (3D design tool, easier than Three.js)
- **rive.app** (Animated illustrations, vector animations)

### Code & Syntax

**Shiki** (v1.0+)
- Beautiful syntax highlighting (VSCode themes)
- Support for `.lumos`, Rust, TypeScript
- Custom theme matching LUMOS brand colors
- Line highlighting, diff support

**Monaco Editor** (VSCode engine)
- Full VSCode experience in browser
- IntelliSense, autocomplete, error detection
- Multi-file editing (schema.lumos, generated.rs, generated.ts)
- Minimap, breadcrumbs, command palette

**Prettier** (v3+)
- Format code examples in real-time
- Consistent style across all examples

### Performance & SEO

**Astro Image Optimization**
- Automatic WebP/AVIF conversion
- Responsive images (srcset)
- Lazy loading with blur-up placeholder
- Art direction (different crops per breakpoint)

**Partytown** (v0.10+)
- Run analytics in web worker (off main thread)
- Zero impact on performance
- Google Analytics, Vercel Analytics support

**View Transitions API**
- Native browser API (no framework needed)
- Instant page transitions
- Smooth cross-fade between views
- Fallback for browsers without support

### Deployment & Infrastructure

**Vercel**
- Edge functions (CDN at 300+ locations)
- Automatic CI/CD (push to deploy)
- Preview deployments (every PR)
- Analytics (Core Web Vitals, user behavior)
- A/B testing (optional, Vercel Edge Config)

**Custom Domain:** lumos-lang.org

---

## Page Structure (10 Sections)

### 1. Hero Section - Cinematic Entrance

**Purpose:** Hook developers in 5 seconds with immediate visual impact

#### Elements

**Headline** (animated entrance):
```
Type-Safe Schemas for Solana.
Write Once. Zero Bugs.
```
- Font: Inter 900, 4rem (desktop), 2.5rem (mobile)
- Text gradient: purple → blue → cyan
- Animation: Fade up + blur (300ms ease-out)

**Subheadline** (delayed entrance):
```
Define data structures in .lumos syntax.
Generate production-ready Rust + TypeScript with guaranteed Borsh compatibility.
```
- Font: Inter 400, 1.25rem
- Color: text-secondary
- Animation: Fade up (600ms, delay 300ms)

**Primary CTA** (magnetic effect):
- "Get Started →"
- Gradient button: purple → blue
- Hover: Lift + glow + scale(1.05)
- Click: Ripple effect
- Links to: docs.lumos-lang.org/quickstart

**Secondary CTA** (with live star count):
- "View on GitHub ★ 127"
- Outline button with subtle glow
- Star count updates in real-time (GitHub API)
- Hover: Glow intensifies

**Terminal Demo** (animated typing):
```bash
$ cargo install lumos-cli
  ⠋ Installing lumos-cli v0.1.1...
  ✓ Installed successfully

$ lumos init my-solana-project
  ✓ Created project structure
  ✓ Initialized Git repository

$ lumos generate schema.lumos
  ⠋ Parsing schema.lumos...
  ⠋ Generating Rust code...
  ✓ Generated: src/generated.rs
  ⠋ Generating TypeScript code...
  ✓ Generated: src/generated.ts

  🎉 Done! Your types are ready.
```
- Use **typed.js** or custom React component
- Syntax highlighting with Shiki
- Cursor blink effect
- Auto-loop after 10s

**Visual: Code Transformation Animation**
- Split-screen layout
- Left: `.lumos` schema (7 lines)
- Right: Generated Rust + TypeScript (animated reveal)
- Arrow animation between panels (particles flowing)
- Line-by-line reveal with syntax highlighting

**Trust Badges** (floating with parallax):
```
✅ 142/142 Tests Passing
📦 Published on crates.io
🔒 Zero Vulnerabilities
🎨 VSCode Extension v0.5.0
```
- Glassmorphism effect (backdrop-filter)
- Subtle float animation (different speeds)
- Parallax on scroll (slower than page)

#### Background Options (Choose One)

**Option A: Animated 3D Grid** (Three.js)
- Wireframe grid, subtle rotation
- Responds to mouse movement
- Particle nodes at intersections
- Performance: 60fps, GPU-accelerated

**Option B: Gradient Mesh** (CSS + GSAP)
- Organic, flowing gradient blobs
- Smooth transitions between colors
- Low memory footprint
- Fallback: Static gradient

**Option C: Particle Field** (Three.js or Canvas)
- Stars/dots, parallax on mouse
- Connections between nearby particles
- Subtle color shifts
- Performance: Throttled to 60fps

**Recommendation:** Option B (Gradient Mesh) for best performance/beauty ratio

#### Animation Timeline

```javascript
gsap.timeline()
  .from('.gradient-mesh', { opacity: 0, duration: 1 })
  .from('.headline', { y: 50, opacity: 0, duration: 0.6, ease: 'power3.out' }, 0.3)
  .from('.subheadline', { y: 30, opacity: 0, duration: 0.6, ease: 'power3.out' }, 0.6)
  .from('.cta-group', { y: 20, opacity: 0, duration: 0.5, ease: 'power2.out' }, 0.9)
  .from('.terminal-demo', { scale: 0.95, opacity: 0, duration: 0.7, ease: 'back.out(1.2)' }, 1.2)
  .from('.trust-badges', { opacity: 0, stagger: 0.1, duration: 0.4 }, 1.5);
```

---

### 2. Problem Statement - The Pain

**Headline:** "Stop Writing Borsh Serialization Twice"

**3-Column Comparison Table** (scroll-triggered animation):

| ❌ The Old Way | ✨ The LUMOS Way | ✅ The Result |
|----------------|------------------|---------------|
| Manual Rust structs | Write schema once | 10x faster development |
| Manual TypeScript types | Auto-generate both languages | Zero serialization bugs |
| Hand-written Borsh schemas | Guaranteed synchronization | Perfect Anchor integration |
| Type mismatches = runtime errors | Catch errors at compile time | Production-ready code |
| Duplicate maintenance | Single source of truth | Sleep soundly 😴 |

**Animation:**
- Table rows fade in one-by-one (scroll-triggered)
- Icons pulse on hover
- Mobile: Swipeable cards instead of table

**Styling:**
- Glassmorphism cards
- Border glow on hover
- Checkmarks/X icons animated (Lottie or SVG)

---

### 3. Live Code Demo - Interactive Editor

**Headline:** "See It In Action"

**Purpose:** Let developers experience LUMOS without installing anything

#### Features

**Monaco Editor** (left pane):
- Full VSCode experience
- Syntax highlighting for `.lumos`
- IntelliSense autocomplete
- Error detection (red squiggles)
- Line numbers, minimap, breadcrumbs
- Command palette (Cmd+Shift+P)

**Generated Output** (right pane, tabbed):
- Tab 1: **Rust** (generated.rs)
- Tab 2: **TypeScript** (generated.ts)
- Tab 3: **Borsh Schema** (borsh-schema.ts)
- Smooth tab transitions (Framer Motion)

**Example Selector** (dropdown):
```
Choose an example:
- 🎮 Gaming (Player Account, Inventory)
- 🖼️ NFT (Metadata, Marketplace)
- 💰 DeFi (Staking Pool, Vesting)
- 🏛️ DAO (Governance, Proposals)
- 📝 Custom (start from scratch)
```
- Smooth transition when switching examples
- Preserve user edits in localStorage
- Reset button to restore default

**Toolbar** (top of editor):
- 📋 Copy (animated checkmark feedback)
- 📥 Download `.zip` (generates project structure)
- 🔗 Share (generate shareable URL)
- ⚙️ Settings (theme, font size, minimap)
- ⚡ Format (Prettier)

**Real-Time Generation:**
- Debounced (300ms after typing stops)
- Loading state: Shimmer effect
- Error state: Red border + error message
- Success state: Green checkmark animation

**Advanced Features:**
- Diff view (show changes as user edits)
- Split vertical/horizontal toggle
- Fullscreen mode
- Keyboard shortcuts (match VSCode)
- Auto-save to localStorage

#### Animation

```javascript
// Editor entrance
gsap.from('.code-editor', {
  scrollTrigger: { trigger: '.code-editor', start: 'top 80%' },
  y: 100,
  opacity: 0,
  duration: 0.8,
  ease: 'power3.out'
});

// Tab switching (Framer Motion)
<motion.div
  key={activeTab}
  initial={{ opacity: 0, x: 20 }}
  animate={{ opacity: 1, x: 0 }}
  exit={{ opacity: 0, x: -20 }}
  transition={{ duration: 0.2 }}
>
```

---

### 4. Features Grid - Why LUMOS?

**Headline:** "Everything You Need for Type-Safe Solana Development"

**6 Feature Cards** (3x2 grid, 2x3 on mobile):

#### Card 1: ⚡ Context-Aware Generation
- Detects `#[account]` for Anchor integration
- Smart import management
- Automatic derive macros
- **Icon:** Lightning bolt with pulse animation

#### Card 2: 🎯 Full Type Support
- Primitives: u8-u128, i8-i128, bool, String
- Solana types: PublicKey, Signature
- Complex: Vec<T>, Option<T>, arrays, enums
- **Icon:** Target with animated rings

#### Card 3: 🔧 Seamless Anchor Integration
- Zero manual derives
- Drop-in replacement
- Works with existing programs
- **Icon:** Puzzle pieces connecting

#### Card 4: 📝 TypeScript + Borsh
- Perfect type definitions
- Automatic Borsh schemas
- Web3.js compatible
- **Icon:** Document with code

#### Card 5: 🚀 Production Ready
- 142/142 tests passing
- Published on crates.io v0.1.1
- Battle-tested in real projects
- **Icon:** Rocket launching

#### Card 6: 🎨 VSCode Extension
- Syntax highlighting
- IntelliSense auto-completion
- Error diagnostics
- Quick fixes
- **Icon:** VSCode logo animated

**Card Styling:**
- Glassmorphism (backdrop-filter: blur)
- Border gradient (animated on hover)
- Lift effect: translateY(-8px) + shadow
- Icon: Scale(1.1) + rotate on hover

**Animation:**
```javascript
gsap.from('.feature-card', {
  scrollTrigger: { trigger: '.features-grid', start: 'top 70%' },
  y: 60,
  opacity: 0,
  stagger: 0.1,
  duration: 0.6,
  ease: 'back.out(1.2)'
});
```

---

### 5. Before/After Comparison - Cinematic Reveal

**Headline:** "From This... To This"

**Purpose:** Visceral demonstration of complexity reduction

#### Visual Design

**BEFORE Panel** (left, larger):
```rust
// Rust side (20+ lines)
use borsh::{BorshSerialize, BorshDeserialize};
use anchor_lang::prelude::*;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Player {
    pub wallet: Pubkey,
    pub level: u16,
    pub experience: u64,
    pub inventory: Vec<Pubkey>,
}
```

```typescript
// TypeScript side (separate file, 30+ lines)
import * as borsh from '@coral-xyz/borsh';
import { PublicKey } from '@solana/web3.js';

export interface Player {
  wallet: PublicKey;
  level: number;
  experience: number;
  inventory: PublicKey[];
}

// Manual Borsh schema - easy to make mistakes!
export const PlayerSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.u16('level'),
  borsh.u64('experience'), // Did you match field order? 🤞
  borsh.vec(borsh.publicKey(), 'inventory'),
]);
```

**Caption:** "~50 lines, 2 files, error-prone, manual sync"

**AFTER Panel** (right, smaller, highlighted):
```rust
#[solana]
#[account]
struct Player {
    wallet: PublicKey,
    level: u16,
    experience: u64,
    inventory: [PublicKey],
}
```

**Caption:** "7 lines, 1 file, guaranteed compatible ✨"

**Arrow Between Panels:**
- Animated particles flowing left → right
- "Run `lumos generate`" text
- Terminal icon with sparkles

#### Interactive Features

**Comparison Slider:**
- Drag handle to reveal BEFORE ↔ AFTER
- Smooth animation (Framer Motion drag)
- Mobile: Tap to toggle

**Line Count Animation:**
- Animated counter: 50 → 7
- Percentage reduction: "86% less code"
- Pulse effect on numbers

**Diff Highlighting:**
- Highlight field names in both panels
- Draw connecting lines between matching fields
- Animated reveal (SVG line drawing)

#### Animation

```javascript
gsap.timeline({
  scrollTrigger: { trigger: '.before-after', start: 'top 60%', end: 'bottom 20%', scrub: 1 }
})
  .from('.before-panel', { x: -100, opacity: 0 })
  .from('.after-panel', { x: 100, opacity: 0 }, 0)
  .from('.arrow-animation', { scale: 0, opacity: 0, ease: 'elastic.out(1, 0.5)' }, 0.5);
```

---

### 6. Use Cases - Who Is This For?

**Headline:** "Built for the Solana Ecosystem"

**4 Persona Cards** (2x2 grid):

#### Card 1: 🎮 Game Developers
- Player accounts, inventory systems, match results
- Real-time multiplayer state synchronization
- Example: 53 types from awesome-lumos/gaming
- **CTA:** "View Gaming Examples →"

#### Card 2: 💰 DeFi Builders
- Staking pools, vesting schedules, AMM state
- Type-safe across smart contracts + frontends
- Example: Token vesting, liquidity pools
- **CTA:** "View DeFi Examples →"

#### Card 3: 🖼️ NFT Creators
- Metadata schemas, marketplace state
- Collection management, royalty tracking
- Example: NFT marketplace with 12 types
- **CTA:** "View NFT Examples →"

#### Card 4: 🏛️ DAO Developers
- Governance proposals, voting records, treasury
- Multi-sig workflows, member management
- Example: DAO governance with 8 instruction types
- **CTA:** "View DAO Examples →"

**Card Styling:**
- Large icon (animated Lottie or SVG)
- Glassmorphism background
- Hover: Icon bounces, card glows
- Border gradient (matches icon color)

**Animation:**
```javascript
gsap.from('.use-case-card', {
  scrollTrigger: { trigger: '.use-cases', start: 'top 70%' },
  scale: 0.9,
  opacity: 0,
  stagger: 0.15,
  duration: 0.7,
  ease: 'power3.out'
});
```

---

### 7. Stats Section - Social Proof

**Background:** Gradient mesh with glow effect (purple → blue)

**4 Big Numbers** (animated count-up on scroll):

```
┌──────────────┬──────────────┬──────────────┬──────────────┐
│    142/142   │    v0.1.1    │      2       │     100%     │
│     Tests    │   Latest     │  Languages   │ Type Safety  │
│    Passing   │   Stable     │  Supported   │  Guaranteed  │
└──────────────┴──────────────┴──────────────┴──────────────┘
```

**Animation:**
- Count-up effect (0 → final number)
- Easing: ease-out (fast start, slow end)
- Trigger: When section enters viewport
- Duration: 1.5s per number
- Stagger: 0.2s between numbers

**Additional Stats** (smaller, below):
- "🚀 Used in 50+ projects"
- "⭐ 127 GitHub stars"
- "📦 10K+ crates.io downloads"
- "🎨 VSCode extension: 5K+ installs"

**Styling:**
- Numbers: Inter 900, 5rem
- Labels: Inter 400, 1rem
- Glow effect around numbers
- Pulse animation on hover

---

### 8. Ecosystem Overview

**Headline:** "Complete Developer Experience"

**4 Ecosystem Cards** (2x2 grid, links to resources):

#### Card 1: 📦 CLI Tool
- `cargo install lumos-cli`
- Commands: generate, validate, init, check, diff
- **Badge:** v0.1.1 on crates.io
- **CTA:** [View on crates.io →]

#### Card 2: 📚 Documentation
- Quickstart guide (5 minutes)
- API reference, migration guides
- **Badge:** Comprehensive tutorials
- **CTA:** [Read the docs →]

#### Card 3: 🔌 VSCode Extension
- Syntax highlighting, snippets
- Real-time validation, quick fixes
- **Badge:** v0.5.0 on Marketplace
- **CTA:** [Install extension →]

#### Card 4: 🌟 Examples
- 5 production-ready templates
- Gaming, DeFi, NFT, DAO, Vesting
- **Badge:** 53+ types, 42+ instructions
- **CTA:** [Browse examples →]

**Card Styling:**
- Icon: Large, colorful, animated
- Hover: Lift + glow + scale
- Badge: Pill shape, subtle pulse
- Arrow on CTA: Slide right on hover

**Animation:**
```javascript
gsap.from('.ecosystem-card', {
  scrollTrigger: { trigger: '.ecosystem', start: 'top 70%' },
  y: 80,
  opacity: 0,
  stagger: 0.12,
  duration: 0.8,
  ease: 'power3.out'
});
```

---

### 9. Getting Started - Final CTA

**Background:** Bold gradient (purple → blue → cyan) with glow

**Headline:** "Start Building in 5 Minutes"

**3-Step Visual Process** (horizontal timeline):

```
1️⃣ Install            2️⃣ Create             3️⃣ Generate
━━━━━━━━━━━━━━━       ━━━━━━━━━━━━━━━       ━━━━━━━━━━━━━━━
$ cargo install       $ lumos init          $ lumos generate
  lumos-cli            my-project            schema.lumos

  30 seconds           30 seconds            Instant ⚡
```

**Visual Design:**
- Timeline with animated connecting line
- Step numbers in gradient circles
- Terminal snippets with syntax highlighting
- Time estimates (animated counters)

**Large CTA Button:**
- "Get Started Now →"
- Size: 200px x 60px (large!)
- Gradient: purple → blue
- Hover: Glow intensifies + scale(1.05)
- Click: Ripple effect + haptic feedback (mobile)

**Secondary Link:**
- "Or view the quickstart guide"
- Underline animation on hover

**Animation:**
```javascript
gsap.from('.timeline-step', {
  scrollTrigger: { trigger: '.getting-started', start: 'top 70%' },
  scale: 0.8,
  opacity: 0,
  stagger: 0.2,
  duration: 0.7,
  ease: 'back.out(1.5)'
});

// Connecting line draws in
gsap.from('.timeline-line', {
  scrollTrigger: { trigger: '.getting-started', start: 'top 70%' },
  scaleX: 0,
  transformOrigin: 'left center',
  duration: 1.5,
  ease: 'power3.inOut'
});
```

---

### 10. Footer

**Background:** Darkest shade (#0A0F1C) with subtle noise texture

**Layout:** 4 columns (stack on mobile)

#### Column 1: Resources
- Documentation
- Quickstart Guide
- API Reference
- Migration Guide
- Changelog

#### Column 2: Community
- GitHub ⭐ (with star count)
- Report an Issue
- Contributing Guide
- Code of Conduct
- Discussions

#### Column 3: Examples
- awesome-lumos
- Gaming Examples
- DeFi Examples
- NFT Examples
- DAO Examples

#### Column 4: Ecosystem
- VSCode Extension
- IntelliJ Plugin
- Neovim Plugin
- Emacs Mode
- Sublime Text Package

**Bottom Bar:**
```
┌──────────────────────────────────────────────────────────┐
│ Copyright © 2025 LUMOS  •  Built with ❤️ for Solana      │
│ Dual-licensed: MIT + Apache 2.0                          │
└──────────────────────────────────────────────────────────┘
```

**Social Links** (icons only):
- GitHub
- Twitter/X
- Discord
- YouTube

**Styling:**
- Link hover: Underline animation (left → right)
- Social icons: Glow on hover
- Logo: Subtle pulse animation

---

## Design System

### Color Palette (Dark Theme)

**Primary Brand Colors:**
```css
--lumos-purple: #7C3AED;    /* Primary CTA, headings */
--lumos-blue: #3B82F6;      /* Secondary accents */
--lumos-cyan: #06B6D4;      /* Highlights, hover states */
--lumos-pink: #EC4899;      /* Accent, warnings */

/* Gradients */
--gradient-primary: linear-gradient(135deg, #7C3AED 0%, #3B82F6 50%, #06B6D4 100%);
--gradient-mesh: radial-gradient(at 20% 30%, #7C3AED 0%, transparent 50%),
                 radial-gradient(at 80% 70%, #3B82F6 0%, transparent 50%),
                 radial-gradient(at 40% 80%, #06B6D4 0%, transparent 50%);
```

**Backgrounds:**
```css
--bg-darkest: #0A0F1C;      /* Footer, deep sections */
--bg-dark: #0F172A;         /* Main background */
--bg-card: #1E293B;         /* Card backgrounds */
--bg-elevated: #334155;     /* Elevated elements */
--bg-glass: rgba(30, 41, 59, 0.8);  /* Glassmorphism */
```

**Text:**
```css
--text-primary: #FFFFFF;    /* Headings */
--text-secondary: #CBD5E1;  /* Body text */
--text-muted: #94A3B8;      /* Captions, labels */
--text-gradient: linear-gradient(135deg, #FFFFFF 0%, #CBD5E1 100%);
```

**Borders:**
```css
--border-default: rgba(148, 163, 184, 0.2);
--border-hover: rgba(148, 163, 184, 0.4);
--border-gradient: linear-gradient(135deg, #7C3AED, #3B82F6, #06B6D4);
```

**Shadows:**
```css
--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.3);
--shadow-md: 0 4px 6px rgba(0, 0, 0, 0.4);
--shadow-lg: 0 10px 20px rgba(0, 0, 0, 0.5);
--shadow-xl: 0 20px 40px rgba(0, 0, 0, 0.6);
--shadow-glow: 0 0 20px rgba(124, 58, 237, 0.5);  /* Purple glow */
```

### Typography

**Font Families:**
```css
--font-display: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
--font-body: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
--font-code: 'JetBrains Mono', 'Fira Code', monospace;
```

**Font Weights:**
- Display: 900 (black, for hero headlines)
- Headings: 700-800 (bold, semibold)
- Body: 400-500 (regular, medium)
- Code: 400-500 (ligatures enabled)

**Type Scale** (fluid typography):
```css
/* Using clamp() for responsive sizing */
--text-xs: clamp(0.75rem, 0.7rem + 0.25vw, 0.875rem);
--text-sm: clamp(0.875rem, 0.8rem + 0.375vw, 1rem);
--text-base: clamp(1rem, 0.9rem + 0.5vw, 1.125rem);
--text-lg: clamp(1.125rem, 1rem + 0.625vw, 1.25rem);
--text-xl: clamp(1.25rem, 1.1rem + 0.75vw, 1.5rem);
--text-2xl: clamp(1.5rem, 1.3rem + 1vw, 1.875rem);
--text-3xl: clamp(1.875rem, 1.6rem + 1.375vw, 2.25rem);
--text-4xl: clamp(2.25rem, 1.9rem + 1.75vw, 3rem);
--text-5xl: clamp(3rem, 2.5rem + 2.5vw, 4rem);
```

**Line Heights:**
- Display: 1.1 (tight, for large headlines)
- Headings: 1.2
- Body: 1.6 (comfortable reading)
- Code: 1.5

**Letter Spacing:**
- Display: -0.02em (tight tracking)
- Headings: -0.01em
- Body: 0
- Code: 0 (preserve monospace alignment)

### Spacing System

**8-point grid system:**
```css
--space-1: 0.5rem;   /* 8px */
--space-2: 1rem;     /* 16px */
--space-3: 1.5rem;   /* 24px */
--space-4: 2rem;     /* 32px */
--space-5: 2.5rem;   /* 40px */
--space-6: 3rem;     /* 48px */
--space-8: 4rem;     /* 64px */
--space-10: 5rem;    /* 80px */
--space-12: 6rem;    /* 96px */
--space-16: 8rem;    /* 128px */
--space-20: 10rem;   /* 160px */
```

### Border Radius

```css
--radius-sm: 0.25rem;   /* 4px */
--radius-md: 0.5rem;    /* 8px */
--radius-lg: 0.75rem;   /* 12px */
--radius-xl: 1rem;      /* 16px */
--radius-2xl: 1.5rem;   /* 24px */
--radius-full: 9999px;  /* Pills, circles */
```

### Animation Principles

**Timing Functions:**
```css
--ease-in: cubic-bezier(0.4, 0, 1, 1);
--ease-out: cubic-bezier(0, 0, 0.2, 1);
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
--ease-spring: cubic-bezier(0.34, 1.56, 0.64, 1);  /* Bounce */
--ease-elastic: cubic-bezier(0.68, -0.55, 0.265, 1.55);
```

**Duration Standards:**
- Micro-interactions: 150ms (hover, focus)
- Small animations: 250ms (cards, buttons)
- Medium animations: 400ms (modals, panels)
- Large animations: 600ms (page sections)
- Cinematic: 1000ms+ (hero entrance)

**Motion Principles:**
- **Purposeful:** Every animation serves a function (guide attention, provide feedback)
- **Natural:** Use physics-based easing (spring, elastic)
- **Performant:** Animate only transform and opacity (GPU-accelerated)
- **Respectful:** Honor prefers-reduced-motion
- **Consistent:** Same durations for similar elements

**Animation Choreography:**
- Stagger: 50-100ms between elements
- Overlap: Start next animation before previous finishes (15-20% overlap)
- Direction: Animate in from direction of user attention

---

## Micro-Interactions

### Hover States

**Buttons:**
```css
.button {
  transition: all 250ms var(--ease-out);
}
.button:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-lg), var(--shadow-glow);
  filter: brightness(1.1);
}
```

**Cards:**
```css
.card {
  transition: all 300ms var(--ease-out);
}
.card:hover {
  transform: translateY(-8px);
  box-shadow: var(--shadow-xl);
  border-color: var(--border-hover);
}
```

**Links:**
```css
.link {
  position: relative;
  text-decoration: none;
}
.link::after {
  content: '';
  position: absolute;
  bottom: -2px;
  left: 0;
  width: 0;
  height: 2px;
  background: var(--gradient-primary);
  transition: width 250ms var(--ease-out);
}
.link:hover::after {
  width: 100%;
}
```

### Focus States

**Keyboard Navigation:**
```css
*:focus-visible {
  outline: 2px solid var(--lumos-purple);
  outline-offset: 4px;
  border-radius: var(--radius-md);
}
```

**Skip to Content Link:**
```css
.skip-to-content {
  position: absolute;
  top: -100px;
  left: 50%;
  transform: translateX(-50%);
  transition: top 200ms var(--ease-out);
}
.skip-to-content:focus {
  top: var(--space-4);
}
```

### Loading States

**Skeleton Screens:**
```css
.skeleton {
  background: linear-gradient(
    90deg,
    var(--bg-card) 0%,
    var(--bg-elevated) 50%,
    var(--bg-card) 100%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}
```

**Spinners:**
```css
.spinner {
  border: 3px solid var(--bg-elevated);
  border-top-color: var(--lumos-purple);
  border-radius: var(--radius-full);
  animation: spin 600ms linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
```

### Success/Error Feedback

**Copy Button:**
```tsx
<button onClick={handleCopy}>
  {copied ? (
    <motion.div
      initial={{ scale: 0 }}
      animate={{ scale: 1 }}
      transition={{ type: 'spring', stiffness: 500 }}
    >
      ✓ Copied!
    </motion.div>
  ) : (
    'Copy'
  )}
</button>
```

**Form Validation:**
```css
.input.error {
  border-color: #EF4444;
  animation: shake 400ms;
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-10px); }
  75% { transform: translateX(10px); }
}
```

---

## Cursor Effects

### Custom Cursor

**Large cursor with gradient trail:**
```tsx
<motion.div
  className="custom-cursor"
  animate={{ x: mouseX, y: mouseY }}
  transition={{ type: 'spring', stiffness: 500, damping: 28 }}
>
  <div className="cursor-dot" />
  <div className="cursor-ring" />
</motion.div>
```

```css
.custom-cursor {
  position: fixed;
  pointer-events: none;
  z-index: 9999;
  mix-blend-mode: difference;
}

.cursor-dot {
  width: 8px;
  height: 8px;
  background: white;
  border-radius: 50%;
}

.cursor-ring {
  width: 40px;
  height: 40px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 50%;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  transition: width 200ms, height 200ms;
}

/* Enlarge on hover over interactive elements */
.custom-cursor.hover .cursor-ring {
  width: 60px;
  height: 60px;
}
```

### Magnetic Effect

**Buttons attract cursor on proximity:**
```tsx
function MagneticButton({ children }) {
  const ref = useRef(null);

  const handleMouseMove = (e) => {
    const { left, top, width, height } = ref.current.getBoundingClientRect();
    const x = (e.clientX - left - width / 2) * 0.3;
    const y = (e.clientY - top - height / 2) * 0.3;

    gsap.to(ref.current, {
      x, y,
      duration: 0.3,
      ease: 'power2.out'
    });
  };

  const handleMouseLeave = () => {
    gsap.to(ref.current, { x: 0, y: 0, duration: 0.5, ease: 'elastic.out(1, 0.3)' });
  };

  return (
    <button
      ref={ref}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
    >
      {children}
    </button>
  );
}
```

---

## Scroll Animations

### Lenis Smooth Scroll

**Configuration:**
```tsx
import Lenis from '@studio-freight/lenis';

const lenis = new Lenis({
  duration: 1.2,
  easing: (t) => Math.min(1, 1.001 - Math.pow(2, -10 * t)),
  smooth: true,
  smoothTouch: false, // Disable on mobile for native feel
});

function raf(time) {
  lenis.raf(time);
  requestAnimationFrame(raf);
}
requestAnimationFrame(raf);
```

### GSAP ScrollTrigger

**Fade in sections as you scroll:**
```javascript
gsap.utils.toArray('section').forEach((section) => {
  gsap.from(section, {
    opacity: 0,
    y: 100,
    duration: 0.8,
    ease: 'power3.out',
    scrollTrigger: {
      trigger: section,
      start: 'top 80%',
      end: 'top 50%',
      toggleActions: 'play none none reverse',
    },
  });
});
```

**Parallax background:**
```javascript
gsap.to('.hero-background', {
  y: '50%',
  ease: 'none',
  scrollTrigger: {
    trigger: '.hero',
    start: 'top top',
    end: 'bottom top',
    scrub: true,
  },
});
```

**Pinned section:**
```javascript
ScrollTrigger.create({
  trigger: '.code-demo',
  start: 'top top',
  end: '+=1000',
  pin: true,
  pinSpacing: true,
});
```

---

## Accessibility

### WCAG 2.1 AAA Compliance

**Color Contrast:**
- Normal text: 7:1 (AAA standard)
- Large text (18pt+): 4.5:1
- UI components: 4.5:1
- Test with: [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)

**Keyboard Navigation:**
- Tab order follows visual order
- All interactive elements are keyboard accessible
- Focus visible on all elements
- Skip to main content link

**Screen Readers:**
- Semantic HTML5 elements
- ARIA labels on complex components
- `aria-live` regions for dynamic content
- Alt text on all images (descriptive, not redundant)

**Motion:**
```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

**Focus Management:**
```tsx
// Trap focus in modals
import { useFocusTrap } from '@/hooks/useFocusTrap';

function Modal({ isOpen, children }) {
  const ref = useFocusTrap(isOpen);

  return (
    <div ref={ref} role="dialog" aria-modal="true">
      {children}
    </div>
  );
}
```

---

## Performance Optimization

### Image Optimization

**Astro Image Component:**
```astro
---
import { Image } from 'astro:assets';
import heroImage from '@/assets/hero.png';
---

<Image
  src={heroImage}
  alt="LUMOS code generation"
  width={1200}
  height={600}
  loading="lazy"
  format="webp"
  quality={80}
/>
```

**Responsive Images:**
```astro
<Image
  src={heroImage}
  alt="LUMOS"
  widths={[400, 800, 1200, 1600]}
  sizes="(max-width: 640px) 400px, (max-width: 1024px) 800px, 1200px"
/>
```

### Code Splitting

**Dynamic Imports:**
```tsx
const LiveCodeDemo = lazy(() => import('@/components/LiveCodeDemo'));

<Suspense fallback={<Skeleton />}>
  <LiveCodeDemo />
</Suspense>
```

**Bundle Analysis:**
```bash
npm run build
npx astro build --analyze
```

### Font Loading

**Self-hosted fonts with preload:**
```html
<link rel="preload" href="/fonts/Inter-Regular.woff2" as="font" type="font/woff2" crossorigin>
<link rel="preload" href="/fonts/JetBrainsMono-Regular.woff2" as="font" type="font/woff2" crossorigin>
```

**Font Display Strategy:**
```css
@font-face {
  font-family: 'Inter';
  src: url('/fonts/Inter-Regular.woff2') format('woff2');
  font-display: swap; /* Show fallback immediately, swap when loaded */
}
```

### Critical CSS

**Inline critical CSS in `<head>`:**
```astro
<style is:inline>
  /* Critical styles for above-the-fold content */
  body { background: #0F172A; color: #FFFFFF; }
  .hero { min-height: 100vh; }
</style>
```

### Resource Hints

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="dns-prefetch" href="https://api.github.com">
<link rel="prefetch" href="/docs">
```

---

## Repository Structure

```
lumos-website/
├── src/
│   ├── pages/
│   │   └── index.astro               # Main landing page
│   ├── layouts/
│   │   └── BaseLayout.astro          # Global layout (head, scripts)
│   ├── components/
│   │   ├── Hero.astro                # Static hero section
│   │   ├── ProblemStatement.astro    # Static comparison table
│   │   ├── LiveCodeDemo.tsx          # React island (Monaco Editor)
│   │   ├── FeaturesGrid.astro        # Static grid of 6 features
│   │   ├── BeforeAfter.tsx           # React island (interactive slider)
│   │   ├── UseCases.astro            # Static persona cards
│   │   ├── Stats.tsx                 # React island (animated counters)
│   │   ├── Ecosystem.astro           # Static ecosystem cards
│   │   ├── GetStarted.astro          # Final CTA section
│   │   └── Footer.astro              # Static footer
│   ├── animations/
│   │   ├── scroll-animations.ts      # GSAP ScrollTrigger setup
│   │   ├── hero-animations.ts        # Hero entrance timeline
│   │   ├── cursor-effects.ts         # Custom cursor logic
│   │   └── lenis-setup.ts            # Smooth scroll configuration
│   ├── styles/
│   │   ├── global.css                # Tailwind + custom styles
│   │   ├── animations.css            # Reusable animation keyframes
│   │   └── variables.css             # CSS custom properties
│   ├── utils/
│   │   ├── monaco-setup.ts           # Monaco Editor configuration
│   │   ├── lumos-generator.ts        # Mock code generation logic
│   │   └── github-api.ts             # Fetch star count
│   └── assets/
│       ├── images/                   # Optimized images
│       └── videos/                   # Hero background (optional)
├── public/
│   ├── fonts/
│   │   ├── Inter-Regular.woff2
│   │   ├── Inter-Bold.woff2
│   │   ├── Inter-Black.woff2
│   │   └── JetBrainsMono-Regular.woff2
│   ├── favicon.ico
│   └── robots.txt
├── astro.config.mjs                  # Astro configuration
├── tailwind.config.js                # Tailwind configuration
├── tsconfig.json                     # TypeScript configuration
├── package.json
└── README.md
```

---

## Development Phases

### Phase 1: Foundation (Week 1)

**Goals:** Set up project structure, design system, basic page shell

**Tasks:**
- [ ] Create repository: `getlumos/lumos-website`
- [ ] Initialize Astro 5.6 project
- [ ] Configure TypeScript (strict mode)
- [ ] Set up Tailwind CSS (dark theme)
- [ ] Install dependencies:
  - Framer Motion
  - GSAP + ScrollTrigger
  - Lenis
  - Shiki
  - Monaco Editor
  - Lucide React (icons)
- [ ] Design system implementation:
  - CSS variables (colors, spacing, typography)
  - Tailwind config (custom theme)
  - Font loading (Inter, JetBrains Mono)
- [ ] Basic page structure (10 sections, static)
- [ ] BaseLayout component (head, scripts)

**Deliverable:** Static page with all 10 sections (no animations yet)

---

### Phase 2: Static Sections (Week 2)

**Goals:** Build all static content sections with styling

**Tasks:**
- [ ] Hero section (static shell)
  - Headline + subheadline
  - CTAs (unstyled buttons)
  - Trust badges
  - Terminal demo placeholder
- [ ] Problem Statement
  - 3-column comparison table
  - Icons + styling
- [ ] Features Grid
  - 6 feature cards
  - Icons + descriptions
- [ ] Use Cases
  - 4 persona cards
  - Links to examples
- [ ] Stats Section
  - 4 big numbers (static, no animation)
- [ ] Ecosystem
  - 4 ecosystem cards
  - External links
- [ ] Getting Started
  - 3-step timeline (static)
  - CTA button
- [ ] Footer
  - 4 columns of links
  - Social icons

**Deliverable:** Fully styled static website (mobile responsive)

---

### Phase 3: Interactive Islands (Week 3)

**Goals:** Build React islands for complex interactions

**Tasks:**
- [ ] Live Code Demo (Monaco Editor)
  - Monaco setup + configuration
  - Schema → Rust/TS generation (mock or real)
  - Example selector (4 examples)
  - Copy button with feedback
  - Download button (generate .zip)
- [ ] Before/After Comparison
  - Interactive slider (drag to reveal)
  - Line count animation
  - Diff highlighting
- [ ] Stats Counter
  - Animated count-up (0 → final number)
  - Trigger on scroll into viewport
- [ ] GitHub Star Count
  - Fetch from GitHub API
  - Display in hero + footer

**Deliverable:** Interactive features working (no advanced animations)

---

### Phase 4: Advanced Animations (Week 4)

**Goals:** Add GSAP, Lenis, Framer Motion animations

**Tasks:**
- [ ] Lenis smooth scroll
  - Configuration (duration, easing)
  - Integration with GSAP ScrollTrigger
- [ ] Hero entrance timeline
  - Gradient mesh fade in
  - Headline slide up + blur fade
  - CTAs slide in + bounce
  - Terminal typing animation
- [ ] Scroll-triggered animations
  - Sections fade/slide in
  - Staggered card animations
  - Parallax effects
- [ ] Micro-interactions
  - Button hover states (lift + glow)
  - Card hover states (lift + scale)
  - Link underline animations
- [ ] Cursor effects (optional)
  - Custom cursor with trail
  - Magnetic effect on CTAs
- [ ] View Transitions API
  - Navigation between sections
  - Smooth cross-fade

**Deliverable:** Fully animated website (60fps smooth)

---

### Phase 5: Polish & Optimization (Week 5)

**Goals:** Achieve 100/100 Lighthouse, perfect accessibility

**Tasks:**
- [ ] Performance optimization
  - Code splitting (dynamic imports)
  - Image optimization (WebP, responsive)
  - Font preloading
  - Bundle size analysis
  - Lazy load below-the-fold content
- [ ] Accessibility audit
  - Keyboard navigation testing
  - Screen reader testing (NVDA, VoiceOver)
  - Color contrast validation
  - ARIA labels on complex components
  - Reduced motion mode
- [ ] Cross-browser testing
  - Chrome, Firefox, Safari, Edge
  - Desktop + mobile
  - Fix browser-specific bugs
- [ ] Mobile perfection
  - Touch interactions (swipe, tap)
  - Bottom sheet for CTAs
  - Optimized animations (reduced on mobile)
- [ ] 3D effects (optional, if time allows)
  - Three.js hero background (animated grid)
  - Particle system
  - Performance budget: <20ms frame time

**Deliverable:** Production-ready website (100/100 Lighthouse)

---

### Phase 6: Deployment & Analytics (Week 6)

**Goals:** Deploy to production, set up analytics

**Tasks:**
- [ ] Vercel deployment
  - Connect GitHub repository
  - Configure build settings
  - Environment variables (API keys)
- [ ] Custom domain configuration
  - Point lumos-lang.org to Vercel
  - SSL certificate (automatic)
  - DNS propagation
- [ ] Analytics integration
  - Vercel Analytics (Core Web Vitals)
  - Partytown (run in web worker)
  - Custom events (CTA clicks, demo interactions)
- [ ] SEO optimization
  - OpenGraph tags
  - Twitter Cards
  - Schema.org structured data
  - Sitemap.xml
  - Robots.txt
- [ ] Final audits
  - Lighthouse CI (100/100 on production)
  - Accessibility scan (automated + manual)
  - Security headers (CSP, HSTS)
- [ ] Handoff documentation
  - Update CLAUDE.md
  - Write deployment guide
  - Document custom animations
  - Content update guide

**Deliverable:** Live production website + documentation

---

## Design Inspiration

**Study these sites for world-class quality:**

1. **[Linear.app](https://linear.app)** ⭐ THE gold standard
   - Every pixel perfect, animations purposeful
   - Dark theme, gradient accents, smooth transitions
   - Subtle 3D effects, depth, layering

2. **[Vercel.com](https://vercel.com)** ⚡ Performance obsession
   - Gradients everywhere, code showcases
   - Fast as hell (Lighthouse 100/100)
   - Edge functions, instant navigation

3. **[Stripe.com](https://stripe.com)** 🎯 Clean, trustworthy
   - Interactive demos, animated diagrams
   - Glassmorphism, depth, polish
   - API documentation UX is unmatched

4. **[Raycast.com](https://raycast.com)** 🚀 Developer-focused
   - Command palette, keyboard shortcuts
   - Slick animations, modern design
   - Feels like a native app

5. **[Warp.dev](https://warp.dev)** 💻 Terminal animations
   - Beautiful code blocks, syntax highlighting
   - Animated terminals, flow diagrams
   - Developer appeal, technical precision

6. **[Framer.com](https://framer.com)** 🎨 Animation mastery
   - Every interaction is delightful
   - Motion design showcase
   - Sophisticated physics-based animations

**Action:** Screenshot these sites. Analyze every detail. We're playing at this level.

---

## Next Steps

1. ✅ **Create repository:** `getlumos/lumos-website`
2. Set up Astro 5.6 project with all dependencies
3. Design review: Finalize color palette, typography, animation principles
4. Content review: Finalize copy for all sections
5. Development: Follow phases 1-6 (6 weeks)
6. Deploy to Vercel staging environment
7. User testing + feedback (internal team)
8. Deploy to production (lumos-lang.org)
9. Monitor analytics, iterate based on data

---

## Success Criteria

**Before Launch:**
- [ ] Lighthouse score 100/100 on all pages
- [ ] WCAG 2.1 AAA accessibility (automated + manual testing)
- [ ] Cross-browser tested (Chrome, Firefox, Safari, Edge)
- [ ] Mobile perfection (touch interactions, performance)
- [ ] All animations at 60fps (no jank)
- [ ] Content reviewed and approved
- [ ] SEO optimized (OpenGraph, Twitter Cards, structured data)

**After Launch:**
- [ ] Bounce rate < 35%
- [ ] Average session duration > 3 minutes
- [ ] GitHub CTR > 20%
- [ ] CLI installation instructions copied > 30%
- [ ] Live demo interaction rate > 40%

**If we hit these numbers, we've built a world-class marketing site.**

---

**Timeline:** 6 weeks to world-class production site
**Effort:** 1 developer full-time (with design/content support)
**Impact:** Critical - this is our first impression to the world
**Philosophy:** Not minimum viable. Not good enough. **World-class.**

**Remember:** We're not building a website. We're building an experience that makes developers say "Holy shit, I need this."

---

**Last Updated:** 2025-11-25
**Status:** Ready for implementation
**Next Action:** Create getlumos/lumos-website repository
