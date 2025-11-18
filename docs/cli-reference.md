# LUMOS CLI Reference

Complete command-line interface reference for LUMOS.

---

## Installation

```bash
cargo install lumos-cli
```

---

## Commands

### `lumos generate`

Generate Rust and TypeScript code from a `.lumos` schema file.

#### Basic Usage

```bash
lumos generate <SCHEMA_FILE> [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--output <DIR>` | Output directory (default: current directory) |
| `--watch` | Watch for changes and regenerate automatically |
| `--dry-run` | Preview changes without writing files |
| `--backup` | Create `.backup` files before overwriting |
| `--show-diff` | Show diff and ask for confirmation before writing |

#### Examples

**Basic generation:**
```bash
lumos generate schema.lumos
```

**Generate to specific directory:**
```bash
lumos generate schema.lumos --output ./generated
```

**Watch mode (auto-regenerate on changes):**
```bash
lumos generate schema.lumos --watch
```

**Preview changes without writing (dry-run):**
```bash
lumos generate schema.lumos --dry-run
```
Output:
```
🔍 Dry-run mode (no files will be written)

Would generate: ./generated.rs (Rust)
  Size: 245 lines (5.2 KB)
  New file (doesn't exist yet)

Would generate: ./generated.ts (TypeScript)
  Size: 180 lines (4.1 KB)
  New file (doesn't exist yet)

No files written (dry-run mode).
Run without --dry-run to apply changes.
```

**Create backups before overwriting:**
```bash
lumos generate schema.lumos --backup
```
Output:
```
Backing up files...
  ./generated.rs → ./generated.rs.backup
  ./generated.ts → ./generated.ts.backup

Backups created. Restore with:
  mv ./generated.rs.backup ./generated.rs
  mv ./generated.ts.backup ./generated.ts
```

**Show diff and confirm before overwriting:**
```bash
lumos generate schema.lumos --show-diff
```
Output:
```
────────────────────────────────────────────────────────────
DIFF: ./generated.rs (Rust)
────────────────────────────────────────────────────────────

  #[account]
  pub struct PlayerAccount {
      pub wallet: Pubkey,
      pub score: u64,
-     pub level: u16,
+     pub level: u32,  // Changed type
  }

Summary:
  Lines added: 1
  Lines removed: 1

Apply changes to ./generated.rs? [y/N] _
```

**Combine backup with diff confirmation:**
```bash
lumos generate schema.lumos --backup --show-diff
```

#### Safety Features

##### `--dry-run` Mode

**Purpose:** Preview what would be generated without writing any files.

**Use when:**
- Testing schema changes before committing
- Verifying output before overwriting existing files
- Debugging generation issues
- Code review workflows

**Behavior:**
- Parses schema and generates code in memory
- Shows file paths, sizes, and line counts
- Indicates if files are new or existing
- No files are written to disk
- Can be combined with `--output` to preview different output directories

##### `--backup` Mode

**Purpose:** Create backup copies before overwriting existing files.

**Use when:**
- Experimenting with schema changes
- Want quick rollback capability
- Working without version control (not recommended)
- Extra safety during development

**Behavior:**
- Checks if output files exist
- Creates `.backup` files (e.g., `generated.rs.backup`)
- Then overwrites original files with new code
- Provides restore commands in output
- Backups are overwritten on subsequent runs (not versioned)

**Restore backup:**
```bash
mv generated.rs.backup generated.rs
mv generated.ts.backup generated.ts
```

##### `--show-diff` Mode

**Purpose:** Interactive mode showing changes and asking for confirmation.

**Use when:**
- Reviewing schema changes before applying
- Want to see exactly what will change
- Working on critical production code
- Teaching/learning workflows

**Behavior:**
- Shows line-by-line diff with colors:
  - `-` Red: Lines removed
  - `+` Green: Lines added
  - White: Unchanged context
- Displays summary (lines added/removed)
- Prompts for confirmation per file: `[y/N]`
- Only writes files if confirmed with `y`
- Skips files if answered `n` or Enter (default: no)

##### Combining Flags

Safety flags can be combined for maximum protection:

```bash
# Backup + Diff: Create backups AND review changes
lumos generate schema.lumos --backup --show-diff

# Dry-run + Backup: Preview first, then generate with backup
lumos generate schema.lumos --dry-run  # Review
lumos generate schema.lumos --backup   # Apply with backup
```

**Flag Conflicts:**
- `--dry-run` and `--show-diff` together: Dry-run takes precedence (no confirmation needed)
- `--dry-run` and `--backup` together: Dry-run takes precedence (no files written, no backups needed)

##### Best Practices

**Development workflow:**
```bash
# 1. Preview changes
lumos generate schema.lumos --dry-run

# 2. Generate with backup for safety
lumos generate schema.lumos --backup

# 3. Test generated code
cargo test && npm test

# 4. If happy, commit to git
git add generated.*
git commit -m "Update schema"
```

**Production workflow:**
```bash
# Review changes interactively
lumos generate schema.lumos --backup --show-diff
```

**Watch mode note:**
Safety flags are disabled in watch mode (`--watch`) to allow automatic regeneration without prompts.

---

### `lumos validate`

Validate a `.lumos` schema file for syntax errors.

#### Usage

```bash
lumos validate <SCHEMA_FILE>
```

#### Example

```bash
lumos validate schema.lumos
```

Output on success:
```
✓ Validating schema.lumos...

  Syntax check ✓
  Type definitions ✓
  Field types ✓
  Enum variants ✓
  Dependencies ✓

✓ Schema is valid!

Summary:
  Structs: 3
  Enums: 2
  Total fields: 24
  No errors, no warnings
```

Output on error:
```
✗ Validation failed

Error at line 12:
  Invalid type: PublickKey
  Did you mean: PublicKey?
```

---

### `lumos init`

Initialize a new LUMOS project.

#### Usage

```bash
lumos init [PROJECT_NAME]
```

#### Example

```bash
lumos init my-solana-project
cd my-solana-project
```

Creates:
```
my-solana-project/
├── schema.lumos        # Example schema
├── .gitignore          # Ignores generated/ folder
└── README.md           # Project setup guide
```

---

### `lumos check`

Health check and diagnostics for LUMOS installation.

#### Usage

```bash
lumos check
```

#### Example Output

```
LUMOS Health Check
──────────────────

✓ LUMOS version: 0.2.0
✓ Rust toolchain: 1.75.0
✓ Node.js: v20.10.0
✓ npm: 10.2.3

All systems operational!
```

---

## Common Workflows

### Starting a New Project

```bash
# Initialize project
lumos init my-project
cd my-project

# Edit schema.lumos
vim schema.lumos

# Generate code
lumos generate schema.lumos

# Use in Rust/TypeScript
```

### Migrating Existing Project

See [Integration Guide](./integration-guide.md) for detailed migration steps.

```bash
# Extract from existing Rust code (planned feature)
lumos extract src/state.rs --output schema.lumos

# Generate TypeScript types
lumos generate schema.lumos --output ./client/src
```

### Development with Watch Mode

```bash
# Terminal 1: Watch and regenerate
lumos generate schema.lumos --watch

# Terminal 2: Edit schema
vim schema.lumos
# (saves automatically trigger regeneration)

# Terminal 3: Run tests
cargo test --watch
```

### Safe Schema Changes

```bash
# 1. Preview what will change
lumos generate schema.lumos --dry-run

# 2. Review line-by-line diff
lumos generate schema.lumos --show-diff

# 3. Accept changes and create backup
lumos generate schema.lumos --backup --show-diff

# 4. Test
cargo test && npm test

# 5. Commit
git add schema.lumos generated.*
git commit -m "feat: Add player level field"
```

---

## Output Files

LUMOS generates two files by default:

| File | Description |
|------|-------------|
| `generated.rs` | Rust structs with Anchor/Borsh derives |
| `generated.ts` | TypeScript interfaces and Borsh schemas |

**Important:**
- Generated files include warning comments: `// DO NOT EDIT - Changes will be overwritten`
- Always edit the `.lumos` schema, never the generated files directly
- Use Git to track both schema and generated files

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (invalid syntax, file not found, etc.) |
| 2 | Validation error |
| 3 | Generation error |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LUMOS_LOG` | Log level (debug, info, warn, error) | `info` |
| `NO_COLOR` | Disable colored output | - |

Example:
```bash
LUMOS_LOG=debug lumos generate schema.lumos
```

---

## Troubleshooting

### Generated files not updating

**Symptoms:** Running `lumos generate` but files don't change.

**Solutions:**
1. Check file permissions: `ls -la generated.*`
2. Use `--show-diff` to see if there are actual changes
3. Delete generated files and regenerate:
   ```bash
   rm generated.* && lumos generate schema.lumos
   ```

### Backup files accumulating

**Symptoms:** Multiple `.backup` files in project.

**Solutions:**
1. Add to `.gitignore`:
   ```
   *.backup
   ```
2. Clean up old backups:
   ```bash
   rm *.backup
   ```
3. Use Git instead of `--backup` flag for version control

### Watch mode not detecting changes

**Symptoms:** `--watch` mode doesn't regenerate on save.

**Solutions:**
1. Check file is actually being saved (vim: `:w`, vscode: Cmd+S)
2. Restart watch mode
3. Check file permissions
4. Try manual generation to verify schema is valid

### Diff shows unexpected changes

**Symptoms:** `--show-diff` shows changes you didn't make.

**Possible causes:**
1. Code formatter changed formatting
2. Import order changed
3. Generator logic updated

**Solutions:**
1. Review changes carefully before accepting
2. Check generator version: `lumos check`
3. Commit formatting changes separately

---

## Getting Help

```bash
# General help
lumos --help

# Command-specific help
lumos generate --help
lumos validate --help
lumos init --help
lumos check --help
```

**Resources:**
- Documentation: `docs/`
- Examples: `examples/`
- GitHub Issues: https://github.com/RECTOR-LABS/lumos/issues

---

**Last Updated:** 2025-11-18 (Safety features added)
