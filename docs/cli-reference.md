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
| `--address <PROGRAM_ID>` | Anchor program id to embed in generated Anchor code (when schema uses `#[account]`). If omitted the CLI will prompt interactively; in non-interactive contexts a sentinel placeholder will be used which causes a compile-time error until replaced. |

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

##### Anchor Program ID (`--address`)

When generating Rust code that uses the Anchor framework (module contains `#[account]` structs), the generator needs a program id to include at crate-level using `declare_id!("...")`.

- If you provide `--address <PROGRAM_ID>`, the value will be embedded directly into the generated `generated.rs` file.
- `--address` is required when the schema uses Anchor (`#[account]` structs). The CLI will return an error if you omit `--address` in that case. This prevents creating generated code containing a valid-looking but incorrect program id that could be accidentally deployed.

Note: Tests and CI that need to compile generated Anchor code should pass a real program id via `--address` (or set it in scripted invocations).

Why this matters:

- Prevents accidentally deploying code with a junk but valid-looking base58 id.
- Using a non-base58 sentinel makes the compiler fail with a clear error until a developer intentionally replaces it.

Example (embed program id):
```bash
lumos generate schema.lumos --address 5Hj3...xyz
```

Example (required):
```bash
lumos generate schema.lumos --address 5Hj3...xyz
```

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

Verify that generated code is up-to-date with the schema.

#### Usage

```bash
lumos check <SCHEMA_FILE> [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--output <DIR>` | Output directory to check (default: current directory) |

#### Examples

**Check if generated files match schema:**
```bash
lumos check schema.lumos
```

Output (when up-to-date):
```
Checking generated code status
 Success generated code is up-to-date
```

Output (when out-of-date):
```
warning: Generated code is out-of-date
  ./generated.rs
  ./generated.ts

Run: lumos generate schema.lumos
```

**Exit codes:**
- `0` - Generated code is up-to-date
- `1` - Generated code is out-of-date or missing

---

### `lumos check-size`

Analyze account sizes and detect Solana size limit violations.

#### Usage

```bash
lumos check-size <SCHEMA_FILE> [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Output format: `text` or `json` (default: text) |

#### Examples

**Basic size analysis:**
```bash
lumos check-size schema.lumos
```

Output:
```
Account Size Analysis:

✓ PlayerAccount: 110+ bytes (variable)
  ├─ discriminator (8 bytes) - Anchor account discriminator
  ├─ wallet (32 bytes) - PublicKey (32 bytes)
  ├─ level (2 bytes) - u16
  ├─ experience (8 bytes) - u64
  └─ Total: 110+ bytes (variable)
     Rent: 0.00000166 SOL

Summary:
  Total accounts: 1
  All accounts within limits ✓
```

**JSON output for CI/CD:**
```bash
lumos check-size schema.lumos --format json
```

Output:
```json
[
  {
    "name": "PlayerAccount",
    "total_bytes": 110,
    "is_variable": true,
    "is_account": true,
    "rent_sol": 0.00000166,
    "warnings": [],
    "fields": [...]
  }
]
```

**Size limit violation:**
```
⚠ LargeAccount: 12,582,912 bytes
  - Exceeds Solana's 10MB limit (12.00 MB)
  - Consider splitting into multiple accounts
```

**Exit codes:**
- `0` - All accounts within size limits
- `1` - One or more accounts exceed limits or have warnings

#### See Also

For detailed information about size calculation and optimization strategies, see [Account Size Guide](./security/account-size.md).

---

### `lumos security analyze`

Analyze schema for common Solana security vulnerabilities through static analysis.

#### Usage

```bash
lumos security analyze <SCHEMA_FILE> [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Output format: `text` or `json` (default: text) |
| `--strict` | Enable strict mode (more aggressive warnings) |

#### Examples

**Basic security analysis:**
```bash
lumos security analyze schema.lumos
```

Output:
```
Security Analysis Report
Schema: schema.lumos

Summary:
  🚨 1 critical issues
  ⚠️  3 warnings

CRITICAL ISSUES
════════════════════════════════════════════════════════════

🚨 [CRITICAL] Missing Signer Check
   Location: UpdateInstruction::authority
   Field 'authority' appears to be an authority but lacks explicit signer validation
   💡 Ensure this field requires signer validation in your Anchor program

Recommendations:
  🚨 Fix all critical issues before deployment
  ⚠️  Review and address warnings
```

**Strict mode (more warnings):**
```bash
lumos security analyze schema.lumos --strict
```

Enables additional checks:
- Owner validation warnings
- Integer overflow detection on all large integers
- Re-initialization vulnerability detection

**JSON output for CI/CD:**
```bash
lumos security analyze schema.lumos --format json
```

Output:
```json
[
  {
    "severity": "CRITICAL",
    "vulnerability_type": "Missing Signer Check",
    "location": {
      "type_name": "UpdateInstruction",
      "field_name": "authority"
    },
    "message": "Field 'authority' appears to be an authority...",
    "suggestion": "Ensure this field requires signer validation..."
  }
]
```

**Vulnerability types detected:**

| Severity | Type | Description |
|----------|------|-------------|
| 🚨 CRITICAL | Missing Signer Check | Authority fields without signer validation |
| ⚠️ WARNING | Unchecked Arithmetic | Arithmetic operations prone to overflow |
| ⚠️ WARNING | No Discriminator | Missing #[account] discriminator protection |
| ⚠️ WARNING | Missing Owner Validation | Owner fields without validation (strict mode) |
| ℹ️ INFO | Integer Overflow Risk | Large integers that may overflow (strict mode) |
| ⚠️ WARNING | Re-initialization Risk | Missing initialization flags (strict mode) |

**Exit codes:**
- `0` - No critical issues found
- `1` - One or more critical issues detected

#### See Also

For detailed information about vulnerability types and fixes, see [Static Analysis Guide](./security/static-analysis.md).

---

### `lumos audit generate`

Generate comprehensive security audit checklist from schema for manual code review.

#### Usage

```bash
lumos audit generate <SCHEMA_FILE> [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--output <PATH>` | Output file path (default: SECURITY_AUDIT.md) |
| `--format <FORMAT>` | Output format: `markdown` or `json` (default: markdown) |

#### Examples

**Basic checklist generation:**
```bash
lumos audit generate schema.lumos
```

Output:
```
Generated: SECURITY_AUDIT.md

Checklist includes:
  ✓ 39 total checks
  ✓ 6 account validation checks
  ✓ 1 signer checks
  ✓ 7 arithmetic safety checks
  ✓ 2 access control checks
```

**Custom output path:**
```bash
lumos audit generate schema.lumos --output my-audit.md
```

**JSON format for tools:**
```bash
lumos audit generate schema.lumos --format json --output audit.json
```

**Generated checklist structure:**

```markdown
# Security Audit Checklist

**Generated from:** `schema.lumos`
**Date:** 2025-11-22
**Total Checks:** 39

---

## 🔐 Account Validation

- [ ] 🔴 **Verify account ownership (program owns the account)**
  - Context: `PlayerAccount`
  - Ensure the account is owned by the program to prevent attacks...

## ✍️ Signer Checks

- [ ] 🔴 **Verify 'authority' field requires signer**
  - Context: `Config::authority`
  - Authority fields must validate that the transaction is signed...
```

**Check categories generated:**

| Category | Icon | Types of Checks |
|----------|------|-----------------|
| Account Validation | 🔐 | Ownership, discriminator, initialization, rent |
| Signer Checks | ✍️ | Authority field signer validation |
| Arithmetic Safety | 🔢 | Checked math, bounds validation |
| Access Control | 🚪 | Owner validation, authorization |
| Data Validation | ✅ | PublicKey checks, Vec/Option handling |
| State Transition | 🔄 | Valid transitions, reentrancy |
| Initialization | 🎬 | Account initialization checks |
| Rent Exemption | 💰 | Lamport balance checks |

**Priority levels:**
- 🔴 CRITICAL - Must fix before deployment
- 🟡 HIGH - Should be addressed
- 🟢 MEDIUM - Recommended to review
- ⚪ LOW - Best practice

**Exit codes:**
- `0` - Checklist generated successfully

#### See Also

For detailed information about using audit checklists and security review processes, see [Audit Checklist Guide](./security/audit-checklist.md).

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

### `lumos fuzz generate`

Generate fuzz targets for testing generated code with cargo-fuzz.

#### Usage

```bash
lumos fuzz generate <SCHEMA_FILE> [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--output <DIR>` | Output directory for fuzz targets (default: fuzz/) |
| `--type <NAME>` | Generate fuzz target for specific type only |

#### Examples

**Generate fuzz targets for all types:**
```bash
lumos fuzz generate schema.lumos
```

Output:
```
Generating fuzz targets...
Created fuzz/Cargo.toml
Created fuzz/README.md
Generated fuzz/fuzz_targets/fuzz_player_account.rs (for PlayerAccount)
Generated fuzz/fuzz_targets/fuzz_game_state.rs (for GameState)

✓ Generated 2 fuzz targets

Next steps:
  1. Install cargo-fuzz: cargo install cargo-fuzz
  2. Run fuzzing: cd fuzz && cargo fuzz run fuzz_player_account
```

**Generate for specific type:**
```bash
lumos fuzz generate schema.lumos --type PlayerAccount
```

**Custom output directory:**
```bash
lumos fuzz generate schema.lumos --output my-fuzz
```

#### Generated Structure

```
fuzz/
├── Cargo.toml                 # Fuzz project configuration
├── README.md                  # How to run fuzzing
└── fuzz_targets/              # Generated fuzz targets
    ├── fuzz_player_account.rs
    └── fuzz_game_state.rs
```

#### See Also

For detailed fuzzing guide, see [Fuzzing Documentation](./security/fuzzing.md).

---

### `lumos fuzz corpus`

Generate corpus files with valid serialized instances for fuzzing.

#### Usage

```bash
lumos fuzz corpus <SCHEMA_FILE> [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--output <DIR>` | Output directory for corpus (default: fuzz/corpus/) |
| `--type <NAME>` | Generate corpus for specific type only |

#### Examples

**Generate corpus for all types:**
```bash
lumos fuzz corpus schema.lumos
```

Output:
```
Generating corpus files...
Created fuzz/corpus/fuzz_player_account/player_account_minimal (50 bytes) - Minimal valid instance
Created fuzz/corpus/fuzz_player_account/player_account_maximal (50 bytes) - Maximal valid instance
Created fuzz/corpus/fuzz_game_state/game_state_active_variant (4 bytes) - Enum variant: Active

✓ Generated 8 corpus files
```

**Generate for specific type:**
```bash
lumos fuzz corpus schema.lumos --type PlayerAccount
```

#### Corpus Types Generated

| Type | Description |
|------|-------------|
| Minimal | Zero/default values |
| Maximal | Maximum values where applicable |
| Optional None | All Option fields set to None |
| Optional Some | All Option fields set to Some |
| Empty Vec | All Vec fields empty |
| Single Element Vec | All Vec fields with one element |
| Enum Variants | One file per variant |

---

### `lumos fuzz run`

Run fuzzing for a specific type (convenience wrapper around cargo-fuzz).

#### Usage

```bash
lumos fuzz run <SCHEMA_FILE> --type <NAME> [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--type <NAME>` | Type to fuzz (required) |
| `--jobs <N>` | Number of parallel jobs (default: 1) |
| `--max-time <SECONDS>` | Maximum run time in seconds |

#### Examples

**Basic fuzzing:**
```bash
lumos fuzz run schema.lumos --type PlayerAccount
```

**Parallel fuzzing (4 jobs):**
```bash
lumos fuzz run schema.lumos --type PlayerAccount --jobs 4
```

**Time-limited fuzzing (60 seconds):**
```bash
lumos fuzz run schema.lumos --type PlayerAccount --max-time 60
```

#### Direct cargo-fuzz Usage

You can also use `cargo fuzz` directly for more control:

```bash
cd fuzz
cargo fuzz run fuzz_player_account -- -jobs=4 -max_total_time=60
```

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
lumos check-size --help
lumos security analyze --help
lumos audit generate --help
lumos fuzz generate --help
lumos fuzz corpus --help
lumos fuzz run --help
```

**Resources:**
- Documentation: `docs/`
- Examples: `examples/`
- GitHub Issues: https://github.com/RECTOR-LABS/lumos/issues

---

**Last Updated:** 2025-11-22 (Fuzzing support added)
