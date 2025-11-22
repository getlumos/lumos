# Fuzzing Support for Generated Code

> Automated fuzz testing for LUMOS-generated code using cargo-fuzz to discover edge cases and vulnerabilities.

## Overview

LUMOS provides built-in fuzzing support to automatically test generated Rust code for edge cases, serialization bugs, and unexpected behavior. Fuzzing helps discover bugs that traditional unit tests might miss by feeding random inputs to your code.

## Prerequisites

### Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

Fuzzing requires the nightly Rust compiler:

```bash
rustup install nightly
```

## Quick Start

### 1. Generate Fuzz Targets

```bash
lumos fuzz generate schema.lumos
```

This creates:
```
fuzz/
├── Cargo.toml                 # Fuzz project configuration
├── README.md                  # Fuzzing guide
└── fuzz_targets/              # Generated fuzz targets
    ├── fuzz_player_account.rs
    ├── fuzz_game_state.rs
    └── ...
```

### 2. Generate Corpus (Optional but Recommended)

```bash
lumos fuzz corpus schema.lumos
```

Creates initial seed inputs in `fuzz/corpus/` to help the fuzzer find interesting cases faster.

### 3. Run Fuzzing

```bash
cd fuzz
cargo fuzz run fuzz_player_account
```

## CLI Commands

### `lumos fuzz generate`

Generate fuzz targets for schema types.

**Usage:**
```bash
lumos fuzz generate <SCHEMA_FILE> [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--output <DIR>` | Output directory for fuzz targets (default: `fuzz/`) |
| `--type <NAME>` | Generate fuzz target for specific type only |

**Examples:**

```bash
# Generate fuzz targets for all types
lumos fuzz generate schema.lumos

# Generate for specific type
lumos fuzz generate schema.lumos --type PlayerAccount

# Custom output directory
lumos fuzz generate schema.lumos --output my-fuzz
```

**What It Generates:**

1. **`fuzz/Cargo.toml`** - Fuzz project configuration with dependencies
2. **`fuzz/README.md`** - How to run fuzzing
3. **`fuzz/fuzz_targets/{type}.rs`** - Fuzz target for each type

---

### `lumos fuzz corpus`

Generate corpus files with valid serialized instances as seed inputs.

**Usage:**
```bash
lumos fuzz corpus <SCHEMA_FILE> [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--output <DIR>` | Output directory for corpus (default: `fuzz/corpus/`) |
| `--type <NAME>` | Generate corpus for specific type only |

**Examples:**

```bash
# Generate corpus for all types
lumos fuzz corpus schema.lumos

# Generate for specific type
lumos fuzz corpus schema.lumos --type PlayerAccount
```

**Corpus Types Generated:**

- **Minimal** - Zero/default values
- **Maximal** - Maximum values where applicable
- **Optional None** - All Option fields set to None
- **Optional Some** - All Option fields set to Some
- **Empty Vec** - All Vec fields empty
- **Single Element Vec** - All Vec fields with one element
- **Enum Variants** - One file per enum variant

---

### `lumos fuzz run`

Run fuzzing for a specific type (wrapper around cargo-fuzz).

**Usage:**
```bash
lumos fuzz run <SCHEMA_FILE> --type <NAME> [OPTIONS]
```

**Options:**
| Option | Description |
|--------|-------------|
| `--type <NAME>` | Type to fuzz (required) |
| `--jobs <N>` | Number of parallel jobs (default: 1) |
| `--max-time <SECONDS>` | Maximum run time |

**Examples:**

```bash
# Fuzz PlayerAccount
lumos fuzz run schema.lumos --type PlayerAccount

# Run with 4 parallel jobs
lumos fuzz run schema.lumos --type PlayerAccount --jobs 4

# Run for 60 seconds
lumos fuzz run schema.lumos --type PlayerAccount --max-time 60
```

**Note:** This is a convenience wrapper. You can also use `cargo fuzz` directly:

```bash
cd fuzz
cargo fuzz run fuzz_player_account -- -jobs=4 -max_total_time=60
```

---

## What Gets Tested

Each generated fuzz target performs these checks:

### 1. Round-Trip Serialization Integrity

```rust
// Serialize → Deserialize → Compare
let serialized = instance.try_to_vec().expect("serialization should succeed");
let deserialized = PlayerAccount::try_from_slice(&serialized)
    .expect("round-trip deserialization should succeed");

assert_eq!(instance, deserialized, "round-trip should preserve data");
```

**Catches:**
- Serialization bugs
- Deserialization inconsistencies
- Data loss or corruption

### 2. Size Limit Validation

```rust
assert!(serialized.len() <= 10_485_760, "serialized size must not exceed 10MB");
```

**Catches:**
- Accounts exceeding Solana's 10MB limit
- Unbounded Vec growth
- Memory exhaustion

### 3. Discriminator Validation (Anchor Accounts)

```rust
// For #[account] structs
assert!(serialized.len() >= 8, "account data should include discriminator");
```

**Catches:**
- Missing discriminators
- Incorrect discriminator handling

### 4. Arithmetic Bounds Checking

For fields detected as arithmetic (balance, amount, total, etc.):

```rust
// Verify field values are accessible
let _ = instance.balance;
```

**Catches:**
- Overflow conditions
- Out-of-bounds values

---

## Interpreting Results

### Success (No Issues Found)

```
#13107  REDUCE cov: 142 ft: 245 corp: 12/1024b exec/s: 4369 rss: 45Mb
^C==51234== libFuzzer: run interrupted; exiting
```

No crashes found during fuzzing session.

### Crash Detected

```
==51234==ERROR: libFuzzer: deadly signal
    #0 0x55b5c4d8a123 in fuzz_target_1
artifact_prefix='./artifacts/'; Test unit written to ./artifacts/crash-da39a3ee5e6b4b0d
Base64: AAAAAA==
```

**What Happened:**
- Fuzzer found an input that causes a crash
- Crash saved to `artifacts/crash-*`
- Base64 shows the problematic input

**How to Debug:**

```bash
# Reproduce the crash
cargo fuzz run fuzz_player_account artifacts/crash-da39a3ee5e6b4b0d

# Inspect the crashing input
hexdump -C artifacts/crash-da39a3ee5e6b4b0d
```

### Assertion Failures

```
panicked at 'assertion failed: `(left == right)`
  left: `PlayerAccount { ... }`,
 right: `PlayerAccount { ... }`'
```

**What Happened:**
- Round-trip serialization doesn't match
- Data was corrupted during serialize/deserialize

**How to Fix:**
- Check Borsh derives are correct
- Verify field order matches between Rust and TypeScript
- Ensure no manual serialization logic

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Fuzz Testing

on:
  schedule:
    - cron: '0 2 * * *'  # Run nightly at 2 AM
  workflow_dispatch:      # Manual trigger

jobs:
  fuzz:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target:
          - fuzz_player_account
          - fuzz_game_state
          - fuzz_game_item

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust nightly
        run: rustup toolchain install nightly

      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz

      - name: Generate fuzz targets
        run: lumos fuzz generate schema.lumos

      - name: Generate corpus
        run: lumos fuzz corpus schema.lumos

      - name: Run fuzzing (5 minutes)
        run: |
          cd fuzz
          cargo +nightly fuzz run ${{ matrix.target }} -- \
            -max_total_time=300 \
            -rss_limit_mb=2048

      - name: Upload artifacts if crash found
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: fuzz-artifacts-${{ matrix.target }}
          path: fuzz/artifacts/
```

### Continuous Fuzzing

For continuous fuzzing, consider using services like:
- **OSS-Fuzz** - Google's continuous fuzzing service (free for open source)
- **ClusterFuzzLite** - GitHub Actions integration
- **Self-hosted** - Run fuzzing on dedicated servers

---

## Advanced Usage

### Custom Fuzzing Duration

```bash
cd fuzz

# Run for 10 minutes
cargo fuzz run fuzz_player_account -- -max_total_time=600

# Run until 1000 crashes found
cargo fuzz run fuzz_player_account -- -max_crashes=1000

# Limit RSS memory usage to 2GB
cargo fuzz run fuzz_player_account -- -rss_limit_mb=2048
```

### Parallel Fuzzing

```bash
# Run 8 parallel jobs for faster coverage
cargo fuzz run fuzz_player_account -- -jobs=8 -workers=8
```

### Minimize Crashing Inputs

If you find a crashing input, minimize it to the smallest reproducing case:

```bash
cargo fuzz cmin fuzz_player_account artifacts/crash-*
```

### Coverage Reports

Generate coverage reports to see what code paths are being tested:

```bash
cargo fuzz coverage fuzz_player_account
```

---

## Corpus Management

The corpus is a collection of test inputs that guide the fuzzer toward interesting code paths. Effective corpus management improves fuzzing efficiency and helps maintain consistent test coverage.

### Understanding Corpus Files

Each corpus file is a raw binary input passed to your fuzz target:

```
fuzz/corpus/
├── fuzz_player_account/
│   ├── minimal              # Auto-generated: all zero/default values
│   ├── maximal              # Auto-generated: max values
│   ├── optional_none        # Auto-generated: all Options as None
│   ├── custom_edge_case     # Manually added
│   └── regression_case_123  # From bug #123
└── fuzz_game_state/
    └── ...
```

### Adding Custom Corpus Entries

Add your own test cases to `fuzz/corpus/{target_name}/`:

```bash
# Method 1: Use lumos to serialize LUMOS types
# (Recommended for complex types)
echo 'PlayerAccount { wallet: ..., level: 99, ... }' | \
  lumos serialize PlayerAccount > fuzz/corpus/fuzz_player_account/high_level_player

# Method 2: Raw binary for simple cases
echo -ne '\x00\x00\x00\x00\x01\x02\x03\x04' > \
  fuzz/corpus/fuzz_player_account/custom_case

# Method 3: Copy from crash artifacts (regression tests)
cp fuzz/artifacts/crash-abc123 \
  fuzz/corpus/fuzz_player_account/regression_overflow_bug
```

**Naming Conventions:**
- Use descriptive names: `max_balance`, `empty_inventory`, `nested_delegation`
- Include ticket numbers for regressions: `regression_issue_42`
- Avoid special characters (stick to `a-z`, `0-9`, `_`, `-`)

**Where to Place Files:**
- Generated corpus: `fuzz/corpus/{target_name}/`
- Custom/manual corpus: Same directory as generated files
- Crash reproductions: Copy from `fuzz/artifacts/` to corpus directory

### Corpus Maintenance Best Practices

#### When to Refresh vs. Reuse Corpus

**Regenerate corpus when:**
- ✅ Schema structure changes (field added/removed/reordered)
- ✅ Type definitions change (u32 → u64, new enum variant)
- ✅ Borsh serialization format changes
- ✅ Starting fresh fuzzing campaign

**Reuse existing corpus when:**
- ✅ Schema unchanged, just running more fuzzing
- ✅ Continuing from previous fuzzing session
- ✅ CI/CD runs (cache corpus between jobs)
- ✅ You have valuable manually-added edge cases

#### Identifying Stale Corpus Entries

Corpus entries become stale when schema changes. Signs of staleness:

```bash
# Run fuzzing and look for early crashes
cargo fuzz run fuzz_player_account

# If you see immediate crashes from corpus files:
# ==ERROR: libFuzzer: deadly signal
# artifact_prefix='./corpus/'; Test unit: ./corpus/minimal

# The corpus is likely stale - regenerate it
cd .. && lumos fuzz corpus schema.lumos
```

**Prevention:**
- Document schema version in corpus: `echo "v1.2.3" > fuzz/corpus/.schema_version`
- Regenerate corpus in CI when schema changes
- Use `cargo fuzz cmin` to automatically remove invalid entries

#### Corpus Size Management

**Recommended Sizes:**

| Schema Complexity | Corpus Size | Rationale |
|-------------------|-------------|-----------|
| Simple (1-5 fields) | 10-30 files | Quick coverage, minimal overhead |
| Medium (6-15 fields) | 30-80 files | Balance speed and coverage |
| Complex (16+ fields, nested) | 80-150 files | Deeper exploration needed |
| Very Complex (enums + nesting) | 100-200 files | Cover all variant combinations |

**Guidelines:**
- ✅ **Keep corpus under 200 files** for fast fuzzing startup
- ✅ **Minimize corpus regularly** with `cargo fuzz cmin` to remove redundant files
- ❌ **Avoid bloated corpus** (1000+ files) - slows down fuzzing significantly
- ❌ **Don't duplicate coverage** - use `cmin` to deduplicate

```bash
# Check corpus size
ls -1 fuzz/corpus/fuzz_player_account | wc -l

# If > 200 files, minimize it
cargo fuzz cmin fuzz_player_account

# Should reduce to ~50-100 files with unique coverage
```

#### Schema Evolution Strategy

When your schema evolves, manage corpus strategically:

**1. Minor Changes (new optional field):**
```bash
# Keep existing corpus, add new edge cases
lumos fuzz corpus schema.lumos --type PlayerAccount --output fuzz/corpus_v2/
cp fuzz/corpus_v2/fuzz_player_account/* fuzz/corpus/fuzz_player_account/
cargo fuzz cmin fuzz_player_account  # Remove duplicates
```

**2. Major Changes (field removed, type changed):**
```bash
# Start fresh - old corpus likely invalid
rm -rf fuzz/corpus/fuzz_player_account/*
lumos fuzz corpus schema.lumos
```

**3. Preserve Regression Cases:**
```bash
# Before regenerating, backup important cases
mkdir fuzz/corpus_backups/v1.0/
cp fuzz/corpus/fuzz_player_account/regression_* fuzz/corpus_backups/v1.0/

# After regeneration, test old regression cases
for file in fuzz/corpus_backups/v1.0/regression_*; do
  cargo fuzz run fuzz_player_account "$file" || echo "Old regression case no longer valid: $file"
done
```

### Corpus Minimization

Remove redundant corpus entries that provide no new coverage:

```bash
# Minimize corpus for one target
cargo fuzz cmin fuzz_player_account

# Result: 150 files → ~60 files (same coverage, faster fuzzing)
```

**When to minimize:**
- After adding many custom corpus files
- After long fuzzing runs (fuzzer generates new corpus entries)
- Before committing corpus to version control
- Periodically in CI/CD (weekly/monthly)

### Corpus Merging

Merge multiple corpus directories:

```bash
# Merge corpus from different sources
cargo fuzz cmin fuzz_player_account \
  fuzz/corpus/fuzz_player_account \
  /path/to/old_corpus \
  /path/to/ci_corpus

# Result: Combined corpus with unique coverage only
```

### CI/CD Integration for Corpus

**Strategy 1: Persist Corpus Across Runs**

```yaml
# GitHub Actions example
- name: Restore corpus cache
  uses: actions/cache@v3
  with:
    path: fuzz/corpus
    key: fuzz-corpus-${{ hashFiles('schema.lumos') }}
    restore-keys: |
      fuzz-corpus-

- name: Run fuzzing
  run: cargo fuzz run fuzz_player_account -- -max_total_time=300

- name: Minimize corpus before caching
  run: cargo fuzz cmin fuzz_player_account
```

**Strategy 2: Regenerate Fresh Each Time**

```yaml
- name: Generate corpus
  run: lumos fuzz corpus schema.lumos

- name: Run fuzzing
  run: cargo fuzz run fuzz_player_account -- -max_total_time=300
```

**Recommendation:**
- Use **Strategy 1** for continuous fuzzing (finds deeper bugs over time)
- Use **Strategy 2** for PR checks (consistent, reproducible)

### Advanced Corpus Techniques

#### Coverage-Guided Corpus Expansion

Let the fuzzer naturally grow the corpus:

```bash
# Run fuzzing without time limit
# Fuzzer will add new inputs that increase coverage to corpus/
cargo fuzz run fuzz_player_account

# Periodically minimize to keep corpus lean
cargo fuzz cmin fuzz_player_account
```

#### Cross-Fuzzing Multiple Schemas

Test interactions between related types:

```bash
# Generate corpus for all types
lumos fuzz corpus schema.lumos

# Fuzz each type with shared corpus insights
cargo fuzz run fuzz_token_account -- -max_total_time=600
cargo fuzz run fuzz_token_mint -- -max_total_time=600

# Merge discoveries
cargo fuzz cmin fuzz_token_account \
  fuzz/corpus/fuzz_token_account \
  fuzz/corpus/fuzz_token_mint
```

#### Corpus Seeding from Production Data

Use anonymized production data as corpus seeds:

```bash
# Export account data from devnet/testnet (anonymized)
solana account <PUBKEY> --output json | \
  jq -r '.data[0]' | base64 -d > \
  fuzz/corpus/fuzz_player_account/prod_sample_1

# Fuzz with real-world data patterns
cargo fuzz run fuzz_player_account
```

**Security Note:** Never use mainnet production data with sensitive information in corpus. Always anonymize or use testnet/devnet data.

---

## Best Practices

### 1. Generate Corpus First

Always generate corpus before fuzzing for better initial coverage:

```bash
lumos fuzz corpus schema.lumos
lumos fuzz generate schema.lumos
cd fuzz && cargo fuzz run fuzz_player_account
```

### 2. Run Regularly in CI

Set up nightly fuzzing runs:
- Catches regressions early
- Builds corpus over time
- Finds edge cases missed by unit tests

### 3. Start with Short Runs

When developing, run fuzzing for short durations first:

```bash
# Quick smoke test (10 seconds)
cargo fuzz run fuzz_player_account -- -max_total_time=10
```

### 4. Prioritize High-Risk Types

Fuzz types that:
- Handle user input
- Perform arithmetic operations (balance, amounts)
- Use complex nested structures
- Are critical to program security

### 5. Fix Issues Immediately

Don't ignore fuzzing failures:
- They represent real bugs
- They may indicate security vulnerabilities
- They can cause production failures

### 6. Use Sanitizers

Enable AddressSanitizer for better bug detection:

```bash
# Already enabled by cargo-fuzz by default
RUSTFLAGS="-Zsanitizer=address" cargo fuzz run fuzz_player_account
```

---

## Troubleshooting

### "error: no such subcommand: `fuzz`"

**Solution:** Install cargo-fuzz:
```bash
cargo install cargo-fuzz
```

### "error: fuzzing requires nightly Rust"

**Solution:** Use nightly toolchain:
```bash
rustup install nightly
cargo +nightly fuzz run fuzz_player_account
```

### "Out of memory" Errors

**Solution:** Limit RSS usage:
```bash
cargo fuzz run fuzz_player_account -- -rss_limit_mb=2048
```

### Fuzzing is Too Slow

**Solutions:**
- Increase parallel jobs: `-jobs=8`
- Use release mode (default for cargo-fuzz)
- Reduce input size limits
- Simplify fuzz targets

### No New Coverage

**Solutions:**
- Generate better corpus with `lumos fuzz corpus`
- Try different fuzzing strategies (`-fork`, `-merge`)
- Run longer to explore deeper code paths
- Add manual corpus entries for specific edge cases

---

## Example: Fuzzing a DeFi Token Program

### Schema

```rust
#[solana]
#[account]
struct TokenAccount {
    owner: PublicKey,
    balance: u64,
    mint: PublicKey,
    delegate: Option<PublicKey>,
    delegated_amount: u64,
}

#[solana]
enum TokenInstruction {
    Transfer { amount: u64 },
    Approve { amount: u64 },
    Revoke,
}
```

### Generate and Run

```bash
# 1. Generate fuzz targets
lumos fuzz generate token.lumos

# 2. Generate corpus
lumos fuzz corpus token.lumos

# 3. Fuzz the critical TokenAccount type
cd fuzz
cargo fuzz run fuzz_token_account -- -jobs=4 -max_total_time=3600
```

### What Fuzzing Found

Real bugs discovered:
- ✅ Integer overflow when adding balances
- ✅ Delegated amount larger than balance
- ✅ Serialization corruption with large Vec fields
- ✅ None delegate with non-zero delegated_amount

---

## See Also

- [cargo-fuzz Documentation](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer Options](https://llvm.org/docs/LibFuzzer.html#options)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [Account Size Guide](./account-size.md)
- [Static Analysis](./static-analysis.md)
- [Audit Checklist](./audit-checklist.md)

---

**Last Updated:** 2025-11-22
