# Schema Versioning Example

This example demonstrates how to use the `#[version]` attribute in LUMOS to track schema versions.

## Overview

Schema versioning helps you:
- **Track Changes**: Document when and how schemas evolve
- **Plan Migrations**: Identify breaking vs. compatible changes
- **Maintain Compatibility**: Use version constants in runtime checks
- **Coordinate Teams**: Clear communication about schema expectations

## Semantic Versioning

LUMOS uses [Semantic Versioning](https://semver.org/) (SemVer):

```
MAJOR.MINOR.PATCH[-prerelease][+build]

Examples:
- 1.0.0         (initial release)
- 1.1.0         (new feature, backwards compatible)
- 2.0.0         (breaking change)
- 3.0.0-beta.1  (pre-release)
- 1.0.0+build.123 (build metadata)
```

### Version Bumping Guidelines

- **MAJOR**: Breaking changes
  - Removing fields
  - Changing field types
  - Renaming fields

- **MINOR**: Backwards-compatible additions
  - Adding optional fields
  - Adding new enum variants
  - New helper functions

- **PATCH**: Non-breaking fixes
  - Documentation updates
  - Bug fixes that don't change schema

## Usage

### Basic Versioning

```rust
#[solana]
#[version = "1.0.0"]
#[account]
struct UserAccount {
    wallet: PublicKey,
    balance: u64,
}
```

This generates version constants in both Rust and TypeScript:

**Rust:**
```rust
pub const USERACCOUNT_VERSION: &str = "1.0.0";

#[account]
pub struct UserAccount {
    pub wallet: Pubkey,
    pub balance: u64,
}
```

**TypeScript:**
```typescript
export const USERACCOUNT_VERSION = "1.0.0";

export interface UserAccount {
  wallet: PublicKey;
  balance: number;
}
```

### Runtime Version Checks

Use the generated constants to verify compatibility:

**Rust:**
```rust
// Check schema version before processing
if account_version != USERACCOUNT_VERSION {
    return Err(ProgramError::InvalidAccountData);
}
```

**TypeScript:**
```typescript
// Verify client compatibility
if (serverVersion !== USERACCOUNT_VERSION) {
    throw new Error(`Version mismatch: expected ${USERACCOUNT_VERSION}, got ${serverVersion}`);
}
```

### Pre-release Versions

Use pre-release versions for beta features:

```rust
#[solana]
#[version = "2.0.0-beta.1"]
struct BetaFeature {
    feature_id: u64,
    enabled: bool,
}
```

### Build Metadata

Track build information:

```rust
#[solana]
#[version = "1.0.0+build.20250123"]
struct DeploymentTracked {
    data: String,
}
```

## Generating Code

```bash
# Generate Rust and TypeScript from versioned schema
lumos generate examples/versioning/schema.lumos

# Output includes version constants
cat generated.rs   # Contains: pub const USERACCOUNT_VERSION
cat generated.ts   # Contains: export const USERACCOUNT_VERSION
```

## Best Practices

1. **Start with 1.0.0**: Begin at 1.0.0 for production schemas
2. **Update on Changes**: Bump version with every schema change
3. **Document Migrations**: Track version history in CHANGELOG
4. **Test Compatibility**: Ensure clients support version ranges
5. **Deprecate Gracefully**: Use pre-release versions for experimental features

## Migration Strategy

When updating schemas:

1. **Create V2**: Define new version alongside V1
2. **Dual Support**: Support both versions temporarily
3. **Migrate Data**: Convert V1 accounts to V2
4. **Deprecate V1**: Remove old version after migration period

See `schema.lumos` for examples of V1 → V2 evolution.

## Further Reading

- [Semantic Versioning Spec](https://semver.org/)
- [LUMOS Syntax Reference](/docs/syntax-reference.md)
- [Migration Guide](/docs/MIGRATION.md)
