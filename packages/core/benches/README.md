# LUMOS Performance Benchmarks

Comprehensive benchmarking suite for LUMOS core components using [Criterion.rs](https://github.com/bheisler/criterion.rs).

## Overview

Measures performance across the entire LUMOS pipeline:

- **Parser**: `.lumos` file → AST (small, medium, large schemas)
- **Transform**: AST → IR (3 schema sizes)
- **Rust Generator**: IR → Rust code (3 schema sizes)
- **TypeScript Generator**: IR → TypeScript code (3 schema sizes)
- **End-to-End**: Complete pipeline (parse → transform → generate)

## Running Benchmarks

### All Benchmarks

```bash
cd packages/core
cargo bench
```

### Specific Benchmark Group

```bash
# Parser only
cargo bench --bench benchmarks parser

# Transform only
cargo bench --bench benchmarks transform

# Rust generator only
cargo bench --bench benchmarks rust_gen

# TypeScript generator only
cargo bench --bench benchmarks typescript_gen

# End-to-end pipeline only
cargo bench --bench benchmarks e2e
```

### Single Benchmark

```bash
# Example: Parser with small schema
cargo bench --bench benchmarks parser_small_schema
```

## Benchmark Results

Results are saved to `target/criterion/` with:

- **HTML Reports**: `target/criterion/*/report/index.html`
- **Statistical Analysis**: Mean, median, standard deviation
- **Comparison**: Automatically compares with previous runs

## Test Schemas

### Small Schema (2 fields, 1 struct)
```lumos
#[solana]
#[account]
struct User {
    id: u64,
    name: String,
}
```

### Medium Schema (3 structs, 1 enum)
- `PlayerAccount` (4 fields)
- `MatchResult` (4 fields)
- `GameState` enum (3 variants)

### Large Schema (4 structs, 4 enums)
- `DAOConfig`, `Proposal`, `Vote`, `Member` structs
- `VoteType`, `ProposalStatus`, `GameEvent`, `GameInstruction` enums
- Demonstrates complex enum variants (unit, tuple, struct)

## Expected Performance

**Approximate baseline (Apple M1/M2):**

| Component | Small | Medium | Large |
|-----------|-------|--------|-------|
| Parser | ~5 µs | ~15 µs | ~40 µs |
| Transform | ~2 µs | ~8 µs | ~20 µs |
| Rust Gen | ~3 µs | ~10 µs | ~30 µs |
| TypeScript Gen | ~3 µs | ~10 µs | ~30 µs |
| E2E Pipeline | ~13 µs | ~43 µs | ~120 µs |

**Note:** Actual times vary by CPU, system load, and schema complexity.

## Interpreting Results

### Criterion Output

```
parser_small_schema     time:   [4.6630 µs 4.9364 µs 5.3296 µs]
                        change: [-2.5% -0.1% +2.3%] (no significant change)
```

- **Middle value (4.9364 µs)**: Best estimate of mean time
- **Range [4.6630 - 5.3296]**: 95% confidence interval
- **change**: Performance change vs. previous run

### Outliers

```
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
```

- **Mild outliers**: Slightly slower than expected (acceptable)
- **Severe outliers**: Much slower (often due to system interruptions)
- **Normal**: <5% outliers expected; >15% suggests instability

## Optimization Guidelines

### When to Investigate Performance

- **>50 µs for small schemas**: Possible inefficiency in parser/generators
- **>200 µs for E2E large**: Check for unnecessary allocations
- **High variance**: Reduce outliers by closing other apps during benchmarks

### Common Optimizations

1. **Parser**: Minimize allocations, reuse `syn` structures
2. **Transform**: Avoid cloning large IR structures
3. **Generators**: Use `String::with_capacity` for code generation
4. **Overall**: Profile with `cargo flamegraph` for hotspots

## Benchmark Methodology

- **Warm-up**: 3 seconds per benchmark
- **Samples**: 100 measurements
- **Iterations**: Adjusted automatically by Criterion
- **Black Box**: `black_box()` prevents compiler optimizations

## CI Integration

Benchmarks are **not** run in CI due to:
- Hardware variance across runners
- Time constraints (benchmarks take ~10 minutes)
- Non-deterministic results

**Recommendation:** Run benchmarks locally before performance-critical PRs.

## Tracking Performance Over Time

Criterion automatically saves baseline results. Compare against baseline:

```bash
# Run benchmarks and save as baseline
cargo bench -- --save-baseline my-baseline

# Later, compare against baseline
cargo bench -- --baseline my-baseline
```

## Troubleshooting

### "Gnuplot not found"

Criterion falls back to plotters backend (pure Rust). To get native gnuplot charts:

```bash
# macOS
brew install gnuplot

# Ubuntu
sudo apt install gnuplot

# Then re-run benchmarks
cargo bench
```

### Inconsistent Results

- Close resource-intensive apps
- Disable power-saving mode
- Run multiple times and check variance
- Use `--sample-size 200` for more samples

## Adding New Benchmarks

1. Add test schema constant to `benchmarks.rs`
2. Create benchmark function:
   ```rust
   fn bench_my_feature(c: &mut Criterion) {
       c.bench_function("my_feature_name", |b| {
           b.iter(|| {
               // Code to benchmark
               black_box(expensive_operation())
           })
       });
   }
   ```
3. Add to `criterion_group!` macro
4. Run: `cargo bench`

---

**Last Updated:** 2025-11-18
**Criterion Version:** 0.5.1
**Benchmarks:** 15 total (3 per component × 5 components)
