# LUMOS + Solana CLI Integration Guide

**Purpose:** Complete workflow guide for using LUMOS with Solana CLI tools

**Last Updated:** 2025-12-16

---

## Overview

This guide covers the end-to-end workflow for developing Solana programs using LUMOS:

```
.lumos schema → Generate code → Build program → Test locally → Deploy to network
```

**What you'll learn:**
- Setting up Solana CLI with LUMOS
- Building Anchor programs from LUMOS schemas
- Testing with `solana-test-validator`
- Deploying to devnet, testnet, and mainnet

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Solana CLI Configuration](#solana-cli-configuration)
3. [Build Workflow](#build-workflow)
4. [Testing with solana-test-validator](#testing-with-solana-test-validator)
5. [Deployment Patterns](#deployment-patterns)
6. [Complete E2E Example](#complete-e2e-example)
7. [Troubleshooting](#troubleshooting)
8. [Quick Reference](#quick-reference)

---

## Prerequisites

### Required Tools

| Tool | Version | Installation |
|------|---------|--------------|
| Rust | 1.70+ | [rustup.rs](https://rustup.rs) |
| Solana CLI | 1.18+ | See below |
| Anchor | 0.30+ | `cargo install --git https://github.com/coral-xyz/anchor avm` |
| LUMOS CLI | 0.1.1+ | `cargo install lumos-cli` |
| Node.js | 18+ | [nodejs.org](https://nodejs.org) |

### Install Solana CLI

**macOS/Linux:**
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```

**Verify installation:**
```bash
solana --version
# solana-cli 1.18.x (src:xxxxxxxx; feat:xxxxxxxxx, client:SolanaLabs)
```

**Add to PATH (if needed):**
```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) for persistence.

### Install LUMOS CLI

```bash
cargo install lumos-cli

# Verify
lumos --version
# lumos-cli 0.1.1
```

### Install Anchor

```bash
# Install AVM (Anchor Version Manager)
cargo install --git https://github.com/coral-xyz/anchor avm --force

# Install latest Anchor
avm install latest
avm use latest

# Verify
anchor --version
# anchor-cli 0.30.x
```

### Generate Keypair

If you don't have a Solana keypair yet:

```bash
# Generate new keypair (saves to ~/.config/solana/id.json)
solana-keygen new

# Or generate to custom location
solana-keygen new --outfile ~/my-wallet.json

# View your public key
solana-keygen pubkey
# 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

**Important:** Back up your keypair file securely. Never share your private key.

---

## Solana CLI Configuration

### View Current Configuration

```bash
solana config get
```

**Output:**
```
Config File: /home/user/.config/solana/cli/config.yml
RPC URL: https://api.mainnet-beta.solana.com
WebSocket URL: wss://api.mainnet-beta.solana.com/ (computed)
Keypair Path: /home/user/.config/solana/id.json
Commitment: confirmed
```

### Configure Network (RPC URL)

**Devnet (recommended for development):**
```bash
solana config set --url devnet

# Or explicit URL
solana config set --url https://api.devnet.solana.com
```

**Testnet:**
```bash
solana config set --url testnet
```

**Mainnet-beta:**
```bash
solana config set --url mainnet-beta
```

**Local validator:**
```bash
solana config set --url localhost

# Or explicit
solana config set --url http://127.0.0.1:8899
```

### Configure Keypair

```bash
# Use default keypair
solana config set --keypair ~/.config/solana/id.json

# Use custom keypair
solana config set --keypair ~/my-project-wallet.json
```

### Environment Variables

Set environment variables for scripting:

```bash
# .env or shell profile
export SOLANA_RPC_URL="https://api.devnet.solana.com"
export ANCHOR_WALLET="$HOME/.config/solana/id.json"
export ANCHOR_PROVIDER_URL="https://api.devnet.solana.com"
```

### Check Balance

```bash
# Check balance on current network
solana balance

# Check specific address
solana balance 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

### Get SOL for Testing

**Devnet airdrop:**
```bash
solana airdrop 2
# Requesting airdrop of 2 SOL
# 2 SOL
```

**Testnet airdrop (rate-limited):**
```bash
solana airdrop 1 --url testnet
```

**Note:** Mainnet requires real SOL. Never airdrop on mainnet.

---

## Build Workflow

### Standard LUMOS → Solana Workflow

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  schema.lumos   │ ──▶ │  lumos generate │ ──▶ │  generated.rs   │
│  (your schema)  │     │                 │     │  generated.ts   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Deploy to Solana│ ◀── │  anchor build   │ ◀── │ Anchor program  │
│    network      │     │  (cargo build)  │     │  (lib.rs)       │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Step 1: Create LUMOS Schema

**Create `schema.lumos`:**
```rust
// Game accounts for on-chain gaming
#[solana]
#[account]
struct PlayerAccount {
    #[key]
    wallet: PublicKey,
    #[max(20)]
    username: String,
    level: u16,
    experience: u64,
    gold: u64,
    created_at: i64,
}

#[solana]
#[account]
struct GameItem {
    #[key]
    id: u64,
    owner: PublicKey,
    #[max(50)]
    name: String,
    power: u16,
    rarity: u8,
}
```

### Step 2: Generate Code

**Basic generation:**
```bash
lumos generate schema.lumos
```

**Output:**
```
✓ Parsed schema.lumos (2 structs, 0 enums)
✓ Generated ./generated.rs (Rust)
✓ Generated ./generated.ts (TypeScript)
```

**Generate with options:**
```bash
# Preview first (dry-run)
lumos generate schema.lumos --dry-run

# Generate to specific directory
lumos generate schema.lumos --output ./src/

# Watch mode for development
lumos generate schema.lumos --watch
```

### Step 3: Generate Anchor Program (Optional)

For complete Anchor program scaffolding:

```bash
# Generate complete Anchor program
lumos anchor generate schema.lumos \
  --name my_game \
  --address $(solana-keygen pubkey) \
  --output ./programs/my_game

# Preview without writing
lumos anchor generate schema.lumos --name my_game --dry-run
```

**Generated structure:**
```
programs/my_game/
├── Cargo.toml
├── Xargo.toml
└── src/
    ├── lib.rs           # Anchor program entry
    ├── state.rs         # Account structures
    ├── instructions/    # Instruction handlers
    └── errors.rs        # Custom errors
```

### Step 4: Build Program

**Using Anchor:**
```bash
cd my_game_project
anchor build
```

**Output:**
```
BPF SDK: /home/user/.local/share/solana/install/active_release/bin/sdk/bpf
cargo-build-bpf child: rustup toolchain list -v
...
To deploy this program:
  $ solana program deploy /path/to/target/deploy/my_game.so
```

**Using cargo-build-sbf directly:**
```bash
cargo build-sbf
```

### Step 5: Check Program Size

Solana programs have size limits (currently ~1.4MB for upgradeable programs).

```bash
# Check compiled program size
ls -lh target/deploy/my_game.so

# Detailed size analysis
solana program show --programs
```

**LUMOS check-size command:**
```bash
# Check account sizes against Solana limits
lumos check-size schema.lumos --format table
```

**Output:**
```
┌─────────────────┬──────────────┬───────────────┬────────┐
│ Account         │ Size (bytes) │ Rent (SOL)    │ Status │
├─────────────────┼──────────────┼───────────────┼────────┤
│ PlayerAccount   │ 156          │ 0.00156       │ ✓      │
│ GameItem        │ 98           │ 0.00098       │ ✓      │
└─────────────────┴──────────────┴───────────────┴────────┘
```

---

## Testing with solana-test-validator

### Start Local Validator

**Basic startup:**
```bash
solana-test-validator
```

**With reset (clean state):**
```bash
solana-test-validator --reset
```

**Background mode:**
```bash
solana-test-validator --reset &
```

**Custom configuration:**
```bash
solana-test-validator \
  --reset \
  --quiet \
  --bpf-program <PROGRAM_ID> target/deploy/my_game.so \
  --ledger ./test-ledger
```

### Configure CLI for Local

```bash
solana config set --url localhost
```

### Deploy to Local Validator

**Terminal 1 - Start validator:**
```bash
solana-test-validator --reset
```

**Terminal 2 - Deploy and test:**
```bash
# Configure for localhost
solana config set --url localhost

# Airdrop SOL for deployment
solana airdrop 10

# Deploy program
solana program deploy target/deploy/my_game.so

# Output:
# Program Id: 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

### Monitor Logs

**Watch all logs:**
```bash
solana logs
```

**Filter by program:**
```bash
solana logs <PROGRAM_ID>

# Example
solana logs 7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
```

**Output:**
```
Streaming transaction logs. Confirmed commitment
Transaction executed in slot 12345:
  Signature: 5K9x...
  Status: Ok
  Log Messages:
    Program 7xKXtg... invoke [1]
    Program log: Instruction: InitializePlayer
    Program 7xKXtg... consumed 25000 of 200000 compute units
    Program 7xKXtg... success
```

### Anchor Test Integration

**Run Anchor tests against local validator:**
```bash
# Start validator in background
solana-test-validator --reset &

# Wait for validator to start
sleep 5

# Run tests (skip built-in validator)
anchor test --skip-local-validator
```

**Or use Anchor's built-in validator:**
```bash
# Anchor manages validator lifecycle
anchor test
```

### Complete Local Test Script

**`scripts/test-local.sh`:**
```bash
#!/bin/bash
set -e

echo "Starting local validator..."
solana-test-validator --reset --quiet &
VALIDATOR_PID=$!

# Wait for validator
sleep 3

echo "Configuring for localhost..."
solana config set --url localhost

echo "Airdropping SOL..."
solana airdrop 10

echo "Building program..."
anchor build

echo "Deploying program..."
anchor deploy

echo "Running tests..."
anchor test --skip-local-validator

echo "Cleaning up..."
kill $VALIDATOR_PID

echo "Tests complete!"
```

**Run:**
```bash
chmod +x scripts/test-local.sh
./scripts/test-local.sh
```

---

## Deployment Patterns

### Network Comparison

| Network | Purpose | SOL | Persistence | RPC URL |
|---------|---------|-----|-------------|---------|
| Localhost | Development | Free (airdrop) | Reset on restart | `http://127.0.0.1:8899` |
| Devnet | Testing | Free (airdrop) | Persistent | `https://api.devnet.solana.com` |
| Testnet | Staging | Free (limited) | Persistent | `https://api.testnet.solana.com` |
| Mainnet | Production | Real SOL | Persistent | `https://api.mainnet-beta.solana.com` |

### Deploy to Devnet

**Step 1: Configure network**
```bash
solana config set --url devnet
```

**Step 2: Get SOL**
```bash
solana airdrop 2
# Wait a few seconds between airdrops if needed
solana airdrop 2
```

**Step 3: Build**
```bash
anchor build
```

**Step 4: Deploy**
```bash
anchor deploy

# Or using solana-cli directly
solana program deploy target/deploy/my_game.so
```

**Step 5: Verify**
```bash
# Check program exists
solana program show <PROGRAM_ID>

# Check your balance after deployment
solana balance
```

### Deploy to Testnet

Same process as devnet, but airdrops are rate-limited:

```bash
solana config set --url testnet
solana airdrop 1  # Smaller amounts, rate-limited
anchor deploy
```

### Deploy to Mainnet

**Production deployment checklist:**

- [ ] All tests passing on devnet
- [ ] Security audit completed
- [ ] Upgrade authority configured
- [ ] Sufficient SOL for deployment (~3-5 SOL recommended)
- [ ] Backup of keypairs

**Deploy:**
```bash
# Configure mainnet
solana config set --url mainnet-beta

# Check balance (need real SOL)
solana balance

# Deploy
anchor deploy

# Or with explicit keypair
solana program deploy target/deploy/my_game.so \
  --keypair ~/.config/solana/mainnet-deployer.json
```

### Program Upgrades

**Check current authority:**
```bash
solana program show <PROGRAM_ID>
```

**Upgrade program:**
```bash
# Build new version
anchor build

# Upgrade (must have upgrade authority)
anchor upgrade target/deploy/my_game.so --program-id <PROGRAM_ID>

# Or using solana-cli
solana program deploy target/deploy/my_game.so --program-id <PROGRAM_ID>
```

**Set upgrade authority:**
```bash
# Transfer authority to new keypair
solana program set-upgrade-authority <PROGRAM_ID> \
  --new-upgrade-authority <NEW_AUTHORITY_PUBKEY>

# Make immutable (irreversible!)
solana program set-upgrade-authority <PROGRAM_ID> --final
```

### Deployment Cost Estimation

```bash
# Check rent for program account
solana rent $(stat -f%z target/deploy/my_game.so 2>/dev/null || stat -c%s target/deploy/my_game.so)

# Example output:
# Rent per byte-year: 0.00000348 SOL
# Rent per epoch: 0.0000019 SOL
# Rent-exempt minimum: 1.14 SOL
```

---

## Complete E2E Example

This example uses the Gaming schema from `examples/gaming/schema.lumos`.

### Project Setup

**Step 1: Create project directory**
```bash
mkdir lumos-gaming-demo
cd lumos-gaming-demo
```

**Step 2: Initialize Anchor project**
```bash
anchor init gaming_program --javascript
cd gaming_program
```

**Step 3: Create LUMOS schema**
```bash
mkdir -p schemas
cat > schemas/gaming.lumos << 'EOF'
// On-Chain Gaming Schema
// Demonstrates: Player accounts, items, leaderboards

#[solana]
#[account]
struct PlayerAccount {
    #[key]
    wallet: PublicKey,
    #[max(20)]
    username: String,
    level: u16,
    experience: u64,
    health: u16,
    gold: u64,
    created_at: i64,
    last_login: i64,
}

#[solana]
#[account]
struct GameItem {
    #[key]
    id: u64,
    owner: PublicKey,
    #[max(50)]
    name: String,
    item_type: u8,
    rarity: u8,
    power: u16,
    is_equipped: bool,
    acquired_at: i64,
}

#[solana]
#[account]
struct Leaderboard {
    #[key]
    season: u32,
    top_players: [PublicKey],
    top_scores: [u64],
    season_start: i64,
    season_end: i64,
    is_active: bool,
}
EOF
```

### Generate and Integrate

**Step 4: Generate code**
```bash
# Generate Rust and TypeScript
lumos generate schemas/gaming.lumos --output programs/gaming_program/src/

# Verify generation
ls -la programs/gaming_program/src/
# generated.rs  lib.rs
```

**Step 5: Update lib.rs to use generated types**

Edit `programs/gaming_program/src/lib.rs`:
```rust
use anchor_lang::prelude::*;

mod generated;
pub use generated::*;

declare_id!("YOUR_PROGRAM_ID_HERE");

#[program]
pub mod gaming_program {
    use super::*;

    pub fn initialize_player(
        ctx: Context<InitializePlayer>,
        username: String,
    ) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.wallet = ctx.accounts.authority.key();
        player.username = username;
        player.level = 1;
        player.experience = 0;
        player.health = 100;
        player.gold = 0;
        player.created_at = Clock::get()?.unix_timestamp;
        player.last_login = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn gain_experience(
        ctx: Context<GainExperience>,
        amount: u64,
    ) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.experience += amount;

        // Level up every 1000 XP
        let new_level = (player.experience / 1000) as u16 + 1;
        if new_level > player.level {
            player.level = new_level;
            msg!("Level up! New level: {}", player.level);
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePlayer<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PlayerAccount::LEN,
        seeds = [b"player", authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, PlayerAccount>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GainExperience<'info> {
    #[account(
        mut,
        seeds = [b"player", authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, PlayerAccount>,
    pub authority: Signer<'info>,
}
```

### Build and Test Locally

**Step 6: Build**
```bash
anchor build
```

**Step 7: Start local validator**
```bash
# Terminal 1
solana-test-validator --reset
```

**Step 8: Configure and deploy**
```bash
# Terminal 2
solana config set --url localhost
solana airdrop 10
anchor deploy
```

**Step 9: Update program ID**

Get the deployed program ID:
```bash
solana address -k target/deploy/gaming_program-keypair.json
```

Update in `lib.rs`:
```rust
declare_id!("YOUR_ACTUAL_PROGRAM_ID");
```

Update in `Anchor.toml`:
```toml
[programs.localnet]
gaming_program = "YOUR_ACTUAL_PROGRAM_ID"
```

Rebuild and redeploy:
```bash
anchor build
anchor deploy
```

### Deploy to Devnet

**Step 10: Configure for devnet**
```bash
solana config set --url devnet
```

**Step 11: Get devnet SOL**
```bash
solana airdrop 2
solana airdrop 2  # May need multiple airdrops
```

**Step 12: Deploy**
```bash
anchor deploy
```

**Step 13: Verify deployment**
```bash
# Check program
solana program show $(solana address -k target/deploy/gaming_program-keypair.json)

# View on explorer
echo "https://explorer.solana.com/address/$(solana address -k target/deploy/gaming_program-keypair.json)?cluster=devnet"
```

### Test with TypeScript Client

**Step 14: Create test script**

`tests/gaming.ts`:
```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GamingProgram } from "../target/types/gaming_program";
import { expect } from "chai";

describe("gaming_program", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GamingProgram as Program<GamingProgram>;

  it("Initializes a player", async () => {
    const [playerPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initializePlayer("HeroPlayer")
      .accounts({
        player: playerPda,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const player = await program.account.playerAccount.fetch(playerPda);
    expect(player.username).to.equal("HeroPlayer");
    expect(player.level).to.equal(1);
    expect(player.experience.toNumber()).to.equal(0);
  });

  it("Gains experience and levels up", async () => {
    const [playerPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("player"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    // Gain 1500 XP (should reach level 2)
    await program.methods
      .gainExperience(new anchor.BN(1500))
      .accounts({
        player: playerPda,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const player = await program.account.playerAccount.fetch(playerPda);
    expect(player.experience.toNumber()).to.equal(1500);
    expect(player.level).to.equal(2);
  });
});
```

**Step 15: Run tests**
```bash
# Against local validator
anchor test

# Against devnet
anchor test --provider.cluster devnet
```

---

## Troubleshooting

### Common Errors and Solutions

#### "Account not found"

**Cause:** Account doesn't exist or wrong network

**Solution:**
```bash
# Check you're on correct network
solana config get

# Check account exists
solana account <ADDRESS>

# Airdrop if needed
solana airdrop 2
```

#### "Insufficient funds for rent"

**Cause:** Not enough SOL to create account

**Solution:**
```bash
# Check balance
solana balance

# Airdrop more
solana airdrop 2

# Check rent requirement
solana rent <SIZE_IN_BYTES>
```

#### "Program failed to complete"

**Cause:** Program error (panic, assertion, compute limit)

**Solution:**
```bash
# Check logs for details
solana logs <PROGRAM_ID>

# Common issues:
# - Compute budget exceeded → Optimize code or request more CU
# - Account size mismatch → Check space calculation
# - Invalid seeds → Verify PDA derivation
```

#### Build fails with "BPF SDK not found"

**Cause:** Solana tools not installed or not in PATH

**Solution:**
```bash
# Reinstall Solana
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# Add to PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Verify
solana --version
cargo build-sbf --version
```

#### "Transaction simulation failed"

**Cause:** Various - check simulation logs

**Solution:**
```bash
# Get detailed error
solana confirm -v <SIGNATURE>

# Check logs
solana logs
```

#### Anchor build fails with version mismatch

**Cause:** Anchor/Solana version incompatibility

**Solution:**
```bash
# Check versions
anchor --version
solana --version

# Update Anchor
avm install latest
avm use latest

# Clean and rebuild
anchor clean
anchor build
```

### Network-Specific Issues

#### Devnet airdrop fails

```bash
# Rate limited - wait and retry
sleep 30
solana airdrop 1

# Try smaller amount
solana airdrop 0.5

# Use faucet website as backup
# https://faucet.solana.com
```

#### Testnet RPC errors

```bash
# Try alternate RPC
solana config set --url https://testnet.solana.com

# Or use custom RPC provider
solana config set --url https://your-rpc-provider.com
```

### LUMOS-Specific Issues

#### "Unknown type" in schema

**Cause:** Type not defined or misspelled

**Solution:**
```bash
# Validate schema
lumos validate schema.lumos

# Check type definitions
# Ensure all referenced types are defined
```

#### Generated code doesn't compile

**Cause:** Schema issue or version mismatch

**Solution:**
```bash
# Regenerate with backup
lumos generate schema.lumos --backup

# Check for breaking changes
lumos diff old-schema.lumos schema.lumos

# Validate compatibility
lumos check-compat old-schema.lumos schema.lumos
```

---

## Quick Reference

### Essential Commands

```bash
# LUMOS
lumos generate schema.lumos           # Generate code
lumos validate schema.lumos           # Validate schema
lumos check schema.lumos              # Check if up-to-date
lumos anchor generate schema.lumos    # Generate Anchor program

# Solana CLI
solana config set --url devnet        # Set network
solana airdrop 2                      # Get test SOL
solana balance                        # Check balance
solana program deploy <FILE>          # Deploy program
solana logs <PROGRAM_ID>              # Watch logs

# Anchor
anchor build                          # Build program
anchor test                           # Run tests
anchor deploy                         # Deploy program
anchor upgrade <FILE> --program-id X  # Upgrade program

# Local Testing
solana-test-validator --reset         # Start local validator
solana config set --url localhost     # Use local validator
```

### Network URLs

```bash
# Localhost
http://127.0.0.1:8899

# Devnet
https://api.devnet.solana.com

# Testnet
https://api.testnet.solana.com

# Mainnet
https://api.mainnet-beta.solana.com
```

### Workflow Cheatsheet

```bash
# Development cycle
lumos generate schema.lumos && anchor build && anchor test

# Deploy to devnet
solana config set --url devnet && solana airdrop 2 && anchor deploy

# Check deployment
solana program show <PROGRAM_ID>
```

---

## Related Guides

After mastering the Solana CLI workflow, explore these related guides:

**Integration Guides:**
- [LUMOS + Anchor Integration](/guides/anchor-integration) - Schema-first Anchor program development
- [LUMOS + web3.js Integration](/guides/web3js-integration) - Frontend TypeScript integration

**Migration Guides:**
- [Migration: TypeScript → LUMOS](/guides/migration-typescript) - Migrate existing TypeScript types
- [Migration: Anchor → LUMOS](/guides/migration-anchor) - Add LUMOS to existing Anchor projects

**Use Case Guides:**
- [Gaming Projects](/guides/use-cases/gaming) - Complete gaming implementation
- [NFT Marketplaces](/guides/use-cases/nft) - Metaplex-compatible NFTs
- [DeFi Protocols](/guides/use-cases/defi) - Staking and vesting patterns

---

**Last Updated:** 2025-12-16
**Version:** 1.0.0
