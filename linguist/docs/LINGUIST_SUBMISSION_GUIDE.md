# GitHub Linguist Submission Guide for LUMOS

This guide explains how to submit LUMOS to GitHub Linguist for official language recognition.

## Prerequisites

1. **Repository Requirements:**
   - LUMOS must have public repositories using it
   - At least 200 unique repositories (GitHub's threshold for new languages)
   - OR: Strong justification for domain-specific language

2. **Language Readiness:**
   - Stable syntax specification
   - Working parser and code generator
   - Sample code demonstrating language features

## Submission Process

### Step 1: Fork GitHub Linguist

```bash
git clone https://github.com/github-linguist/linguist.git
cd linguist
git checkout -b add-lumos-language
```

### Step 2: Add Language Definition

Edit `lib/linguist/languages.yml` and add the LUMOS entry from `../languages.yml`:

```yaml
LUMOS:
  type: programming
  color: "#8B5CF6"
  aliases:
  - lumos
  extensions:
  - ".lumos"
  tm_scope: source.lumos
  ace_mode: text
  language_id: <ASSIGNED_BY_MAINTAINERS>
```

### Step 3: Add TextMate Grammar

Copy `../grammars/lumos.tmLanguage.json` to `grammars/` directory in Linguist repo:

```bash
cp /path/to/lumos/linguist/grammars/lumos.tmLanguage.json grammars/
```

### Step 4: Add Sample Files

Copy sample files from `../samples/` to `samples/LUMOS/` in Linguist repo:

```bash
mkdir -p samples/LUMOS
cp ../samples/*.lumos samples/LUMOS/
```

Requirements:
- At least 2-3 representative sample files
- Show diverse language features
- Well-commented to demonstrate syntax

### Step 5: Update Vendor Files (if needed)

If LUMOS has third-party files that shouldn't count in language stats, add them to `lib/linguist/vendor.yml`.

### Step 6: Run Tests

```bash
# Install dependencies
bundle install

# Run Linguist tests
bundle exec rake test

# Generate language data
bundle exec licensed cache

# Verify samples are detected correctly
bundle exec github-linguist samples/LUMOS/
```

### Step 7: Commit and Push

```bash
git add .
git commit -m "Add support for LUMOS language

LUMOS is a type-safe schema language for Solana blockchain development.
It generates cross-language code (Rust/TypeScript) with guaranteed type safety.

Repository: https://github.com/RECTOR-LABS/lumos
"

git push origin add-lumos-language
```

### Step 8: Create Pull Request

1. Go to: https://github.com/github-linguist/linguist/pulls
2. Click "New Pull Request"
3. Select your fork and branch
4. Fill in PR template:

**Title:** Add support for LUMOS language

**Description:**
```markdown
## Summary
This PR adds support for LUMOS, a type-safe schema language for Solana blockchain development.

## Language Overview
- **Purpose:** Cross-language type-safe code generation for Solana
- **Target Languages:** Rust (on-chain programs) and TypeScript (clients)
- **Use Case:** Solana blockchain application development

## Repository
- Main repository: https://github.com/RECTOR-LABS/lumos
- License: MIT
- Stars: [current count]

## Sample Repositories (if available)
- [List repositories using LUMOS]

## Justification
LUMOS is a domain-specific language designed for the Solana blockchain ecosystem.
While it may not yet meet the 200-repository threshold, it serves a specialized
domain with active development and growing adoption in the Solana community.

## Changes
- Added language definition to `languages.yml`
- Added TextMate grammar for syntax highlighting
- Added sample files demonstrating language features
- All tests passing

## Related Issues
Closes #XXXX (if there's a related issue)
```

### Step 9: Address Review Feedback

Linguist maintainers may request:
- More sample repositories using LUMOS
- Grammar improvements
- Additional test coverage
- Justification for domain-specific exception

## Alternative: Interim Solution

If LUMOS doesn't meet the 200-repository threshold yet:

### Option 1: Use `.gitattributes` (Already Implemented)

Currently using:
```gitattributes
*.lumos linguist-language=Rust linguist-detectable=true
```

This classifies LUMOS as Rust for now, which is semantically close since LUMOS compiles to Rust.

### Option 2: Create VSCode Extension First

Build adoption through developer tools:
1. VSCode extension with syntax highlighting
2. Language server for IntelliSense
3. Use extension to grow user base
4. Return to Linguist when threshold is met

### Option 3: Submit as "Data" or "Markup" Type

If Linguist rejects "programming" type, consider:
```yaml
LUMOS:
  type: data  # or markup
  # ... rest of definition
```

This has lower adoption requirements but different classification.

## Timeline Expectations

- **Initial review:** 1-2 weeks
- **Revision cycles:** 2-4 weeks
- **Merge time:** 1-3 months (if accepted)
- **GitHub.com deployment:** 1-2 weeks after merge

## Checklist

- [ ] Fork Linguist repository
- [ ] Add language definition to `languages.yml`
- [ ] Add TextMate grammar to `grammars/`
- [ ] Add sample files to `samples/LUMOS/`
- [ ] Run all tests successfully
- [ ] Create pull request
- [ ] Respond to maintainer feedback
- [ ] Wait for merge and deployment

## Resources

- **Linguist Contributing Guide:** https://github.com/github-linguist/linguist/blob/master/CONTRIBUTING.md
- **Language Guidelines:** https://github.com/github-linguist/linguist/blob/master/docs/languages.md
- **TextMate Grammars:** https://macromates.com/manual/en/language_grammars
- **Linguist Issues:** https://github.com/github-linguist/linguist/issues

## Notes

- GitHub assigns the `language_id` - leave placeholder in initial PR
- Color should be unique and representative
- `tm_scope` must match grammar file
- Consider SEO: use searchable language name

## Post-Acceptance

After LUMOS is accepted:

1. Update `.gitattributes` in LUMOS repo:
   ```gitattributes
   *.lumos linguist-language=LUMOS linguist-detectable=true
   ```

2. Add badge to README:
   ```markdown
   ![Language](https://img.shields.io/github/languages/top/RECTOR-LABS/lumos)
   ```

3. Announce on social media and dev communities
4. Update documentation to reference GitHub support
