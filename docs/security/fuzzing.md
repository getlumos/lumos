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

### Adding Custom Corpus Entries

Add your own test cases to `fuzz/corpus/{target_name}/`:

```bash
# Create a custom corpus file
echo -ne '\x00\x00\x00\x00\x01\x02\x03\x04' > fuzz/corpus/fuzz_player_account/custom_case
```

### Corpus Minimization

Remove redundant corpus entries:

```bash
cargo fuzz cmin fuzz_player_account
```

### Corpus Merging

Merge multiple corpus directories:

```bash
cargo fuzz cmin fuzz_player_account corpus1 corpus2 corpus3
```

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
