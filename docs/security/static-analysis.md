# Static Analysis for Solana Vulnerabilities

> Detect common Solana security vulnerabilities before deployment through automated static analysis.

## Overview

The `lumos security analyze` command performs static analysis on your schema to identify potential security vulnerabilities in Solana programs. It detects common attack vectors and provides actionable recommendations.

## Usage

### Basic Analysis

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

### Strict Mode

Enable more aggressive warnings:

```bash
lumos security analyze schema.lumos --strict
```

Strict mode enables additional checks:
- Owner validation warnings
- Integer overflow detection on all large integers
- Re-initialization vulnerability detection

### JSON Output

For CI/CD integration:

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

## Vulnerability Types

### 🚨 CRITICAL

#### Missing Signer Check

**Description:** Fields that appear to be authority/admin fields but lack explicit signer validation.

**Detected Keywords:** `authority`, `admin`, `owner`, `signer`, `payer`, `creator`, `minter`, `updater`

**Example:**

```lumos
// ❌ Vulnerable
#[solana]
struct UpdateInstruction {
    authority: PublicKey,  // No signer validation!
    new_data: String,
}
```

**Fix in Anchor:**

```rust
// ✅ Secure
#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,  // Enforces signer check
    pub account: Account<'info, MyAccount>,
}
```

**Impact:** Unauthorized users can perform privileged operations.

---

### ⚠️  WARNINGS

#### Unchecked Arithmetic

**Description:** Fields prone to arithmetic operations without checked math.

**Detected Fields:** `balance`, `amount`, `supply`, `total`, `count`, `price`, `value`, `reward`, `stake`, `fee`, `lamport` (+ u64/u128/i64/i128 types)

**Example:**

```lumos
// ⚠️  Warning
#[solana]
#[account]
struct TokenAccount {
    balance: u64,  // Arithmetic operations risky
}
```

**Fix:**

```rust
// ✅ Use checked arithmetic
impl TokenAccount {
    pub fn transfer(&mut self, amount: u64) -> Result<()> {
        self.balance = self.balance
            .checked_sub(amount)
            .ok_or(ErrorCode::InsufficientFunds)?;
        Ok(())
    }
}
```

**Impact:** Integer overflow/underflow can lead to loss of funds or incorrect state.

#### No Discriminator

**Description:** Struct marked `#[solana]` but missing `#[account]` attribute.

**Example:**

```lumos
// ⚠️  Warning
#[solana]
struct GameData {  // Missing #[account]!
    score: u64,
}
```

**Fix:**

```lumos
// ✅ Secure
#[solana]
#[account]  // Adds 8-byte discriminator
struct GameData {
    score: u64,
}
```

**Impact:** Type confusion attacks where one account type is mistaken for another.

#### Missing Owner Validation (Strict Mode)

**Description:** Owner field without explicit validation reminder.

**Example:**

```lumos
// ⚠️  Warning (strict mode)
#[solana]
#[account]
struct Asset {
    owner: PublicKey,
}
```

**Fix:**

```rust
// ✅ Validate owner
require!(
    ctx.accounts.asset.owner == ctx.accounts.signer.key(),
    ErrorCode::Unauthorized
);
```

**Impact:** Unauthorized access to owned resources.

---

### ℹ️  INFORMATIONAL (Strict Mode)

#### Integer Overflow Risk

**Description:** Large integer types (u64, u128, i64, i128) that may require overflow protection.

**Example:**

```lumos
// ℹ️  Info (strict mode)
#[solana]
#[account]
struct LargeData {
    counter: u128,  // Very large, consider overflow
}
```

**Suggestion:** Use saturating or checked operations.

#### Re-initialization Risk (Strict Mode)

**Description:** Account lacks explicit initialization flag.

**Example:**

```lumos
// ⚠️  Warning (strict mode)
#[solana]
#[account]
struct Config {
    admin: PublicKey,
    // No is_initialized field!
}
```

**Fix:**

```lumos
// ✅ Add initialization flag
#[solana]
#[account]
struct Config {
    is_initialized: bool,
    admin: PublicKey,
}
```

Or use Anchor's `init` constraint:

```rust
#[account(
    init,
    payer = authority,
    space = 8 + 32
)]
pub config: Account<'info, Config>,
```

**Impact:** Account can be re-initialized, overwriting critical data.

---

## Severity Levels

| Level | Symbol | Exit Code | Meaning |
|-------|--------|-----------|---------|
| CRITICAL | 🚨 | 1 | Security vulnerability that must be fixed |
| WARNING | ⚠️ | 0 | Potential issue that should be reviewed |
| INFO | ℹ️ | 0 | Best practice recommendation |

## CI/CD Integration

### GitHub Actions

```yaml
name: Security Analysis

on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install LUMOS CLI
        run: cargo install lumos-cli

      - name: Run Security Analysis
        run: |
          lumos security analyze schema.lumos --format json > security.json

      - name: Check for Critical Issues
        run: |
          if jq -e '.[] | select(.severity == "CRITICAL")' security.json; then
            echo "::error::Critical security issues found"
            exit 1
          fi

      - name: Upload Security Report
        uses: actions/upload-artifact@v3
        with:
          name: security-report
          path: security.json
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running security analysis..."
lumos security analyze schema.lumos

if [ $? -ne 0 ]; then
  echo "❌ Security issues found. Fix critical issues before committing."
  echo "Or use: git commit --no-verify (not recommended)"
  exit 1
fi

echo "✓ Security checks passed"
```

Make executable:
```bash
chmod +x .git/hooks/pre-commit
```

### CI/CD Best Practices

1. **Fail on Critical:** Always fail the build on critical findings
2. **Report Warnings:** Log warnings but don't block deployment
3. **Track Over Time:** Store security reports as artifacts
4. **Audit Trail:** Document why warnings are accepted
5. **Regular Scans:** Run on every commit and PR

## Common Patterns

### DeFi Token Program

```lumos
#[solana]
#[account]
struct TokenAccount {
    owner: PublicKey,           // ⚠️  Validate in program
    balance: u64,               // ⚠️  Use checked arithmetic
    is_initialized: bool,       // ✅ Prevents re-init
}

#[solana]
struct TransferInstruction {
    from_authority: PublicKey,  // 🚨 CRITICAL: Needs signer check
    amount: u64,                // ⚠️  Use checked math
}
```

**Fixes Required:**
1. Add signer validation for `from_authority`
2. Use `checked_sub`/`checked_add` for balance operations
3. Validate `owner` matches transaction signer

### NFT Minting

```lumos
#[solana]
#[account]
struct NFTMint {
    authority: PublicKey,       // 🚨 CRITICAL: Needs signer check
    supply: u64,                // ⚠️  Use checked_add for minting
    is_mutable: bool,
}
```

**Fixes Required:**
1. Enforce signer check on `authority`
2. Use `checked_add` when incrementing supply
3. Consider max supply limit

### DAO Governance

```lumos
#[solana]
#[account]
struct Proposal {
    creator: PublicKey,
    yes_votes: u64,             // ⚠️  Use checked_add
    no_votes: u64,              // ⚠️  Use checked_add
    executed: bool,
}
```

**Fixes Required:**
1. Use checked arithmetic for vote counting
2. Validate creator on execution
3. Ensure executed flag prevents re-execution

## Limitations

### What Static Analysis CAN'T Detect

1. **Program Logic Bugs**
   - Business logic errors
   - Incorrect state transitions
   - Race conditions

2. **External Dependencies**
   - Vulnerabilities in imported crates
   - Cross-program invocation (CPI) issues

3. **Economic Attacks**
   - MEV (Maximal Extractable Value)
   - Front-running
   - Price manipulation

4. **Runtime Issues**
   - Compute budget exhaustion
   - Account data deserialization failures

### Complementary Security Measures

Static analysis is one layer of defense. Also use:

1. **Manual Code Review** - Expert security audits
2. **Fuzzing** - See `lumos fuzz` (coming soon)
3. **Penetration Testing** - Attack simulations
4. **Formal Verification** - Mathematical proofs
5. **Bug Bounties** - Community security research

## Customization

### Suppressing False Positives

If you have a false positive, document why the warning doesn't apply:

```rust
// SECURITY: Field 'counter' is read-only and never modified arithmetically.
// Static analyzer warning suppressed via code review.
pub counter: u64,
```

### Requesting New Rules

Open an issue on GitHub to request new vulnerability detection rules:

- Describe the vulnerability type
- Provide example vulnerable code
- Explain the exploit scenario
- Suggest detection heuristics

## Examples

### Clean Schema (No Issues)

```bash
$ lumos security analyze examples/safe.lumos

Security Analysis Report
Schema: examples/safe.lumos

✓ No security issues found!

All checks passed. Your schema follows Solana security best practices.
```

### Vulnerable Schema

```bash
$ lumos security analyze examples/vulnerable.lumos

Security Analysis Report
Schema: examples/vulnerable.lumos

Summary:
  🚨 2 critical issues
  ⚠️  5 warnings

CRITICAL ISSUES
════════════════════════════════════════════════════════════

🚨 [CRITICAL] Missing Signer Check
   Location: TransferTokens::authority
   Field 'authority' appears to be an authority but lacks explicit signer validation
   💡 Ensure this field requires signer validation in your Anchor program

🚨 [CRITICAL] Missing Signer Check
   Location: MintNFT::minter
   Field 'minter' appears to be an authority but lacks explicit signer validation
   💡 Ensure this field requires signer validation in your Anchor program

Recommendations:
  🚨 Fix all critical issues before deployment
  ⚠️  Review and address warnings
  📚 See: docs/security/static-analysis.md
```

## See Also

- [Account Size Guide](./account-size.md) - Prevent size overflow
- [Security Audit Checklist](./audit-checklist.md) (coming soon)
- [Solana Security Best Practices](https://github.com/coral-xyz/sealevel-attacks)
- [Anchor Security](https://www.anchor-lang.com/docs/security-intro)
