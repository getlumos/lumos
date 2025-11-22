# Security Audit Checklist Generator

> Auto-generate comprehensive security audit checklists from LUMOS schemas for manual code review.

## Overview

The `lumos audit generate` command automatically creates a detailed security audit checklist tailored to your schema. This helps ensure thorough security reviews by providing a structured list of checks organized by category and priority.

## Usage

### Basic Generation

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

### Custom Output Path

```bash
lumos audit generate schema.lumos --output my-audit.md
```

### JSON Format

For integration with audit management tools:

```bash
lumos audit generate schema.lumos --format json --output audit.json
```

## Generated Checklist Structure

### Markdown Format

```markdown
# Security Audit Checklist

**Generated from:** `schema.lumos`
**Date:** 2025-11-22
**Total Checks:** 39

---

## How to Use This Checklist

- [ ] = Not checked yet
- [x] = Verified and compliant
- Priority: 🔴 CRITICAL | 🟡 HIGH | 🟢 MEDIUM | ⚪ LOW

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

### JSON Format

```json
[
  {
    "category": "Account Validation",
    "priority": "CRITICAL",
    "item": "Verify account ownership (program owns the account)",
    "context": "PlayerAccount",
    "explanation": "Ensure the account is owned by the program...",
    "checked": false
  }
]
```

## Check Categories

### 🔐 Account Validation (CRITICAL)

**For all `#[account]` structs:**

| Check | Priority | Description |
|-------|----------|-------------|
| Verify account ownership | 🔴 CRITICAL | Ensure program owns the account |
| Validate account discriminator | 🔴 CRITICAL | Verify 8-byte Anchor discriminator |
| Check account is initialized | 🟡 HIGH | Verify not in uninitialized state |
| Verify rent exemption | 🟢 MEDIUM | Sufficient lamports for rent |

### ✍️ Signer Checks (CRITICAL)

**For authority fields** (`authority`, `admin`, `owner`, `signer`, `payer`, `creator`, `minter`, `updater`):

| Check | Priority | Description |
|-------|----------|-------------|
| Verify field requires signer | 🔴 CRITICAL | Transaction must be signed by corresponding key |
| Ensure access control | 🔴 CRITICAL | Only authorized users can perform operations |

### 🔢 Arithmetic Safety (HIGH)

**For arithmetic fields** (`balance`, `amount`, `supply`, `total`, `count`, `price`, `value`, `reward`, `stake`, `fee`):

| Check | Priority | Description |
|-------|----------|-------------|
| Verify checked arithmetic | 🟡 HIGH | Use checked_add, checked_sub, checked_mul |
| Validate bounds/constraints | 🟢 MEDIUM | Ensure values are within acceptable ranges |

### 🚪 Access Control (CRITICAL)

**For `owner` fields:**

| Check | Priority | Description |
|-------|----------|-------------|
| Validate owner matches signer | 🔴 CRITICAL | Verify signer is owner for mutations |

### ✅ Data Validation (MEDIUM)

**For PublicKey fields:**

| Check | Priority | Description |
|-------|----------|-------------|
| Verify not default/system pubkey | 🟢 MEDIUM | Check not all zeros or system program |

**For Vec/Array fields:**

| Check | Priority | Description |
|-------|----------|-------------|
| Validate length before iteration | 🟡 HIGH | Prevent excessive compute or out-of-bounds |
| Ensure max size within limits | 🟢 MEDIUM | Won't exceed 10MB account limit |

**For Option fields:**

| Check | Priority | Description |
|-------|----------|-------------|
| Handle None case | 🟢 MEDIUM | Proper logic when field is None |

### 🔄 State Transition (HIGH)

**For all accounts:**

| Check | Priority | Description |
|-------|----------|-------------|
| Verify valid/atomic transitions | 🟡 HIGH | Can't leave account inconsistent |
| Check for reentrancy | 🟢 MEDIUM | Safe if making CPIs |

### 🎬 Initialization (HIGH)

**For `#[account]` structs:**

| Check | Priority | Description |
|-------|----------|-------------|
| Check account initialized | 🟡 HIGH | Verify proper initialization before use |

### 💰 Rent Exemption (MEDIUM)

**For `#[account]` structs:**

| Check | Priority | Description |
|-------|----------|-------------|
| Verify sufficient lamports | 🟢 MEDIUM | Won't be garbage collected |

## Example Use Cases

### DeFi Token Program

```bash
$ lumos audit generate examples/defi-staking/schema.lumos

Generated: SECURITY_AUDIT.md

Checklist includes:
  ✓ 45 total checks
  ✓ 8 account validation checks
  ✓ 4 signer checks
  ✓ 12 arithmetic safety checks  # Critical for token operations!
  ✓ 4 access control checks
```

**Key checks generated:**
- Verify `balance` uses checked arithmetic
- Validate `authority` requires signer
- Check `stake_amount` bounds
- Ensure `reward_total` won't overflow

### NFT Marketplace

```bash
$ lumos audit generate examples/nft-marketplace/schema.lumos

Checklist includes:
  ✓ 32 total checks
  ✓ 6 account validation checks
  ✓ 3 signer checks
  ✓ 2 arithmetic safety checks
  ✓ 5 access control checks
```

**Key checks generated:**
- Verify `seller` field requires signer
- Validate `nft_mint` not default pubkey
- Check `price` uses checked math
- Ensure `buyer` authorization

### DAO Governance

```bash
$ lumos audit generate examples/dao-governance/schema.lumos

Checklist includes:
  ✓ 38 total checks
  ✓ 6 account validation checks
  ✓ 2 signer checks
  ✓ 6 arithmetic safety checks
  ✓ 4 access control checks
```

**Key checks generated:**
- Verify vote counts use checked arithmetic
- Validate proposal creator signer
- Check execution authorization
- Ensure state transitions valid

## Integration with Audit Process

### Manual Security Audit Workflow

1. **Generate Checklist**
   ```bash
   lumos audit generate schema.lumos
   ```

2. **Review Generated Checks**
   - Open `SECURITY_AUDIT.md`
   - Review each category
   - Understand context and explanations

3. **Perform Code Review**
   - Check Rust implementation against each item
   - Mark `[x]` when verified
   - Document findings in notes

4. **Track Completion**
   - Fill in auditor name and dates
   - Document any issues found
   - Create tickets for fixes

### Team Collaboration

```bash
# Generate checklist
lumos audit generate schema.lumos --output AUDIT_v1.md

# Commit to repo
git add AUDIT_v1.md
git commit -m "chore: Add security audit checklist"

# Team members check off items as they review
# Track progress in GitHub issues or project board
```

### CI/CD Integration

```yaml
# .github/workflows/security.yml
- name: Generate Audit Checklist
  run: |
    lumos audit generate schema.lumos --format json > audit.json

- name: Check Critical Items
  run: |
    # Custom script to verify critical checks are documented
    node scripts/verify-audit-items.js audit.json
```

## Customization

### Adding Custom Checks

While the generator creates comprehensive checklists, you may want to add project-specific items:

```markdown
## 🔐 Account Validation

<!-- Generated checks -->

<!-- Add custom checks here -->
- [ ] 🔴 **Verify integration with Oracle X**
  - Context: `PriceAccount`
  - Ensure price feed updates are from trusted oracle

- [ ] 🟡 **Validate custom constraint Y**
  - Context: `SpecialAccount`
  - Check business-specific validation logic
```

### Suppressing Non-Applicable Checks

If a check doesn't apply to your program:

```markdown
- [~] 🟢 **Handle None case for optional 'data' field**
  - Context: `Account::data`
  - ~~This field is always Some in our program logic~~
  - **SKIPPED:** Field is never None per invariant X
```

## Best Practices

### 1. Generate Early

Create checklist at the start of development:
```bash
# After initial schema design
lumos audit generate schema.lumos
```

Benefits:
- Security considerations from day one
- Team aware of requirements
- Easier to build securely than retrofit

### 2. Update Regularly

Regenerate when schema changes:
```bash
# After schema modifications
lumos audit generate schema.lumos --output AUDIT_v2.md
git diff AUDIT_v1.md AUDIT_v2.md  # Review new checks
```

### 3. Use in PR Reviews

Include checklist verification in PR process:
```markdown
## Security Checklist

Relevant items from SECURITY_AUDIT.md:

- [x] Verified 'balance' uses checked_add (line 45)
- [x] Confirmed 'authority' requires signer (line 67)
- [x] Validated state transition logic (line 89)
```

### 4. Combine with Other Tools

Use checklist alongside:
- **Static Analysis** - `lumos security analyze` for automated checks
- **Size Analysis** - `lumos check-size` for account limits
- **Testing** - Comprehensive test suite
- **Formal Verification** - Mathematical proofs where critical

### 5. Document Decisions

For each check, document:
```markdown
- [x] 🔴 **Verify 'balance' uses checked arithmetic**
  - Context: `TokenAccount::balance`
  - **VERIFIED:** Using `checked_add` at line 145
  - **VERIFIED:** Using `checked_sub` at line 189
  - **TESTED:** Test case `test_balance_overflow` covers this
```

## Limitations

### What the Generator Does

✅ Creates comprehensive checklist from schema
✅ Identifies security-sensitive fields
✅ Categorizes and prioritizes checks
✅ Provides context and explanations

### What the Generator Does NOT Do

❌ Actually perform the security audit
❌ Analyze program logic implementation
❌ Detect runtime vulnerabilities
❌ Replace professional security audit

## Example Generated Checklist

Here's a partial example of what gets generated:

```markdown
# Security Audit Checklist

**Generated from:** `examples/gaming/schema.lumos`
**Date:** 2025-11-22
**Total Checks:** 39

---

## 🔐 Account Validation

- [ ] 🔴 **Verify account ownership (program owns the account)**
  - Context: `PlayerAccount`
  - Ensure the account is owned by the program to prevent attacks where an attacker passes an account owned by a different program.

- [ ] 🔴 **Validate account discriminator**
  - Context: `PlayerAccount`
  - Anchor's 8-byte discriminator prevents type confusion attacks. Verify it's checked on deserialization.

## ✍️ Signer Checks

- [ ] 🔴 **Verify 'owner' field requires signer**
  - Context: `GameItem::owner`
  - Authority fields must validate that the transaction is signed by the corresponding private key.

## 🔢 Arithmetic Safety

- [ ] 🟡 **Verify 'total_playtime' uses checked arithmetic operations**
  - Context: `PlayerAccount::total_playtime`
  - Use checked_add, checked_sub, checked_mul to prevent integer overflow/underflow vulnerabilities that could lead to loss of funds.

---

## Additional Security Considerations

- [ ] **Program Logic:** Verify business logic correctness
- [ ] **Error Handling:** Ensure all error paths are covered
- [ ] **Testing:** Comprehensive test suite including edge cases

---

**Audit Status:**

- Auditor: _________________
- Date Started: _________________
- Date Completed: _________________
- Findings: _________________
```

## See Also

- [Static Analysis Guide](./static-analysis.md) - Automated vulnerability detection
- [Account Size Guide](./account-size.md) - Prevent size overflow
- [Solana Security Best Practices](https://github.com/coral-xyz/sealevel-attacks)
- [Anchor Security](https://www.anchor-lang.com/docs/security-intro)
