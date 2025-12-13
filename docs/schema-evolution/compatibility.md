# Schema Backward Compatibility Guide

**Automatically validate schema changes to prevent breaking deployments**

## Overview

When evolving Solana program schemas, you need to ensure that new versions can read data written by old versions. LUMOS provides automatic backward compatibility checking to prevent breaking changes that could cause deployed programs to fail.

## Quick Start

```bash
# Check compatibility between two schema versions
lumos check-compat schema_v1.lumos schema_v2.lumos

# Verbose output with detailed explanations
lumos check-compat schema_v1.lumos schema_v2.lumos --verbose

# JSON output for CI/CD
lumos check-compat schema_v1.lumos schema_v2.lumos --format json

# Treat warnings as errors (strict mode)
lumos check-compat schema_v1.lumos schema_v2.lumos --strict
```

---

## How It Works

The compatibility checker:

1. **Parses both schemas** - Old and new versions
2. **Detects changes** - Fields added, removed, or modified
3. **Classifies changes** - Breaking vs compatible vs informational
4. **Validates SemVer** - Ensures version bump matches changes
5. **Reports results** - Clear, actionable feedback

---

## Breaking vs Non-Breaking Changes

### ✅ Non-Breaking (Backward Compatible)

These changes allow new programs to read old account data:

**Adding Optional Fields**
```lumos
// v1.0.0
struct User {
    wallet: PublicKey,
    balance: u64,
}

// v1.1.0 - COMPATIBLE ✓
struct User {
    wallet: PublicKey,
    balance: u64,
    email: Option<String>,  // Safe: old data → email = None
}
```

**Reordering Fields**
```lumos
// v1.0.0
struct Player {
    wallet: PublicKey,
    level: u16,
    score: u64,
}

// v1.0.0 - COMPATIBLE ✓
struct Player {
    level: u16,        // Reordered
    wallet: PublicKey,
    score: u64,
}
```
*Borsh uses field position for serialization, so order is preserved.*

**Adding Enum Variants (at end)**
```lumos
// v1.0.0
enum Status {
    Active,
    Paused,
}

// v1.1.0 - COMPATIBLE ✓
enum Status {
    Active,     // discriminant = 0
    Paused,     // discriminant = 1
    Completed,  // discriminant = 2 (new)
}
```
*Old data only uses discriminants 0-1, so it still deserializes correctly.*

---

### ❌ Breaking (NOT Backward Compatible)

These changes cause deserialization failures:

**Adding Required Fields**
```lumos
// v1.0.0
struct User {
    wallet: PublicKey,
}

// v2.0.0 - BREAKING ✗
struct User {
    wallet: PublicKey,
    balance: u64,  // Required field - old data lacks this
}
```
**Error**: Old account data is missing `balance`, causing deserialization failure.

**Fix**: Make field optional or provide migration code.

---

**Removing Fields**
```lumos
// v1.0.0
struct Account {
    balance: u64,
    deprecated_field: bool,
}

// v2.0.0 - BREAKING ✗
struct Account {
    balance: u64,
    // deprecated_field removed
}
```
**Error**: Old account data contains extra bytes, causing deserialization mismatch.

**Fix**: Keep field but mark as deprecated, or create migration.

---

**Changing Field Types**
```lumos
// v1.0.0
struct Stats {
    count: u64,
}

// v2.0.0 - BREAKING ✗
struct Stats {
    count: u32,  // Type changed from u64 → u32
}
```
**Error**: Borsh expects u32 (4 bytes) but old data has u64 (8 bytes).

**Fix**: Create migration function or use new field name.

---

**Removing Enum Variants**
```lumos
// v1.0.0
enum Status {
    Active,
    Paused,
    Deprecated,
}

// v2.0.0 - BREAKING ✗
enum Status {
    Active,
    Paused,
    // Deprecated removed
}
```
**Error**: Old data may contain `Deprecated` variant (discriminant = 2), causing deserialization failure.

**Fix**: Keep variant but mark as deprecated.

---

## CLI Usage

### Basic Command

```bash
lumos check-compat <FROM_SCHEMA> <TO_SCHEMA> [OPTIONS]
```

### Options

| Option | Description |
|--------|-------------|
| `-f, --format <FORMAT>` | Output format: `text` or `json` (default: text) |
| `-v, --verbose` | Show detailed explanations for each change |
| `-s, --strict` | Treat warnings as errors (exit code 2) |

### Examples

**Preview Compatibility**
```bash
lumos check-compat v1.lumos v2.lumos
```

Output:
```
    Checking backward compatibility...
  From: v1.lumos
  To:   v2.lumos

Version: 1.0.0 → 2.0.0

✗ PlayerAccount: Added required field: experience (u64)
  Suggestion: Make field optional or provide migration code
✗ PlayerAccount: Added required field: inventory (Vec<u64>)
  Suggestion: Make field optional or provide migration code
ℹ PlayerAccount: Added optional field: email

Summary:
  ✗ 2 breaking changes
  ✗ New schema CANNOT read data written by old schema

  Recommendation: Create migration code or bump major version
```

---

**Verbose Output**
```bash
lumos check-compat v1.lumos v2.lumos --verbose
```

Output:
```
✗ PlayerAccount: Added required field: experience (u64)
  Suggestion: Make field optional or provide migration code
  Reason: Old account data lacks this field, deserialization will fail
```

---

**JSON Output (for CI/CD)**
```bash
lumos check-compat v1.lumos v2.lumos --format json
```

Output:
```json
{
  "from_version": "1.0.0",
  "to_version": "2.0.0",
  "compatible": false,
  "version_bump_valid": true,
  "breaking_changes": 2,
  "warnings": 0,
  "info": 1,
  "reports": [
    {
      "from_version": "1.0.0",
      "to_version": "2.0.0",
      "is_compatible": false,
      "issues": [
        {
          "level": "breaking",
          "type_name": "PlayerAccount",
          "message": "Added required field: experience (u64)",
          "reason": "Old account data lacks this field, deserialization will fail",
          "suggestion": "Make field optional or provide migration code"
        }
      ]
    }
  ]
}
```

---

### Exit Codes

| Code | Meaning | Use Case |
|------|---------|----------|
| `0` | Fully compatible | Safe to deploy |
| `1` | Breaking changes | Create migration or bump major version |
| `2` | Warnings (strict mode) | Review warnings before proceeding |

---

## SemVer Enforcement

LUMOS validates that version bumps match the severity of changes:

### Version Bump Rules

| Change Type | Required Version Bump | Example |
|-------------|---------------------|---------|
| Breaking changes | Major (`1.0.0` → `2.0.0`) | Added required field |
| New features (compatible) | Minor (`1.0.0` → `1.1.0`) | Added optional field |
| Bug fixes only | Patch (`1.0.0` → `1.0.1`) | No schema changes |

### Examples

**Valid: Major Bump for Breaking Change**
```bash
# v1.0.0 → v2.0.0 with required field added
lumos check-compat v1.lumos v2.lumos
```
✅ Version bump is valid

---

**Invalid: Minor Bump for Breaking Change**
```bash
# v1.0.0 → v1.1.0 with field removed
lumos check-compat v1.lumos v1.1.lumos
```
❌ Breaking changes require major version bump

Output:
```
✗ Version bump validation failed
  Breaking changes require a major version bump
```

---

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Schema Compatibility Check

on:
  pull_request:
    paths:
      - '**/*.lumos'

jobs:
  check-compat:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need full history

      - name: Install LUMOS CLI
        run: cargo install lumos-cli

      - name: Get main branch schema
        run: git show origin/main:schema.lumos > schema_old.lumos

      - name: Check compatibility
        run: |
          lumos check-compat \
            schema_old.lumos \
            schema.lumos \
            --format json \
            --strict
```

---

### GitLab CI Example

```yaml
schema-compatibility:
  stage: validate
  script:
    - cargo install lumos-cli
    - git show origin/main:schema.lumos > schema_old.lumos
    - lumos check-compat schema_old.lumos schema.lumos --strict
  only:
    changes:
      - "**/*.lumos"
```

---

## Best Practices

### 1. Always Use Versions

Always add `#[version]` attribute to your schemas:

```lumos
#[solana]
#[version = "1.0.0"]
#[account]
struct MyAccount {
    // ...
}
```

### 2. Check Before Merging

Run compatibility checks in CI/CD before merging schema changes:

```bash
lumos check-compat main:schema.lumos HEAD:schema.lumos --strict
```

### 3. Document Breaking Changes

When breaking changes are necessary:

1. **Document migration path**
2. **Provide migration code** (use `lumos migrate`)
3. **Coordinate with stakeholders**
4. **Plan deployment timeline**

### 4. Prefer Compatible Changes

When possible, evolve schemas using compatible changes:

```lumos
// Instead of adding required field:
balance: u64,

// Use optional field:
balance: Option<u64>,
```

### 5. Test Compatibility Locally

Before pushing, test compatibility:

```bash
# Compare with main branch
git show origin/main:schema.lumos > /tmp/schema_old.lumos
lumos check-compat /tmp/schema_old.lumos schema.lumos --verbose
```

---

## Troubleshooting

### "Type names don't match"

**Problem**: Comparing different types.

**Solution**: Ensure both schemas define the same type names:
```bash
# Both must have PlayerAccount type
lumos check-compat player_v1.lumos player_v2.lumos
```

---

### "Version bump validation failed"

**Problem**: Version bump doesn't match change severity.

**Solution**: Update version in new schema:
```lumos
// If breaking changes, use major bump
#[version = "2.0.0"]  // not 1.1.0
```

---

### False Positives

**Problem**: Reordering flagged as issue but safe.

**Solution**: Reordering is informational only, not breaking. Check exit code:
- Exit code `0` = safe (reordering is OK)
- Exit code `1` = breaking changes present

---

## Examples

### Example 1: Compatible Evolution

```lumos
// v1.0.0
struct Player {
    wallet: PublicKey,
    level: u16,
}

// v1.1.0 - All changes compatible
struct Player {
    wallet: PublicKey,
    level: u16,
    email: Option<String>,     // ✓ Optional
    last_seen: Option<u64>,    // ✓ Optional
}
```

Check:
```bash
lumos check-compat player_v1.lumos player_v1_1.lumos
```

Result:
```
✓ All changes are backward compatible
✓ New schema can read data written by old schema
```

---

### Example 2: Breaking Change

```lumos
// v1.0.0
struct Player {
    wallet: PublicKey,
    level: u16,
    deprecated_score: u64,
}

// v2.0.0 - Breaking change
struct Player {
    wallet: PublicKey,
    level: u32,  // ✗ Type changed
    // deprecated_score removed ✗
}
```

Check:
```bash
lumos check-compat player_v1.lumos player_v2.lumos
```

Result:
```
✗ 2 breaking changes
✗ New schema CANNOT read data written by old schema

Recommendation: Create migration code or bump major version
```

---

## Related

- [Schema Migration Guide](./migrations.md) - Generate migration code
- [Schema Versioning](../syntax-reference.md#version) - `#[version]` attribute
- [Borsh Serialization](https://borsh.io/) - Understanding data format

---

**Next Steps**:
- [Migration Code Generation](./migrations.md)
- [Backward Compatibility Validation](./compatibility.md)
- [Schema Diff Tool](./diff.md)
