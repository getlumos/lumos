# Account Size Overflow Detection

> Prevent runtime allocation failures and rent issues by analyzing account sizes at compile time.

## Overview

The `lumos check-size` command analyzes your schema and calculates the exact byte size of each account type, helping you:

- **Detect size limit violations** before deployment
- **Calculate rent costs** for account storage
- **Optimize account structure** for efficiency
- **Prevent runtime failures** from oversized accounts

## Solana Account Size Limits

| Limit | Size | Impact |
|-------|------|--------|
| Maximum account size | 10 MB | Hard limit - exceeding causes transaction failure |
| Recommended threshold | 1 MB | Warn at this size for optimization |
| Rent-exempt minimum | Variable | Based on account size |

## Usage

### Basic Analysis

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

### JSON Output

For programmatic use or CI/CD integration:

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
    "fields": [
      {
        "name": "discriminator",
        "bytes": 8,
        "is_variable": false,
        "description": "Anchor account discriminator"
      },
      ...
    ]
  }
]
```

## Size Calculation Rules

### Primitive Types

| Type | Size (bytes) |
|------|--------------|
| `u8`, `i8`, `bool` | 1 |
| `u16`, `i16` | 2 |
| `u32`, `i32`, `f32` | 4 |
| `u64`, `i64`, `f64` | 8 |
| `u128`, `i128` | 16 |
| `PublicKey` | 32 |
| `Signature` | 64 |
| `String` | 4 + UTF-8 bytes (variable) |

### Complex Types

| Type | Size Calculation |
|------|------------------|
| `Vec<T>` | 4 bytes (length prefix) + element sizes (variable) |
| `Option<T>` | 1 byte (discriminant) + size of T (if Some) |
| `[T; N]` | N × size of T (fixed array) |
| Enum | 1 byte (discriminant) + max variant size |
| Struct | Sum of all field sizes |

### Anchor Accounts

Structs with `#[account]` attribute automatically include an 8-byte discriminator:

```rust
#[solana]
#[account]
struct GameAccount {
    score: u64,  // 8 bytes
}
// Total: 8 (discriminator) + 8 (score) = 16 bytes
```

## Warnings and Errors

### Size Limit Exceeded

```
⚠ PlayerAccount: 12,582,912 bytes
  - Exceeds Solana's 10MB limit (12.00 MB)
  - Consider splitting into multiple accounts
```

**Solution:** Split large accounts into smaller related accounts using PDAs.

### Large Account Warning

```
⚠ LargeAccount: 1,048,576 bytes
  - Large account size (1024.00 KB)
  - Consider optimization
```

**Solution:** Review field necessity, use smaller types where possible, or normalize data.

### Variable Size Fields

Fields with variable sizes (Vec, String) show minimum size:

```
✓ PlayerAccount: 110+ bytes (variable)
  ├─ username (4+ bytes) - String (variable)
  ├─ inventory (4+ bytes) - Vec<PublicKey>
```

**Important:** Ensure your program handles max sizes appropriately to avoid allocation failures.

## Rent Calculation

Rent is calculated using Solana's rent formula:

```
rent_lamports = (account_size + 128) * 6.96
rent_sol = rent_lamports / 1,000,000,000
```

The calculation:
- Includes 128-byte overhead for account metadata
- Uses 6.96 lamports per byte (current rate)
- Converts to SOL for readability

### Example

```
PlayerAccount: 110 bytes
Rent: (110 + 128) * 6.96 = 1,656.48 lamports = 0.00000166 SOL
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Check Account Sizes
  run: |
    cargo install lumos-cli
    lumos check-size schema.lumos --format json > sizes.json

- name: Verify Size Limits
  run: |
    # Fail if any account exceeds 1MB
    if jq -e '.[] | select(.total_bytes > 1048576)' sizes.json; then
      echo "Account size exceeds 1MB threshold"
      exit 1
    fi
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

lumos check-size schema.lumos
if [ $? -ne 0 ]; then
  echo "Account size check failed. Fix warnings before committing."
  exit 1
fi
```

## Optimization Strategies

### 1. Use Smaller Types

```diff
#[account]
struct Player {
-   level: u64,        // 8 bytes
+   level: u16,        // 2 bytes (max 65,535)

-   health: u64,      // 8 bytes
+   health: u16,      // 2 bytes (max 65,535)
}
```

**Savings:** 12 bytes

### 2. Normalize Large Collections

Instead of:

```rust
#[account]
struct Player {
    // Bad: stores all items in one account
    inventory: Vec<Item>,  // Variable, could be huge
}
```

Use:

```rust
#[account]
struct Player {
    // Good: stores references to separate accounts
    inventory_items: Vec<PublicKey>,  // 4 + (32 * count) bytes
}

#[account]
struct Item {
    // Each item in its own account
    id: u64,
    name: String,
    // ...
}
```

### 3. Use Fixed Arrays When Possible

```rust
#[account]
struct Player {
    // Known max size
    equipped_items: [PublicKey; 5],  // Fixed: 160 bytes (5 * 32)

    // Instead of variable size
    // equipped_items: Vec<PublicKey>,  // Variable: 4+ bytes
}
```

### 4. Split Large Accounts

For complex game state:

```rust
// Instead of one large account:
#[account]
struct GameState {
    players: Vec<Player>,       // Could be huge
    leaderboard: Vec<Score>,    // Could be huge
    config: GameConfig,
}

// Use multiple accounts:
#[account]
struct PlayerRegistry {
    count: u32,
    // Store player addresses, not full data
}

#[account]
struct PlayerData {
    // Each player in separate account
    wallet: PublicKey,
    stats: PlayerStats,
}

#[account]
struct Leaderboard {
    // Separate account for leaderboard
    top_scores: [Score; 100],  // Fixed size
}
```

## Best Practices

1. **Run `check-size` early and often**
   - Before writing Anchor program code
   - After schema changes
   - In CI/CD pipelines

2. **Plan for growth**
   - Consider future fields when designing schemas
   - Leave some buffer below the 10MB limit
   - Use variable-size fields judiciously

3. **Monitor production accounts**
   - Track actual sizes vs. calculated minimums
   - Alert on accounts approaching limits
   - Plan migrations before hitting limits

4. **Document size assumptions**
   - Comment expected max sizes for Vec fields
   - Validate sizes in program logic
   - Test with realistic data volumes

## Examples

### Gaming Inventory

```bash
$ lumos check-size examples/gaming/schema.lumos

✓ PlayerAccount: 110+ bytes (variable)
  ├─ discriminator (8 bytes)
  ├─ wallet (32 bytes)
  ├─ equipped_items (4+ bytes) - Vec<PublicKey>
  ├─ inventory_items (4+ bytes) - Vec<PublicKey>
  └─ Total: 110+ bytes (variable)
     Rent: 0.00000166 SOL
```

### NFT Marketplace

```bash
$ lumos check-size examples/nft-marketplace/schema.lumos

✓ Listing: 121 bytes
  ├─ discriminator (8 bytes)
  ├─ seller (32 bytes)
  ├─ nft_mint (32 bytes)
  ├─ price (8 bytes)
  ├─ created_at (8 bytes)
  └─ Total: 121 bytes
     Rent: 0.00000173 SOL
```

## Troubleshooting

### "Exceeds Solana's 10MB limit"

**Problem:** Account size calculation exceeds 10,485,760 bytes.

**Solutions:**
1. Split data across multiple accounts using PDAs
2. Use references (PublicKeys) instead of embedded data
3. Store large data off-chain (IPFS, Arweave) with on-chain references

### "Large account size" warning

**Problem:** Account is over 1MB but under 10MB.

**Solutions:**
1. Review if all fields are necessary
2. Use smaller integer types (u16 vs u64)
3. Consider normalization strategies
4. Use fixed-size arrays with known maximums

### Variable size shows "4+ bytes"

**Problem:** Vec or String shows minimum size only.

**Solutions:**
1. Set maximum sizes in your program logic
2. Use fixed-size arrays if max is known
3. Validate sizes on-chain before allocation
4. Document expected maximums in code comments

## See Also

- [Solana Account Model](https://docs.solana.com/developing/programming-model/accounts)
- [Rent Economics](https://docs.solana.com/implemented-proposals/rent)
- [Anchor Account Documentation](https://docs.rs/anchor-lang/latest/anchor_lang/attr.account.html)
- [Static Analysis Guide](./static-analysis.md) (coming soon)
