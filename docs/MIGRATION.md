# LUMOS Migration Guide

This guide helps you migrate between major versions of LUMOS, handling breaking changes and deprecated features.

---

## Semantic Versioning Policy

LUMOS follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** (x.0.0) - Breaking changes that require code updates
- **MINOR** (0.x.0) - New features, backwards compatible
- **PATCH** (0.0.x) - Bug fixes, backwards compatible

### Breaking Change Policy

- **Deprecation period**: Features are deprecated for at least one minor version before removal
- **Migration warnings**: Deprecated features trigger warnings during generation
- **Documentation**: All breaking changes documented here with migration steps

---

## Version Compatibility Matrix

| LUMOS Core | LUMOS CLI | Rust Target | TypeScript Target | Anchor | Borsh |
|------------|-----------|-------------|-------------------|--------|-------|
| v0.1.x     | v0.1.x    | 1.70+       | 5.0+              | 0.29+ | 1.0+ |
| v0.2.x     | v0.2.x    | 1.75+       | 5.0+              | 0.30+ | 1.5+ |
| v1.0.x     | v1.0.x    | 1.80+       | 5.5+              | 0.30+ | 2.0+ |

**Note**: Always use matching major.minor versions for `lumos-core` and `lumos-cli`.

---

## Migration Paths

### v0.1.x → v0.2.x (Hypothetical Example)

> **Status**: Not yet released - This section shows the template for future migrations

#### Breaking Changes

##### 1. Enum Discriminants Now Start at 1

**Impact:** 🔴 High - Serialization compatibility broken
**Reason:** Align with Anchor's enum representation for consistency

**Before (v0.1.x):**
```rust
#[solana]
enum GameState {
    Active,   // discriminant: 0
    Paused,   // discriminant: 1
    Finished  // discriminant: 2
}
```

**After (v0.2.x):**
```rust
#[solana]
enum GameState {
    Active,   // discriminant: 1
    Paused,   // discriminant: 2
    Finished  // discriminant: 3
}
```

**Migration Steps:**
1. **Backup**: Save copies of all on-chain account data
2. **Regenerate**: Run `lumos generate` with v0.2.x
3. **Review**: Check all generated enum discriminants
4. **Deploy**: Re-deploy Solana programs (incompatible with old data)
5. **Migrate Data**: Clear existing accounts or migrate data manually
6. **Update Clients**: Deploy updated TypeScript SDKs

**Automated Migration:**
```bash
# This migration cannot be automated due to on-chain data incompatibility
# Manual data migration required
```

---

##### 2. PublicKey Type Renamed to Pubkey

**Impact:** 🟡 Medium - Code changes required
**Reason:** Consistency with Solana's `Pubkey` terminology

**Before (v0.1.x):**
```rust
struct Account {
    owner: PublicKey,
}
```

**After (v0.2.x):**
```rust
struct Account {
    owner: Pubkey,  // Renamed
}
```

**Migration Steps:**
1. Find and replace `PublicKey` → `Pubkey` in all `.lumos` files
2. Regenerate code
3. No on-chain incompatibility (same underlying type)

**Automated Migration:**
```bash
# Find all occurrences
find . -name "*.lumos" -exec grep -l "PublicKey" {} \;

# Replace (review before applying)
find . -name "*.lumos" -exec sed -i '' 's/PublicKey/Pubkey/g' {} \;
```

---

#### New Features

##### String Interpolation in Attributes

```rust
#[doc("Generated for {name}")]  // New in v0.2.x
struct Account {
    name: String,
}
```

**Migration:** Optional - No action required for existing schemas

---

#### Deprecations

##### `#[deprecated]` Attribute

**Status:** Deprecated in v0.2.x, removed in v0.3.x
**Replacement:** Use Rust's native `#[deprecated]` in generated code

**Before:**
```rust
#[deprecated]
struct OldAccount {}
```

**After:**
Add deprecation manually in generated Rust code, or use documentation comments

---

#### Performance Improvements

- 🚀 30% faster code generation for large schemas (100+ types)
- 🚀 50% reduction in WASM bundle size with wasm-opt enabled
- 🚀 Improved string pre-allocation reduces memory allocations

**Migration:** Automatic - Rebuild your project to benefit

---

## Migration Testing Checklist

Before deploying migrated code to production:

### Pre-Migration
- [ ] Read full migration guide for your version jump
- [ ] Backup all on-chain account data
- [ ] Note all breaking changes affecting your schemas
- [ ] Review deprecation warnings in current version
- [ ] Tag current deployment in version control

### During Migration
- [ ] Update `Cargo.toml` versions for `lumos-core` and `lumos-cli`
- [ ] Run `lumos generate` on all schema files
- [ ] Review all generated code diffs
- [ ] Update integration tests
- [ ] Test serialization compatibility (if applicable)

### Post-Migration
- [ ] Run full test suite
- [ ] Verify generated Rust compiles without warnings
- [ ] Verify generated TypeScript type-checks correctly
- [ ] Test Borsh serialization round-trips
- [ ] Deploy to devnet/testnet first
- [ ] Monitor for runtime errors
- [ ] Update client SDKs if needed

---

## Common Migration Scenarios

### Scenario 1: Updating Deployed Solana Program

**Problem:** Breaking change requires program re-deployment

**Solution:**
1. Deploy new program with different program ID
2. Migrate user data to new accounts
3. Update frontend to use new program ID
4. Deprecate old program gracefully

### Scenario 2: Multiple Schemas in Workspace

**Problem:** Need to migrate all schemas consistently

**Solution:**
```bash
# Regenerate all schemas in workspace
find . -name "*.lumos" -exec lumos generate {} \;

# Or use workspace-level script
./scripts/regenerate-all.sh
```

### Scenario 3: Gradual Migration

**Problem:** Cannot migrate entire codebase at once

**Solution:**
- Keep both v0.1.x and v0.2.x schemas temporarily
- Use different file extensions (`.lumos` vs `.lumos2`)
- Migrate module-by-module
- Remove old schemas after full migration

---

## Rollback Procedures

If migration fails, roll back safely:

### Rollback Steps
1. **Restore schemas** from version control
2. **Downgrade LUMOS** to previous version
3. **Regenerate code** with old version
4. **Restore deployments** from backup
5. **Investigate failure** before retrying

### Rollback Commands
```bash
# Restore specific version
cargo install lumos-cli --version 0.1.0

# Regenerate with specific version
lumos generate schema.lumos

# Verify version
lumos --version
```

---

## Getting Help

If you encounter migration issues:

1. **Check this guide** for your specific version transition
2. **Search issues**: https://github.com/getlumos/lumos/issues
3. **Ask community**: Discord/GitHub Discussions
4. **Report bugs**: Create detailed issue with:
   - Source version
   - Target version
   - Minimal reproduction schema
   - Error messages

---

## Maintainer Guidelines

For LUMOS maintainers planning breaking changes:

### Before Breaking Change
1. Discuss in GitHub issue
2. Document deprecation warnings
3. Update this migration guide
4. Add migration tests
5. Update CHANGELOG.md

### Deprecation Process
1. Add deprecation warning in v0.x.0
2. Keep feature working with warning
3. Remove in v0.(x+2).0 (skip one minor version)
4. Document alternative approach

### Communication
- Announce breaking changes in release notes
- Update migration guide before release
- Post migration examples in discussions
- Update documentation site

---

**Last Updated:** 2025-11-21
**Current Version:** v0.1.x
**Next Major Version:** v1.0.0 (planned)
