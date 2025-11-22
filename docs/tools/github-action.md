# GitHub Action for LUMOS

Automate LUMOS schema generation and validation in your CI/CD pipeline.

## Overview

The LUMOS GitHub Action automatically:

1. **Validates** LUMOS schemas for syntax and semantic correctness
2. **Generates** Rust and TypeScript code from schemas
3. **Detects drift** between generated and committed files
4. **Comments on PRs** with generation results and diffs
5. **Fails builds** when drift is detected (configurable)

## Quick Start

Add this to `.github/workflows/lumos.yml`:

```yaml
name: LUMOS Generate

on:
  push:
    branches: [main, dev]
  pull_request:

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Generate from LUMOS schemas
        uses: getlumos/lumos/.github/actions/lumos-generate@main
        with:
          schema: 'schemas/**/*.lumos'
```

## Use Cases

### 1. Automated Generation on Push

Generate code whenever schemas change:

```yaml
name: Auto-generate

on:
  push:
    paths:
      - '**/*.lumos'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: getlumos/lumos/.github/actions/lumos-generate@main
        with:
          schema: 'schema.lumos'
          fail-on-drift: false

      - name: Commit generated files
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"
          git add .
          git diff --staged --quiet || git commit -m "chore: Update generated files from schemas"
          git push
```

### 2. PR Validation (Check Mode)

Ensure PRs don't have uncommitted generated files:

```yaml
name: LUMOS Validation

on:
  pull_request:

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: getlumos/lumos/.github/actions/lumos-generate@main
        with:
          schema: '**/*.lumos'
          check-only: false      # Generate to check drift
          fail-on-drift: true    # Fail if drift detected
          comment-on-pr: true    # Post results as comment
```

### 3. Multi-Schema Monorepo

Generate from multiple schema files:

```yaml
jobs:
  generate:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        program:
          - name: nft-marketplace
            schema: programs/nft/schema.lumos
          - name: defi-staking
            schema: programs/defi/schema.lumos
          - name: dao-governance
            schema: programs/dao/schema.lumos

    steps:
      - uses: actions/checkout@v4

      - uses: getlumos/lumos/.github/actions/lumos-generate@main
        with:
          schema: ${{ matrix.program.schema }}
          working-directory: programs/${{ matrix.program.name }}
```

### 4. Version Pinning

Lock to specific LUMOS version for reproducibility:

```yaml
- uses: getlumos/lumos/.github/actions/lumos-generate@main
  with:
    schema: 'schema.lumos'
    version: '0.1.1'  # Pin to specific version
```

## Configuration

### Inputs

#### `schema` (required)

Path pattern for LUMOS schema files. Supports glob patterns.

```yaml
# Single file
schema: 'schema.lumos'

# Glob pattern
schema: 'schemas/**/*.lumos'

# Multiple files (YAML multiline)
schema: |
  programs/nft/schema.lumos
  programs/defi/schema.lumos
```

#### `check-only` (optional, default: `false`)

When `true`, only validates schemas without generating code. Useful for quick checks.

```yaml
check-only: true  # Validation only
check-only: false # Validation + generation
```

#### `version` (optional, default: `latest`)

LUMOS CLI version to install from crates.io.

```yaml
version: 'latest'  # Latest published version
version: '0.1.1'   # Specific version
```

#### `working-directory` (optional, default: `.`)

Directory to run commands in.

```yaml
working-directory: './programs/my-program'
```

#### `fail-on-drift` (optional, default: `true`)

Whether to fail the action when drift is detected.

```yaml
fail-on-drift: true   # Fail build on drift
fail-on-drift: false  # Only warn, don't fail
```

#### `comment-on-pr` (optional, default: `true`)

Whether to post PR comments with results.

```yaml
comment-on-pr: true   # Post comments
comment-on-pr: false  # Silent mode
```

### Outputs

Access outputs in subsequent steps:

```yaml
- name: Generate
  id: lumos
  uses: getlumos/lumos/.github/actions/lumos-generate@main
  with:
    schema: 'schema.lumos'

- name: Use outputs
  run: |
    echo "Validated: ${{ steps.lumos.outputs.schemas-validated }}"
    echo "Generated: ${{ steps.lumos.outputs.schemas-generated }}"
    echo "Drift: ${{ steps.lumos.outputs.drift-detected }}"
```

Available outputs:

- `schemas-validated` - Number of schemas validated
- `schemas-generated` - Number of schemas generated
- `drift-detected` - `true` or `false`
- `diff-summary` - Markdown summary of differences

## Advanced Workflows

### Conditional Generation

Only generate on main branch, validate on PRs:

```yaml
jobs:
  lumos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: getlumos/lumos/.github/actions/lumos-generate@main
        with:
          schema: '**/*.lumos'
          check-only: ${{ github.event_name == 'pull_request' }}
          fail-on-drift: ${{ github.event_name == 'pull_request' }}
```

### Artifact Upload

Save generated files as artifacts:

```yaml
- uses: getlumos/lumos/.github/actions/lumos-generate@main
  with:
    schema: 'schema.lumos'

- name: Upload generated files
  uses: actions/upload-artifact@v4
  with:
    name: generated-code
    path: |
      generated.rs
      generated.ts
```

### Cache LUMOS CLI

Speed up builds by caching the CLI:

```yaml
- name: Cache LUMOS CLI
  uses: actions/cache@v4
  with:
    path: ~/.cargo/bin/lumos
    key: ${{ runner.os }}-lumos-${{ inputs.version }}

- uses: getlumos/lumos/.github/actions/lumos-generate@main
  with:
    schema: 'schema.lumos'
```

### Slack/Discord Notifications

Notify on drift detection:

```yaml
- name: Generate
  id: lumos
  uses: getlumos/lumos/.github/actions/lumos-generate@main
  with:
    schema: 'schema.lumos'
    fail-on-drift: false

- name: Notify on drift
  if: steps.lumos.outputs.drift-detected == 'true'
  uses: slackapi/slack-github-action@v1
  with:
    webhook-url: ${{ secrets.SLACK_WEBHOOK }}
    payload: |
      {
        "text": "⚠️ LUMOS drift detected in ${{ github.repository }}"
      }
```

## PR Comments

When `comment-on-pr: true`, the action posts comments like:

```markdown
## 🔮 LUMOS Generation Report

- Validated: 3 schema(s)
- Generated: 3 schema(s)

## 📊 LUMOS Generation Drift Detected

The following files differ from their generated versions:

- `programs/nft/generated.rs`
- `programs/nft/generated.ts`

<details>
<summary>View full diff</summary>

```diff
diff --git a/programs/nft/generated.rs b/programs/nft/generated.rs
index 1234567..abcdefg 100644
--- a/programs/nft/generated.rs
+++ b/programs/nft/generated.rs
@@ -10,7 +10,7 @@ pub struct Metadata {
-    pub name: String,
+    pub title: String,
```

</details>

---

*Generated by [LUMOS](https://lumos-lang.org) v0.1.1*
```

## Troubleshooting

### Drift Always Detected

**Problem**: Drift is detected even when files match.

**Solutions**:

1. **Line endings**: Ensure consistent line endings (LF vs CRLF)
   ```yaml
   - name: Configure git
     run: git config core.autocrlf false
   ```

2. **Formatting**: Ensure rustfmt version matches local development
   ```yaml
   - uses: actions-rust-lang/setup-rust-toolchain@v1
     with:
       toolchain: stable
       components: rustfmt
   ```

### Version Not Found

**Problem**: `cargo install` fails with version not found.

**Solution**: Verify version exists on crates.io or use `latest`.

### Permission Errors

**Problem**: Cannot commit/push generated files.

**Solution**: Grant write permissions to `GITHUB_TOKEN`:

```yaml
permissions:
  contents: write
  pull-requests: write
```

### Glob Pattern Not Working

**Problem**: Schema files not found with glob pattern.

**Solution**: Ensure glob syntax is correct:

```yaml
# Correct
schema: 'schemas/**/*.lumos'

# Incorrect (missing quotes)
schema: schemas/**/*.lumos
```

## Best Practices

### 1. Pin Versions in Production

```yaml
version: '0.1.1'  # ✅ Reproducible builds
version: 'latest'  # ❌ May break unexpectedly
```

### 2. Separate Validation and Generation

```yaml
# PR validation (fast)
on: pull_request
  check-only: true

# Production generation (slow)
on: push
  check-only: false
```

### 3. Use Matrix for Monorepos

```yaml
strategy:
  matrix:
    schema:
      - programs/nft/schema.lumos
      - programs/defi/schema.lumos
```

### 4. Fail Fast on PRs

```yaml
fail-on-drift: true  # PRs must have committed generated files
comment-on-pr: true  # Show what needs to be fixed
```

## Publishing to Marketplace

To publish this action to GitHub Marketplace:

1. **Create Repository**
   ```bash
   gh repo create getlumos/lumos-action --public
   ```

2. **Copy Files**
   ```bash
   cp .github/actions/lumos-generate/action.yml ./
   cp .github/actions/lumos-generate/README.md ./
   ```

3. **Tag Release**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

4. **Create GitHub Release**
   - Go to repository releases
   - Create release from tag
   - Enable "Publish to Marketplace"

5. **Update Usage**
   ```yaml
   uses: getlumos/lumos-action@v1
   ```

## Migration from Manual Generation

**Before** (manual):

```yaml
- name: Install LUMOS
  run: cargo install lumos-cli

- name: Generate
  run: lumos generate schema.lumos

- name: Check diff
  run: git diff --exit-code
```

**After** (automated):

```yaml
- uses: getlumos/lumos/.github/actions/lumos-generate@main
  with:
    schema: 'schema.lumos'
```

## Security Considerations

### 1. Dependency Security

The action installs LUMOS CLI from crates.io. Use specific versions to avoid supply chain attacks:

```yaml
version: '0.1.1'  # Pin to audited version
```

### 2. Token Permissions

Minimize GitHub token permissions:

```yaml
permissions:
  contents: read        # Read repo
  pull-requests: write  # Comment on PRs (if enabled)
```

### 3. Branch Protection

Require drift checks in branch protection rules:

```yaml
# .github/workflows/lumos-check.yml
on: pull_request
  fail-on-drift: true  # Enforced by branch protection
```

## Examples Repository

See full working examples at: https://github.com/getlumos/awesome-lumos/tree/main/.github/workflows

## Support

- **Documentation**: https://lumos-lang.org
- **Issues**: https://github.com/getlumos/lumos/issues
- **Discussions**: https://github.com/getlumos/lumos/discussions

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
