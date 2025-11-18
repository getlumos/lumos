# LUMOS Language Recognition Materials

This directory contains all materials needed to get LUMOS recognized as a programming language on GitHub.

## Current Status

**Phase 1: Immediate Solution ✅ COMPLETE**
- `.gitattributes` file created in repository root
- LUMOS files currently classified as Rust on GitHub
- This provides basic recognition until official Linguist support

**Phase 2: Linguist Submission Preparation ✅ COMPLETE**
- TextMate grammar created (`grammars/lumos.tmLanguage.json`)
- Language definition prepared (`languages.yml`)
- Sample files collected (`samples/`)
- Submission guide documented

**Phase 3: Official GitHub Linguist Support ⏳ PENDING**
- Requires: 200+ repositories using LUMOS, OR strong domain-specific justification
- Action: Submit PR to github-linguist/linguist when ready

## Directory Structure

```
linguist/
├── grammars/
│   └── lumos.tmLanguage.json       # Syntax highlighting grammar
├── samples/
│   ├── enums.lumos                 # Enum example
│   └── gaming.lumos                # Gaming example
├── docs/
│   ├── README.md                   # This file
│   └── LINGUIST_SUBMISSION_GUIDE.md # Detailed submission instructions
└── languages.yml                   # Language definition for Linguist
```

## What's Implemented

### 1. TextMate Grammar (`grammars/lumos.tmLanguage.json`)

Provides syntax highlighting for:
- **Comments:** `//` line comments and `/* */` block comments
- **Attributes:** `#[solana]`, `#[account]`, `#[derive(...)]`
- **Keywords:** `struct`, `enum`, `pub`, `use`, etc.
- **Types:** Primitives (`u64`, `String`), Solana types (`PublicKey`), Compounds (`Vec`, `Option`)

### 2. Language Definition (`languages.yml`)

Defines LUMOS for GitHub Linguist:
- **Type:** Programming language
- **Color:** `#8B5CF6` (purple - represents type-safety and cross-language nature)
- **Extensions:** `.lumos`
- **Scope:** `source.lumos`

### 3. Sample Files (`samples/`)

Representative LUMOS code demonstrating:
- Struct definitions with `#[account]`
- Enum support (unit, tuple, struct variants)
- Solana-specific types (`PublicKey`, `Signature`)
- Type annotations and comments

## How to Use These Materials

### Immediate: Apply `.gitattributes`

Already applied in repo root:
```bash
git add .gitattributes
git commit -m "feat: Add .gitattributes for LUMOS language recognition"
git push
```

GitHub will now recognize `.lumos` files (classified as Rust until official support).

### Future: Submit to GitHub Linguist

When LUMOS gains adoption (200+ repos or strong domain justification):

1. **Follow submission guide:** `docs/LINGUIST_SUBMISSION_GUIDE.md`
2. **Fork Linguist:** https://github.com/github-linguist/linguist
3. **Copy materials:**
   - `grammars/lumos.tmLanguage.json` → `grammars/`
   - `languages.yml` entry → `lib/linguist/languages.yml`
   - `samples/*.lumos` → `samples/LUMOS/`
4. **Run tests:** `bundle exec rake test`
5. **Submit PR:** Follow guide in `LINGUIST_SUBMISSION_GUIDE.md`

## Testing Syntax Highlighting Locally

### VSCode (Manual Test)

1. Create `.vscode/extensions/lumos/` in your workspace
2. Copy `grammars/lumos.tmLanguage.json` there
3. Create `package.json`:
```json
{
  "name": "lumos-syntax",
  "version": "0.1.0",
  "engines": { "vscode": "^1.50.0" },
  "contributes": {
    "languages": [{
      "id": "lumos",
      "extensions": [".lumos"],
      "configuration": "./language-configuration.json"
    }],
    "grammars": [{
      "language": "lumos",
      "scopeName": "source.lumos",
      "path": "./grammars/lumos.tmLanguage.json"
    }]
  }
}
```
4. Reload VSCode

### GitHub (After Merge)

Once Linguist PR is merged:
- Syntax highlighting on GitHub.com
- Language stats in repository
- Searchable by language filter

## Adoption Strategy

To meet Linguist's 200-repository threshold:

1. **Build Developer Tools:**
   - VSCode extension with IntelliSense
   - Language server (LSP)
   - CLI improvements

2. **Community Engagement:**
   - Publish to crates.io and npm
   - Write tutorials and blog posts
   - Present at Solana conferences
   - Create starter templates

3. **Showcase Projects:**
   - Build real Solana apps with LUMOS
   - Create public repositories
   - Encourage community contributions

4. **Documentation:**
   - Comprehensive guides
   - Video tutorials
   - Example projects

## Alternative Recognition Paths

If 200-repository threshold is challenging:

### 1. Domain-Specific Exception
Argue for exception based on:
- Specialized blockchain development need
- No existing alternatives for type-safe Solana schemas
- Growing Solana ecosystem (billions in TVL)
- Active development and documentation

### 2. VSCode Marketplace First
Build user base through:
- Publish VSCode extension
- Syntax highlighting
- IntelliSense/autocomplete
- This builds credibility for Linguist submission

### 3. Classification as DSL
Some DSLs get accepted with lower thresholds:
- Emphasize "schema language" nature
- Compare to similar accepted languages (Protocol Buffers, Thrift)

## Timeline

**Current (2025-01-18):**
- ✅ Basic recognition via `.gitattributes`
- ✅ Materials prepared for Linguist
- ⏳ Building LUMOS adoption

**Q2 2025:**
- Publish LUMOS packages (crates.io, npm)
- Release VSCode extension
- Create tutorial content

**Q3-Q4 2025:**
- Monitor repository adoption
- Gather community feedback
- Consider Linguist submission

**2026:**
- Submit to Linguist when threshold met
- OR: Apply for domain-specific exception

## Resources

- **GitHub Linguist:** https://github.com/github-linguist/linguist
- **Contributing Guide:** https://github.com/github-linguist/linguist/blob/master/CONTRIBUTING.md
- **TextMate Grammars:** https://macromates.com/manual/en/language_grammars
- **VSCode Language Extensions:** https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide

## Maintenance

When updating LUMOS syntax:

1. **Update grammar:** Modify `grammars/lumos.tmLanguage.json`
2. **Add samples:** Include new syntax in `samples/`
3. **Test locally:** Verify highlighting works
4. **Document changes:** Note in commit message

## Questions?

For questions about language recognition:
- **GitHub Linguist Issues:** https://github.com/github-linguist/linguist/issues
- **LUMOS Repository:** https://github.com/RECTOR-LABS/lumos/issues
