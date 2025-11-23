# `lumos check-compat` - Backward Compatibility Checker

Validate that schema changes are backward compatible and won't break existing deployed programs.

## Synopsis

```bash
lumos check-compat <FROM_SCHEMA> <TO_SCHEMA> [OPTIONS]
```

## Description

The `check-compat` command analyzes two schema versions and determines whether the new version can safely read data written by the old version. This prevents breaking changes that could cause Solana program failures in production.

The command:
- Detects all changes between schemas
- Classifies changes as breaking, warning, or informational
- Validates semantic version bumps
- Provides actionable recommendations
- Supports both human-readable and JSON output
- Returns appropriate exit codes for CI/CD integration

## Arguments

### `<FROM_SCHEMA>`

Path to the old `.lumos` schema file (v1).

**Example**: `schema_v1.lumos`

### `<TO_SCHEMA>`

Path to the new `.lumos` schema file (v2).

**Example**: `schema_v2.lumos`

## Options

### `-f, --format <FORMAT>`

Output format.

**Values**:
- `text` - Human-readable output with colors (default)
- `json` - Machine-readable JSON for CI/CD

**Default**: `text`

**Example**:
```bash
lumos check-compat v1.lumos v2.lumos --format json
```

### `-v, --verbose`

Show detailed explanations for each change, including:
- Full reason for compatibility issue
- Suggestions for fixing breaking changes
- Information about safe changes

**Example**:
```bash
lumos check-compat v1.lumos v2.lumos --verbose
```

### `-s, --strict`

Treat warnings as errors. Exit with code 2 if any warnings are found.

Useful for enforcing strict compatibility policies in CI/CD.

**Example**:
```bash
lumos check-compat v1.lumos v2.lumos --strict
```

## Exit Codes

| Code | Meaning | Description |
|------|---------|-------------|
| `0` | Compatible | All changes are backward compatible. Safe to deploy. |
| `1` | Breaking changes | Incompatible changes detected. Migration or major version bump required. |
| `2` | Warnings (strict mode) | Warnings found and `--strict` flag used. |

## Examples

### Basic Usage

Check compatibility between two schemas:

```bash
lumos check-compat player_v1.lumos player_v2.lumos
```

Output:
```
    Checking backward compatibility...
  From: player_v1.lumos
  To:   player_v2.lumos

Version: 1.0.0 → 2.0.0

✗ PlayerAccount: Added required field: experience (u64)
  Suggestion: Make field optional or provide migration code
ℹ PlayerAccount: Added optional field: email

Summary:
  ✗ 1 breaking change
  ✗ New schema CANNOT read data written by old schema

  Recommendation: Create migration code or bump major version
```

### Verbose Output

Get detailed explanations for each change:

```bash
lumos check-compat player_v1.lumos player_v2.lumos --verbose
```

Output:
```
✗ PlayerAccount: Added required field: experience (u64)
  Suggestion: Make field optional or provide migration code
  Reason: Old account data lacks this field, deserialization will fail
ℹ PlayerAccount: Added optional field: email
  Reason: Old data will deserialize successfully with this field as None
```

### JSON Output

Get machine-readable output for CI/CD:

```bash
lumos check-compat player_v1.lumos player_v2.lumos --format json
```

Output:
```json
{
  "from_version": "1.0.0",
  "to_version": "2.0.0",
  "compatible": false,
  "version_bump_valid": true,
  "breaking_changes": 1,
  "warnings": 0,
  "info": 1,
  "reports": [
    {
      "from_version": "1.0.0",
      "to_version": "2.0.0",
      "is_compatible": false,
      "version_bump_valid": true,
      "issues": [
        {
          "level": "breaking",
          "type_name": "PlayerAccount",
          "message": "Added required field: experience (u64)",
          "reason": "Old account data lacks this field, deserialization will fail",
          "suggestion": "Make field optional or provide migration code",
          "change": {
            "FieldAdded": {
              "name": "experience",
              "type_info": { "Primitive": "u64" },
              "optional": false
            }
          }
        },
        {
          "level": "info",
          "type_name": "PlayerAccount",
          "message": "Added optional field: email",
          "reason": "Old data will deserialize successfully with this field as None",
          "suggestion": null,
          "change": {
            "FieldAdded": {
              "name": "email",
              "type_info": { "Option": { "Primitive": "String" } },
              "optional": true
            }
          }
        }
      ]
    }
  ]
}
```

### Strict Mode

Fail on any warnings:

```bash
lumos check-compat v1.lumos v2.lumos --strict
```

If warnings exist, exit code will be `2`.

### CI/CD Integration

Use in GitHub Actions:

```yaml
- name: Check schema compatibility
  run: |
    git show origin/main:schema.lumos > schema_old.lumos
    lumos check-compat schema_old.lumos schema.lumos --strict
```

Use in GitLab CI:

```yaml
schema-check:
  script:
    - git show origin/main:schema.lumos > schema_old.lumos
    - lumos check-compat schema_old.lumos schema.lumos --format json --strict
```

## Compatibility Rules

### Non-Breaking Changes

These changes are backward compatible:

- ✅ Adding optional fields (`Option<T>`)
- ✅ Reordering fields (Borsh uses position, not names)
- ✅ Adding enum variants at the end

### Breaking Changes

These changes are NOT backward compatible:

- ❌ Adding required fields (unless migrated)
- ❌ Removing fields (data loss)
- ❌ Changing field types (deserialization failure)
- ❌ Removing enum variants (old data may use them)

## SemVer Validation

The command validates that version bumps match the severity of changes:

| Change Type | Required Version Bump | Example |
|-------------|---------------------|---------|
| Breaking | Major (`1.0.0` → `2.0.0`) | Required field added |
| Compatible new features | Minor (`1.0.0` → `1.1.0`) | Optional field added |
| No schema changes | Patch (`1.0.0` → `1.0.1`) | Bug fixes only |

**Invalid version bump example**:

```bash
# v1.0.0 → v1.1.0 with breaking changes
lumos check-compat v1.lumos v1_1.lumos
```

Output:
```
✗ Version bump validation failed
  Breaking changes require a major version bump
```

## JSON Output Schema

The JSON output follows this schema:

```typescript
{
  from_version: string | null,
  to_version: string | null,
  compatible: boolean,
  version_bump_valid: boolean,
  breaking_changes: number,
  warnings: number,
  info: number,
  reports: Array<{
    from_version: string | null,
    to_version: string | null,
    is_compatible: boolean,
    version_bump_valid: boolean,
    issues: Array<{
      level: "breaking" | "warning" | "info",
      type_name: string,
      message: string,
      reason: string,
      suggestion: string | null,
      change: SchemaChange
    }>
  }>
}
```

## Common Use Cases

### 1. Pre-Commit Hook

Check compatibility before committing:

```bash
#!/bin/bash
# .git/hooks/pre-commit

git show HEAD:schema.lumos > /tmp/schema_old.lumos
lumos check-compat /tmp/schema_old.lumos schema.lumos --strict

if [ $? -ne 0 ]; then
  echo "❌ Schema compatibility check failed"
  exit 1
fi
```

### 2. Pull Request Validation

GitHub Action example:

```yaml
name: Schema Compatibility

on: pull_request

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install LUMOS
        run: cargo install lumos-cli

      - name: Check compatibility
        run: |
          git show origin/main:schema.lumos > old.lumos
          lumos check-compat old.lumos schema.lumos --format json --strict > report.json

      - name: Comment on PR
        uses: actions/github-script@v7
        if: failure()
        with:
          script: |
            const report = require('./report.json');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## ❌ Schema Compatibility Check Failed\n\nBreaking changes: ${report.breaking_changes}`
            });
```

### 3. Release Validation

Check before creating a release:

```bash
git show v1.0.0:schema.lumos > schema_v1.lumos
lumos check-compat schema_v1.lumos schema.lumos --verbose

if [ $? -eq 1 ]; then
  echo "Breaking changes detected. Create migration code first."
  lumos migrate schema_v1.lumos schema.lumos --output migration.rs
fi
```

## Troubleshooting

### Type Names Don't Match

**Error**: `Type names don't match: 'Player' vs 'User'`

**Solution**: Ensure both schemas define the same types. You cannot compare different types.

### No Version Attribute

**Warning**: Schemas without `#[version]` attribute won't have version bump validation.

**Solution**: Add version to your schemas:
```lumos
#[solana]
#[version = "1.0.0"]
#[account]
struct MyAccount { ... }
```

### Exit Code Always 0

**Problem**: Command succeeds even with breaking changes.

**Solution**: Check exit code correctly:
```bash
if lumos check-compat v1.lumos v2.lumos; then
  echo "Compatible"
else
  echo "Breaking changes (exit code: $?)"
fi
```

## See Also

- [`lumos migrate`](./migrate.md) - Generate migration code
- [`lumos diff`](./diff.md) - Show schema differences
- [Compatibility Guide](../schema-evolution/compatibility.md) - Full compatibility reference
- [Migration Guide](../schema-evolution/migrations.md) - Schema migration patterns

---

**Related Commands**:
- `lumos validate` - Validate schema syntax
- `lumos generate` - Generate code from schema
- `lumos migrate` - Generate migration code
