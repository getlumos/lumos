# DeFi Projects with LUMOS

**Purpose:** Complete guide for building DeFi applications with LUMOS on Solana

**Last Updated:** 2025-12-16

---

## Overview

This guide walks through building production-ready DeFi applications using LUMOS, covering:

- Staking pools with reward distribution
- Token vesting with cliff and linear release
- Vault patterns for secure token custody
- Precision handling for financial calculations
- APY and compound interest formulas

**What we'll build:**

```
┌─────────────────────────────────────────────────────────────┐
│                    DeFi Applications                        │
├─────────────────────────────────────────────────────────────┤
│  STAKING POOL                    TOKEN VESTING              │
│  • Deposit/withdraw tokens       • Cliff periods            │
│  • Reward rate configuration     • Linear vesting           │
│  • APY calculations              • Revocable schedules      │
│  • Lock periods                  • Multi-beneficiary        │
│  • Claim rewards                 • Release tracking         │
└─────────────────────────────────────────────────────────────┘
```

---

## Table of Contents

1. [Why LUMOS for DeFi](#why-lumos-for-defi)
2. [DeFi Architecture](#defi-architecture)
3. [Staking Pool System](#staking-pool-system)
4. [Staker Accounts & Rewards](#staker-accounts--rewards)
5. [Reward Calculations & APY](#reward-calculations--apy)
6. [Token Vesting](#token-vesting)
7. [Vault Patterns](#vault-patterns)
8. [Precision & Safety](#precision--safety)
9. [Complete Staking Implementation](#complete-staking-implementation)
10. [Complete Vesting Implementation](#complete-vesting-implementation)
11. [Frontend Integration](#frontend-integration)
12. [Resources](#resources)

---

## Prerequisites

Before starting, complete these guides:

- [LUMOS + Solana CLI Integration](../solana-cli-integration.md) - Backend deployment
- [LUMOS + web3.js Integration](../web3js-integration.md) - Frontend patterns

**Required tools:**
```bash
cargo install lumos-cli
cargo install --git https://github.com/coral-xyz/anchor avm
solana --version  # 1.18+
```

**SPL Token knowledge recommended:**
```bash
# Install SPL Token CLI for testing
cargo install spl-token-cli
```

---

## Why LUMOS for DeFi

### The DeFi Challenge

DeFi applications require:
- **Precision** - Financial calculations must be exact
- **Type safety** - No room for type mismatches in money code
- **Time handling** - Timestamps critical for rewards/vesting
- **Multi-token** - Often stake one token, earn another

### The LUMOS Solution

| Benefit | Impact on DeFi |
|---------|----------------|
| Type-safe timestamps | `i64` for Unix timestamps, consistent across Rust/TS |
| Precision warnings | JSDoc comments for u64 limits in TypeScript |
| Option types | Clean handling of optional end times, max stakes |
| Generated schemas | Borsh serialization for all financial data |

### Solana DeFi Landscape

| Protocol | TVL | Use Case |
|----------|-----|----------|
| Marinade | $1B+ | Liquid staking |
| Jito | $500M+ | MEV staking |
| Raydium | $200M+ | AMM/DEX |
| Orca | $100M+ | Concentrated liquidity |

LUMOS helps you build protocols with the same type safety as these leaders.

---

## DeFi Architecture

### Account Model

```
┌─────────────────────────────────────────────────────────────┐
│                     STAKING SYSTEM                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐         ┌─────────────┐                   │
│  │ StakingPool │         │   Token     │                   │
│  │    (PDA)    │────────▶│   Vault     │                   │
│  │             │ manages │   (ATA)     │                   │
│  │ • authority │         └─────────────┘                   │
│  │ • rate/sec  │                                           │
│  │ • total     │         ┌─────────────┐                   │
│  └──────┬──────┘         │   Reward    │                   │
│         │                │   Vault     │                   │
│         │ tracks         │   (ATA)     │                   │
│         ▼                └─────────────┘                   │
│  ┌─────────────┐                                           │
│  │StakerAccount│                                           │
│  │    (PDA)    │                                           │
│  │             │                                           │
│  │ • owner     │                                           │
│  │ • staked    │                                           │
│  │ • rewards   │                                           │
│  └─────────────┘                                           │
│                                                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                     VESTING SYSTEM                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐        ┌─────────────┐                   │
│  │VestingProgram│        │   Token     │                   │
│  │    (PDA)     │───────▶│   Vault     │                   │
│  │              │manages │   (ATA)     │                   │
│  │ • authority  │        └─────────────┘                   │
│  │ • total      │                                          │
│  └──────┬───────┘                                          │
│         │                                                   │
│         │ creates                                           │
│         ▼                                                   │
│  ┌──────────────┐                                          │
│  │VestingSchedule│                                          │
│  │    (PDA)     │                                          │
│  │              │                                          │
│  │ • beneficiary│                                          │
│  │ • cliff      │                                          │
│  │ • released   │                                          │
│  └──────────────┘                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### PDA Derivation Strategy

```rust
// Staking Pool - one per authority + token
seeds = [b"pool", authority.as_ref(), token_mint.as_ref()]

// Staker Account - one per user per pool
seeds = [b"staker", pool.as_ref(), owner.as_ref()]

// Token Vault - PDA-controlled ATA
seeds = [b"vault", pool.as_ref()]

// Vesting Schedule - one per beneficiary
seeds = [b"vesting", beneficiary.as_ref(), token_mint.as_ref()]
```

### Two-Token Model

Many DeFi protocols use separate tokens for staking and rewards:

```
Stake Token (e.g., SOL)  →  Staking Pool  →  Reward Token (e.g., PROTOCOL)
         ↓                                              ↓
    Token Vault                                   Reward Vault
```

---

## Staking Pool System

### Complete Schema

```rust
// schemas/defi-staking.lumos

#[solana]
#[account]
struct StakingPool {
    // === Authority ===
    #[key]
    authority: PublicKey,           // Pool admin

    // === Token Configuration ===
    token_mint: PublicKey,          // Token being staked
    vault: PublicKey,               // Vault holding staked tokens

    // === Pool Statistics ===
    total_staked: u64,              // Sum of all stakes
    total_stakers: u32,             // Active staker count

    // === Reward Configuration ===
    reward_rate_per_second: u64,    // Tokens distributed per second
    reward_token_mint: PublicKey,   // Reward token (can differ from stake)
    reward_vault: PublicKey,        // Vault holding reward tokens

    // === Timing ===
    start_time: i64,                // Pool activation time
    end_time: Option<i64>,          // Optional pool expiration
    last_update_time: i64,          // Last reward calculation

    // === Constraints ===
    min_stake_amount: u64,          // Minimum stake (prevent dust)
    max_stake_amount: Option<u64>,  // Optional per-user cap

    // === Status ===
    is_active: bool,                // Pool accepting stakes?
}
```

### Generate Code

```bash
lumos generate schemas/defi-staking.lumos --output programs/staking/src/state/
```

### Pool Configuration Constants

```rust
// programs/staking/src/constants.rs

// Time constants
pub const SECONDS_PER_DAY: i64 = 86_400;
pub const SECONDS_PER_YEAR: i64 = 31_536_000;

// Precision for calculations (9 decimals)
pub const PRECISION: u128 = 1_000_000_000;

// Constraints
pub const MIN_STAKE_DURATION: i64 = 60;          // 1 minute minimum
pub const MAX_REWARD_RATE: u64 = 1_000_000_000;  // 1 token/second max
pub const MIN_STAKE_AMOUNT: u64 = 1_000;         // Prevent dust attacks
```

### Initialize Pool

```rust
pub fn initialize_pool(
    ctx: Context<InitializePool>,
    reward_rate_per_second: u64,
    min_stake_amount: u64,
    max_stake_amount: Option<u64>,
    end_time: Option<i64>,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    // Validate reward rate
    require!(
        reward_rate_per_second <= MAX_REWARD_RATE,
        StakingError::RewardRateTooHigh
    );

    // Validate end time if provided
    if let Some(end) = end_time {
        require!(
            end > clock.unix_timestamp,
            StakingError::InvalidEndTime
        );
    }

    // Initialize pool
    pool.authority = ctx.accounts.authority.key();
    pool.token_mint = ctx.accounts.token_mint.key();
    pool.vault = ctx.accounts.vault.key();
    pool.reward_token_mint = ctx.accounts.reward_token_mint.key();
    pool.reward_vault = ctx.accounts.reward_vault.key();

    pool.total_staked = 0;
    pool.total_stakers = 0;

    pool.reward_rate_per_second = reward_rate_per_second;
    pool.start_time = clock.unix_timestamp;
    pool.end_time = end_time;
    pool.last_update_time = clock.unix_timestamp;

    pool.min_stake_amount = min_stake_amount;
    pool.max_stake_amount = max_stake_amount;
    pool.is_active = true;

    msg!("Pool initialized with {} tokens/second reward rate", reward_rate_per_second);
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + StakingPool::INIT_SPACE,
        seeds = [b"pool", authority.key().as_ref(), token_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, StakingPool>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = pool
    )]
    pub vault: Account<'info, TokenAccount>,

    pub reward_token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = reward_token_mint,
        associated_token::authority = pool
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
```

---

## Staker Accounts & Rewards

### Staker Schema

```rust
#[solana]
#[account]
struct StakerAccount {
    // === Identity ===
    #[key]
    owner: PublicKey,               // Staker's wallet
    pool: PublicKey,                // Associated pool

    // === Staking Data ===
    amount_staked: u64,             // Current stake amount
    stake_timestamp: i64,           // Initial stake time

    // === Rewards Tracking ===
    accumulated_rewards: u64,       // Pending rewards
    last_claim_time: i64,           // Last reward claim
    total_claimed: u64,             // Lifetime claimed rewards

    // === Lock Period ===
    unlock_timestamp: Option<i64>,  // When unstaking allowed
}
```

### Stake Tokens

```rust
pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let staker = &mut ctx.accounts.staker;
    let clock = Clock::get()?;

    // Validate pool is active
    require!(pool.is_active, StakingError::PoolInactive);

    // Check pool hasn't ended
    if let Some(end_time) = pool.end_time {
        require!(
            clock.unix_timestamp < end_time,
            StakingError::PoolEnded
        );
    }

    // Validate amount
    require!(amount >= pool.min_stake_amount, StakingError::BelowMinimum);

    if let Some(max) = pool.max_stake_amount {
        require!(
            staker.amount_staked.checked_add(amount).unwrap() <= max,
            StakingError::ExceedsMaximum
        );
    }

    // Update pending rewards before changing stake
    update_rewards(staker, pool, clock.unix_timestamp)?;

    // Transfer tokens to vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.staker_token_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, amount)?;

    // Update staker account
    if staker.amount_staked == 0 {
        // New staker
        staker.stake_timestamp = clock.unix_timestamp;
        pool.total_stakers = pool.total_stakers.checked_add(1).unwrap();
    }

    staker.amount_staked = staker.amount_staked.checked_add(amount).unwrap();
    pool.total_staked = pool.total_staked.checked_add(amount).unwrap();

    // Emit event
    emit!(StakeEvent {
        staker: staker.owner,
        pool: pool.key(),
        amount,
        timestamp: clock.unix_timestamp,
        event_type: 0,  // stake
    });

    msg!("Staked {} tokens", amount);
    Ok(())
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref(), pool.token_mint.as_ref()],
        bump
    )]
    pub pool: Account<'info, StakingPool>,

    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + StakerAccount::INIT_SPACE,
        seeds = [b"staker", pool.key().as_ref(), owner.key().as_ref()],
        bump
    )]
    pub staker: Account<'info, StakerAccount>,

    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = pool.token_mint,
        associated_token::authority = owner
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
```

### Unstake Tokens

```rust
pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let staker = &mut ctx.accounts.staker;
    let clock = Clock::get()?;

    // Check lock period
    if let Some(unlock_time) = staker.unlock_timestamp {
        require!(
            clock.unix_timestamp >= unlock_time,
            StakingError::StillLocked
        );
    }

    // Validate amount
    require!(amount <= staker.amount_staked, StakingError::InsufficientStake);

    // Update rewards before unstaking
    update_rewards(staker, pool, clock.unix_timestamp)?;

    // Transfer tokens from vault to staker
    let pool_key = pool.key();
    let seeds = &[
        b"pool",
        pool.authority.as_ref(),
        pool.token_mint.as_ref(),
        &[ctx.bumps.pool],
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.staker_token_account.to_account_info(),
            authority: pool.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, amount)?;

    // Update accounts
    staker.amount_staked = staker.amount_staked.checked_sub(amount).unwrap();
    pool.total_staked = pool.total_staked.checked_sub(amount).unwrap();

    if staker.amount_staked == 0 {
        pool.total_stakers = pool.total_stakers.checked_sub(1).unwrap();
    }

    emit!(StakeEvent {
        staker: staker.owner,
        pool: pool.key(),
        amount,
        timestamp: clock.unix_timestamp,
        event_type: 1,  // unstake
    });

    msg!("Unstaked {} tokens", amount);
    Ok(())
}
```

### Claim Rewards

```rust
pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let staker = &mut ctx.accounts.staker;
    let clock = Clock::get()?;

    // Calculate pending rewards
    update_rewards(staker, pool, clock.unix_timestamp)?;

    let rewards_to_claim = staker.accumulated_rewards;
    require!(rewards_to_claim > 0, StakingError::NoRewardsToClaim);

    // Transfer rewards
    let seeds = &[
        b"pool",
        pool.authority.as_ref(),
        pool.token_mint.as_ref(),
        &[ctx.bumps.pool],
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.reward_vault.to_account_info(),
            to: ctx.accounts.staker_reward_account.to_account_info(),
            authority: pool.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, rewards_to_claim)?;

    // Update staker
    staker.accumulated_rewards = 0;
    staker.last_claim_time = clock.unix_timestamp;
    staker.total_claimed = staker.total_claimed.checked_add(rewards_to_claim).unwrap();

    emit!(StakeEvent {
        staker: staker.owner,
        pool: pool.key(),
        amount: rewards_to_claim,
        timestamp: clock.unix_timestamp,
        event_type: 2,  // claim
    });

    msg!("Claimed {} reward tokens", rewards_to_claim);
    Ok(())
}
```

---

## Reward Calculations & APY

### Core Reward Formula

```rust
// Reward calculation helper
fn update_rewards(
    staker: &mut StakerAccount,
    pool: &StakingPool,
    current_time: i64,
) -> Result<()> {
    if staker.amount_staked == 0 || pool.total_staked == 0 {
        staker.last_claim_time = current_time;
        return Ok(());
    }

    // Time since last update
    let time_elapsed = current_time
        .checked_sub(staker.last_claim_time)
        .unwrap()
        .max(0) as u128;

    // Staker's share of pool (with precision)
    let share = (staker.amount_staked as u128)
        .checked_mul(PRECISION)
        .unwrap()
        .checked_div(pool.total_staked as u128)
        .unwrap();

    // Calculate rewards
    // rewards = rate_per_second * time_elapsed * (user_stake / total_staked)
    let rewards = (pool.reward_rate_per_second as u128)
        .checked_mul(time_elapsed)
        .unwrap()
        .checked_mul(share)
        .unwrap()
        .checked_div(PRECISION)
        .unwrap();

    staker.accumulated_rewards = staker
        .accumulated_rewards
        .checked_add(rewards as u64)
        .unwrap();

    staker.last_claim_time = current_time;

    Ok(())
}
```

### APY Calculation

```rust
// Calculate Annual Percentage Yield
pub fn calculate_apy(pool: &StakingPool) -> f64 {
    if pool.total_staked == 0 {
        return 0.0;
    }

    // Annual rewards = rate_per_second * seconds_per_year
    let annual_rewards = (pool.reward_rate_per_second as f64) * (SECONDS_PER_YEAR as f64);

    // APY = (annual_rewards / total_staked) * 100
    (annual_rewards / pool.total_staked as f64) * 100.0
}

// Calculate user's daily rewards
pub fn calculate_daily_rewards(
    staker: &StakerAccount,
    pool: &StakingPool,
) -> u64 {
    if pool.total_staked == 0 {
        return 0;
    }

    let share = (staker.amount_staked as u128)
        .checked_mul(PRECISION)
        .unwrap()
        .checked_div(pool.total_staked as u128)
        .unwrap();

    let daily = (pool.reward_rate_per_second as u128)
        .checked_mul(SECONDS_PER_DAY as u128)
        .unwrap()
        .checked_mul(share)
        .unwrap()
        .checked_div(PRECISION)
        .unwrap();

    daily as u64
}
```

### APY Display Examples

```typescript
// TypeScript APY calculations for frontend

function calculateAPY(pool: StakingPool): number {
  if (pool.total_staked === 0) return 0;

  const SECONDS_PER_YEAR = 31_536_000;
  const annualRewards = pool.reward_rate_per_second * SECONDS_PER_YEAR;

  return (annualRewards / pool.total_staked) * 100;
}

function formatAPY(apy: number): string {
  if (apy > 1000) {
    return `${(apy / 1000).toFixed(1)}k%`;
  }
  return `${apy.toFixed(2)}%`;
}

// Example output:
// 5.25% - Normal APY
// 125.50% - High APY
// 1.2k% - Very high APY (early pool)
```

### Compound Interest (Auto-Compound)

```rust
// For auto-compounding pools
pub fn calculate_compound_rewards(
    principal: u64,
    rate_per_second: u64,
    duration_seconds: i64,
    compound_frequency: i64,  // How often to compound (e.g., daily = 86400)
) -> u64 {
    // A = P(1 + r/n)^(nt)
    // Where:
    // P = principal
    // r = annual rate
    // n = compounds per year
    // t = time in years

    let compounds = duration_seconds / compound_frequency;
    let rate_per_compound = (rate_per_second as u128)
        .checked_mul(compound_frequency as u128)
        .unwrap();

    let mut amount = principal as u128;

    for _ in 0..compounds {
        let interest = amount
            .checked_mul(rate_per_compound)
            .unwrap()
            .checked_div(principal as u128)
            .unwrap();
        amount = amount.checked_add(interest).unwrap();
    }

    amount as u64
}
```

---

## Token Vesting

### Vesting Schema

```rust
// schemas/token-vesting.lumos

#[solana]
#[account]
struct VestingSchedule {
    // === Identity ===
    #[key]
    beneficiary: PublicKey,         // Who receives tokens
    token_mint: PublicKey,          // Token being vested
    vault: PublicKey,               // Vault holding tokens

    // === Amounts ===
    total_amount: u64,              // Total tokens to vest
    released_amount: u64,           // Already released

    // === Timeline ===
    start_timestamp: i64,           // Vesting start
    cliff_timestamp: i64,           // No release before this
    end_timestamp: i64,             // Fully vested after this

    // === Revocation ===
    is_revocable: bool,             // Can be cancelled?
    is_revoked: bool,               // Has been cancelled?
    revoked_at: Option<i64>,        // When revoked
}

#[solana]
#[account]
struct VestingProgram {
    // === Authority ===
    #[key]
    authority: PublicKey,           // Program admin
    token_mint: PublicKey,          // Token for all schedules

    // === Statistics ===
    total_schedules: u32,           // Number of schedules
    total_vested: u64,              // Total tokens vesting
    total_released: u64,            // Total tokens released

    // === Configuration ===
    min_vesting_period: i64,        // Minimum duration
    max_beneficiaries: u32,         // Cap on schedules
}

#[solana]
struct ReleaseEvent {
    beneficiary: PublicKey,
    schedule: PublicKey,
    amount_released: u64,
    timestamp: i64,
    remaining_balance: u64,
}
```

### Vesting Timeline Visualization

```
                    Cliff              End
Start               │                  │
  │                 │                  │
  ├─────────────────┼──────────────────┤
  │   No Release    │  Linear Release  │
  │                 │                  │
  │                 │    ████████████  │ 100% vested
  │                 │    ██████████    │
  │                 │    ████████      │
  │                 │    ██████        │
  │   ░░░░░░░░░░░░  │    ████          │
  │                 │    ██            │
  └─────────────────┴──────────────────┘
       0% vested         Time →
```

### Create Vesting Schedule

```rust
pub fn create_vesting_schedule(
    ctx: Context<CreateVestingSchedule>,
    total_amount: u64,
    start_timestamp: i64,
    cliff_duration: i64,
    vesting_duration: i64,
    is_revocable: bool,
) -> Result<()> {
    let program = &mut ctx.accounts.vesting_program;
    let schedule = &mut ctx.accounts.schedule;
    let clock = Clock::get()?;

    // Validate durations
    require!(
        vesting_duration >= program.min_vesting_period,
        VestingError::VestingPeriodTooShort
    );
    require!(cliff_duration < vesting_duration, VestingError::CliffExceedsVesting);

    // Check beneficiary limit
    require!(
        program.total_schedules < program.max_beneficiaries,
        VestingError::MaxBeneficiariesReached
    );

    // Calculate timestamps
    let cliff_timestamp = start_timestamp.checked_add(cliff_duration).unwrap();
    let end_timestamp = start_timestamp.checked_add(vesting_duration).unwrap();

    // Initialize schedule
    schedule.beneficiary = ctx.accounts.beneficiary.key();
    schedule.token_mint = program.token_mint;
    schedule.vault = ctx.accounts.vault.key();
    schedule.total_amount = total_amount;
    schedule.released_amount = 0;
    schedule.start_timestamp = start_timestamp;
    schedule.cliff_timestamp = cliff_timestamp;
    schedule.end_timestamp = end_timestamp;
    schedule.is_revocable = is_revocable;
    schedule.is_revoked = false;
    schedule.revoked_at = None;

    // Transfer tokens to vault
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.authority_token_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, total_amount)?;

    // Update program stats
    program.total_schedules = program.total_schedules.checked_add(1).unwrap();
    program.total_vested = program.total_vested.checked_add(total_amount).unwrap();

    msg!(
        "Created vesting schedule: {} tokens over {} seconds",
        total_amount,
        vesting_duration
    );

    Ok(())
}
```

### Calculate Releasable Amount

```rust
pub fn calculate_releasable(schedule: &VestingSchedule, current_time: i64) -> u64 {
    // If revoked, nothing more to release
    if schedule.is_revoked {
        return 0;
    }

    // Before cliff - nothing vested
    if current_time < schedule.cliff_timestamp {
        return 0;
    }

    // Calculate total vested amount
    let vested_amount = if current_time >= schedule.end_timestamp {
        // Fully vested
        schedule.total_amount
    } else {
        // Linear vesting between cliff and end
        let total_duration = schedule.end_timestamp - schedule.start_timestamp;
        let elapsed = current_time - schedule.start_timestamp;

        ((schedule.total_amount as u128)
            .checked_mul(elapsed as u128)
            .unwrap()
            .checked_div(total_duration as u128)
            .unwrap()) as u64
    };

    // Releasable = vested - already released
    vested_amount.saturating_sub(schedule.released_amount)
}
```

### Release Vested Tokens

```rust
pub fn release(ctx: Context<Release>) -> Result<()> {
    let schedule = &mut ctx.accounts.schedule;
    let clock = Clock::get()?;

    require!(!schedule.is_revoked, VestingError::ScheduleRevoked);

    let releasable = calculate_releasable(schedule, clock.unix_timestamp);
    require!(releasable > 0, VestingError::NothingToRelease);

    // Transfer tokens to beneficiary
    let seeds = &[
        b"schedule",
        schedule.beneficiary.as_ref(),
        schedule.token_mint.as_ref(),
        &[ctx.bumps.schedule],
    ];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.beneficiary_token_account.to_account_info(),
            authority: schedule.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, releasable)?;

    // Update schedule
    schedule.released_amount = schedule.released_amount.checked_add(releasable).unwrap();

    // Update program stats
    ctx.accounts.vesting_program.total_released = ctx
        .accounts
        .vesting_program
        .total_released
        .checked_add(releasable)
        .unwrap();

    emit!(ReleaseEvent {
        beneficiary: schedule.beneficiary,
        schedule: schedule.key(),
        amount_released: releasable,
        timestamp: clock.unix_timestamp,
        remaining_balance: schedule.total_amount - schedule.released_amount,
    });

    msg!("Released {} tokens", releasable);
    Ok(())
}
```

### Revoke Vesting

```rust
pub fn revoke(ctx: Context<Revoke>) -> Result<()> {
    let schedule = &mut ctx.accounts.schedule;
    let clock = Clock::get()?;

    require!(schedule.is_revocable, VestingError::NotRevocable);
    require!(!schedule.is_revoked, VestingError::AlreadyRevoked);

    // Calculate and release any vested tokens first
    let releasable = calculate_releasable(schedule, clock.unix_timestamp);
    if releasable > 0 {
        // Transfer vested amount to beneficiary
        // ... transfer logic ...
        schedule.released_amount = schedule.released_amount.checked_add(releasable).unwrap();
    }

    // Return unvested tokens to authority
    let unvested = schedule.total_amount - schedule.released_amount;
    if unvested > 0 {
        // Transfer unvested back to authority
        // ... transfer logic ...
    }

    // Mark as revoked
    schedule.is_revoked = true;
    schedule.revoked_at = Some(clock.unix_timestamp);

    msg!("Vesting schedule revoked. {} tokens returned.", unvested);
    Ok(())
}
```

---

## Vault Patterns

### Single Token Vault

```rust
// Simple vault for staking single token
#[derive(Accounts)]
pub struct SingleVault<'info> {
    #[account(
        init,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = pool  // PDA controls vault
    )]
    pub vault: Account<'info, TokenAccount>,

    pub pool: Account<'info, StakingPool>,
    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
```

### Dual Vault (Stake + Rewards)

```rust
// Separate vaults for staked tokens and rewards
#[derive(Accounts)]
pub struct DualVault<'info> {
    // Vault for staked tokens
    #[account(
        init,
        payer = authority,
        associated_token::mint = stake_token_mint,
        associated_token::authority = pool
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    // Vault for reward tokens
    #[account(
        init,
        payer = authority,
        associated_token::mint = reward_token_mint,
        associated_token::authority = pool
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub pool: Account<'info, StakingPool>,
    pub stake_token_mint: Account<'info, Mint>,
    pub reward_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
```

### Escrow Pattern

```rust
// Escrow for trustless operations
#[account]
pub struct Escrow {
    pub depositor: Pubkey,
    pub recipient: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub release_time: i64,
    pub is_released: bool,
}

pub fn create_escrow(
    ctx: Context<CreateEscrow>,
    amount: u64,
    release_time: i64,
) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;

    escrow.depositor = ctx.accounts.depositor.key();
    escrow.recipient = ctx.accounts.recipient.key();
    escrow.mint = ctx.accounts.mint.key();
    escrow.amount = amount;
    escrow.release_time = release_time;
    escrow.is_released = false;

    // Transfer to escrow vault
    // ...

    Ok(())
}

pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;
    let clock = Clock::get()?;

    require!(clock.unix_timestamp >= escrow.release_time, EscrowError::NotYetReleasable);
    require!(!escrow.is_released, EscrowError::AlreadyReleased);

    // Transfer to recipient
    // ...

    escrow.is_released = true;
    Ok(())
}
```

---

## Precision & Safety

### Overflow Protection

```rust
// Always use checked arithmetic for financial calculations
let result = a.checked_add(b).ok_or(StakingError::Overflow)?;
let result = a.checked_sub(b).ok_or(StakingError::Underflow)?;
let result = a.checked_mul(b).ok_or(StakingError::Overflow)?;
let result = a.checked_div(b).ok_or(StakingError::DivisionByZero)?;

// For u128 intermediate calculations
let intermediate = (amount as u128)
    .checked_mul(rate as u128)
    .ok_or(StakingError::Overflow)?;

let final_result = intermediate
    .checked_div(PRECISION)
    .ok_or(StakingError::DivisionByZero)? as u64;
```

### Decimal Handling

```rust
// Token amounts include decimals
// e.g., 1 USDC = 1_000_000 (6 decimals)
// e.g., 1 SOL = 1_000_000_000 (9 decimals)

pub fn normalize_amount(amount: u64, decimals: u8) -> u64 {
    // Convert to base units
    let multiplier = 10u64.pow(decimals as u32);
    amount.checked_mul(multiplier).unwrap()
}

pub fn denormalize_amount(amount: u64, decimals: u8) -> f64 {
    // Convert to human-readable
    let divisor = 10u64.pow(decimals as u32) as f64;
    amount as f64 / divisor
}

// Example:
// User wants to stake 100 USDC
// normalize_amount(100, 6) = 100_000_000
```

### Rounding Strategies

```rust
// Floor (default) - round down, safer for payouts
pub fn floor_div(a: u128, b: u128) -> u128 {
    a / b
}

// Ceiling - round up, use for fee calculations
pub fn ceil_div(a: u128, b: u128) -> u128 {
    (a + b - 1) / b
}

// Round to nearest
pub fn round_div(a: u128, b: u128) -> u128 {
    (a + b / 2) / b
}

// Example: Calculate fee with ceiling (ensures platform gets full fee)
let fee = ceil_div(
    (amount as u128).checked_mul(fee_rate as u128).unwrap(),
    10000,
) as u64;
```

### Dust Prevention

```rust
// Minimum amounts to prevent dust attacks
pub const MIN_STAKE: u64 = 1_000;         // 0.001 tokens (3 decimals)
pub const MIN_REWARD_CLAIM: u64 = 1_000;   // Don't claim tiny amounts

// Check in stake instruction
require!(amount >= MIN_STAKE, StakingError::AmountTooSmall);

// Check in claim instruction
require!(
    staker.accumulated_rewards >= MIN_REWARD_CLAIM,
    StakingError::RewardsTooSmall
);
```

### Timestamp Safety

```rust
// Always use Clock sysvar for timestamps
let clock = Clock::get()?;
let current_time = clock.unix_timestamp;

// Never trust user-provided timestamps for critical logic
// BAD: if user_provided_timestamp > some_deadline
// GOOD: if clock.unix_timestamp > some_deadline

// Handle timestamp comparisons safely
let time_elapsed = current_time
    .checked_sub(last_update_time)
    .unwrap_or(0)
    .max(0);  // Never negative
```

---

## Complete Staking Implementation

### Full Program Structure

```rust
// programs/staking/src/lib.rs

use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;
pub mod utils;

use instructions::*;

declare_id!("StakeXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

#[program]
pub mod staking {
    use super::*;

    // === Pool Management ===

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        reward_rate_per_second: u64,
        min_stake_amount: u64,
        max_stake_amount: Option<u64>,
        end_time: Option<i64>,
    ) -> Result<()> {
        instructions::pool::initialize(
            ctx, reward_rate_per_second, min_stake_amount, max_stake_amount, end_time
        )
    }

    pub fn update_reward_rate(
        ctx: Context<UpdatePool>,
        new_rate: u64,
    ) -> Result<()> {
        instructions::pool::update_reward_rate(ctx, new_rate)
    }

    pub fn pause_pool(ctx: Context<UpdatePool>) -> Result<()> {
        instructions::pool::pause(ctx)
    }

    pub fn resume_pool(ctx: Context<UpdatePool>) -> Result<()> {
        instructions::pool::resume(ctx)
    }

    pub fn fund_rewards(
        ctx: Context<FundRewards>,
        amount: u64,
    ) -> Result<()> {
        instructions::pool::fund_rewards(ctx, amount)
    }

    // === Staking Operations ===

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::staker::stake(ctx, amount)
    }

    pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
        instructions::staker::unstake(ctx, amount)
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::staker::claim_rewards(ctx)
    }

    pub fn compound_rewards(ctx: Context<CompoundRewards>) -> Result<()> {
        instructions::staker::compound_rewards(ctx)
    }

    // === View Functions ===

    pub fn get_pending_rewards(ctx: Context<GetPendingRewards>) -> Result<u64> {
        instructions::view::get_pending_rewards(ctx)
    }

    pub fn get_apy(ctx: Context<GetApy>) -> Result<u64> {
        instructions::view::get_apy(ctx)
    }
}
```

### Error Definitions

```rust
// programs/staking/src/errors.rs

use anchor_lang::prelude::*;

#[error_code]
pub enum StakingError {
    #[msg("Pool is not active")]
    PoolInactive,

    #[msg("Pool has ended")]
    PoolEnded,

    #[msg("Reward rate exceeds maximum")]
    RewardRateTooHigh,

    #[msg("Invalid end time")]
    InvalidEndTime,

    #[msg("Stake amount below minimum")]
    BelowMinimum,

    #[msg("Stake amount exceeds maximum")]
    ExceedsMaximum,

    #[msg("Insufficient staked balance")]
    InsufficientStake,

    #[msg("Tokens still locked")]
    StillLocked,

    #[msg("No rewards to claim")]
    NoRewardsToClaim,

    #[msg("Reward amount too small")]
    RewardsTooSmall,

    #[msg("Arithmetic overflow")]
    Overflow,

    #[msg("Arithmetic underflow")]
    Underflow,

    #[msg("Division by zero")]
    DivisionByZero,

    #[msg("Amount too small")]
    AmountTooSmall,

    #[msg("Insufficient reward balance in vault")]
    InsufficientRewardBalance,

    #[msg("Unauthorized")]
    Unauthorized,
}
```

---

## Complete Vesting Implementation

### Full Program Structure

```rust
// programs/vesting/src/lib.rs

use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;

use instructions::*;

declare_id!("VestXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

#[program]
pub mod vesting {
    use super::*;

    // === Program Management ===

    pub fn initialize_program(
        ctx: Context<InitializeProgram>,
        min_vesting_period: i64,
        max_beneficiaries: u32,
    ) -> Result<()> {
        instructions::program::initialize(ctx, min_vesting_period, max_beneficiaries)
    }

    // === Schedule Management ===

    pub fn create_schedule(
        ctx: Context<CreateSchedule>,
        total_amount: u64,
        start_timestamp: i64,
        cliff_duration: i64,
        vesting_duration: i64,
        is_revocable: bool,
    ) -> Result<()> {
        instructions::schedule::create(
            ctx, total_amount, start_timestamp, cliff_duration, vesting_duration, is_revocable
        )
    }

    pub fn release(ctx: Context<Release>) -> Result<()> {
        instructions::schedule::release(ctx)
    }

    pub fn revoke(ctx: Context<Revoke>) -> Result<()> {
        instructions::schedule::revoke(ctx)
    }

    // === View Functions ===

    pub fn get_releasable(ctx: Context<GetReleasable>) -> Result<u64> {
        instructions::view::get_releasable(ctx)
    }

    pub fn get_vesting_status(ctx: Context<GetVestingStatus>) -> Result<VestingStatus> {
        instructions::view::get_vesting_status(ctx)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct VestingStatus {
    pub total_amount: u64,
    pub released_amount: u64,
    pub releasable_amount: u64,
    pub remaining_amount: u64,
    pub percent_vested: u8,
    pub is_fully_vested: bool,
}
```

---

## Frontend Integration

### React Hooks

```typescript
// src/hooks/useStakingPool.ts
import { useEffect, useState, useCallback } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { StakingPool, StakingPoolSchema } from '../generated';
import { PROGRAM_ID } from '../constants';

export function useStakingPool(authority: PublicKey, tokenMint: PublicKey) {
  const { connection } = useConnection();
  const [pool, setPool] = useState<StakingPool | null>(null);
  const [loading, setLoading] = useState(true);

  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('pool'), authority.toBuffer(), tokenMint.toBuffer()],
    PROGRAM_ID
  );

  const fetchPool = useCallback(async () => {
    try {
      const accountInfo = await connection.getAccountInfo(poolPda);
      if (accountInfo) {
        const data = accountInfo.data.slice(8);
        setPool(StakingPoolSchema.decode(Buffer.from(data)));
      }
    } catch (err) {
      console.error('Failed to fetch pool:', err);
    } finally {
      setLoading(false);
    }
  }, [connection, poolPda]);

  useEffect(() => {
    fetchPool();

    // Subscribe to changes
    const subscriptionId = connection.onAccountChange(poolPda, (accountInfo) => {
      const data = accountInfo.data.slice(8);
      setPool(StakingPoolSchema.decode(Buffer.from(data)));
    });

    return () => {
      connection.removeAccountChangeListener(subscriptionId);
    };
  }, [connection, poolPda, fetchPool]);

  return { pool, poolPda, loading, refetch: fetchPool };
}
```

### Staker Account Hook

```typescript
// src/hooks/useStakerAccount.ts
import { useEffect, useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { StakerAccount, StakerAccountSchema } from '../generated';
import { PROGRAM_ID } from '../constants';

export function useStakerAccount(poolPda: PublicKey | null) {
  const { connection } = useConnection();
  const { publicKey } = useWallet();
  const [staker, setStaker] = useState<StakerAccount | null>(null);
  const [loading, setLoading] = useState(true);

  const stakerPda = publicKey && poolPda
    ? PublicKey.findProgramAddressSync(
        [Buffer.from('staker'), poolPda.toBuffer(), publicKey.toBuffer()],
        PROGRAM_ID
      )[0]
    : null;

  useEffect(() => {
    async function fetch() {
      if (!stakerPda) {
        setStaker(null);
        setLoading(false);
        return;
      }

      try {
        const accountInfo = await connection.getAccountInfo(stakerPda);
        if (accountInfo) {
          const data = accountInfo.data.slice(8);
          setStaker(StakerAccountSchema.decode(Buffer.from(data)));
        }
      } catch (err) {
        console.error('Failed to fetch staker:', err);
      } finally {
        setLoading(false);
      }
    }

    fetch();
  }, [connection, stakerPda]);

  return { staker, stakerPda, loading };
}
```

### APY Calculator Component

```typescript
// src/components/APYCalculator.tsx
import { StakingPool } from '../generated';

interface APYCalculatorProps {
  pool: StakingPool;
}

const SECONDS_PER_YEAR = 31_536_000;

export function APYCalculator({ pool }: APYCalculatorProps) {
  const calculateAPY = (): number => {
    if (pool.total_staked === 0) return 0;

    const annualRewards = pool.reward_rate_per_second * SECONDS_PER_YEAR;
    return (annualRewards / pool.total_staked) * 100;
  };

  const calculateDailyRewards = (stakeAmount: number): number => {
    if (pool.total_staked === 0) return 0;

    const share = stakeAmount / pool.total_staked;
    return pool.reward_rate_per_second * 86_400 * share;
  };

  const apy = calculateAPY();

  return (
    <div className="apy-calculator">
      <div className="apy-display">
        <span className="label">APY</span>
        <span className="value">
          {apy > 1000 ? `${(apy / 1000).toFixed(1)}k%` : `${apy.toFixed(2)}%`}
        </span>
      </div>

      <div className="stats">
        <div className="stat">
          <span className="label">Total Staked</span>
          <span className="value">
            {(pool.total_staked / 1e9).toLocaleString()} tokens
          </span>
        </div>

        <div className="stat">
          <span className="label">Stakers</span>
          <span className="value">{pool.total_stakers.toLocaleString()}</span>
        </div>

        <div className="stat">
          <span className="label">Reward Rate</span>
          <span className="value">
            {pool.reward_rate_per_second.toLocaleString()} / sec
          </span>
        </div>
      </div>
    </div>
  );
}
```

### Staking Form Component

```typescript
// src/components/StakingForm.tsx
import { useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useWallet } from '@solana/wallet-adapter-react';
import { useProgram } from '../hooks/useProgram';
import { StakingPool } from '../generated';

interface StakingFormProps {
  pool: StakingPool;
  poolPda: PublicKey;
  onSuccess?: () => void;
}

export function StakingForm({ pool, poolPda, onSuccess }: StakingFormProps) {
  const { publicKey } = useWallet();
  const program = useProgram();
  const [amount, setAmount] = useState('');
  const [loading, setLoading] = useState(false);

  async function handleStake() {
    if (!program || !publicKey) return;

    setLoading(true);
    try {
      const amountBN = new BN(parseFloat(amount) * 1e9);

      await program.methods
        .stake(amountBN)
        .accounts({
          pool: poolPda,
          // ... other accounts
        })
        .rpc();

      setAmount('');
      onSuccess?.();
    } catch (err) {
      console.error('Stake failed:', err);
    } finally {
      setLoading(false);
    }
  }

  async function handleUnstake() {
    // Similar to stake
  }

  const minStake = pool.min_stake_amount / 1e9;
  const maxStake = pool.max_stake_amount ? pool.max_stake_amount / 1e9 : undefined;

  return (
    <div className="staking-form">
      <h3>Stake Tokens</h3>

      <div className="input-group">
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder={`Min: ${minStake}`}
          min={minStake}
          max={maxStake}
          step="0.01"
        />
        <span className="suffix">tokens</span>
      </div>

      <div className="constraints">
        <span>Min: {minStake}</span>
        {maxStake && <span>Max: {maxStake}</span>}
      </div>

      <div className="actions">
        <button
          onClick={handleStake}
          disabled={loading || !amount || parseFloat(amount) < minStake}
        >
          {loading ? 'Staking...' : 'Stake'}
        </button>

        <button onClick={handleUnstake} disabled={loading}>
          Unstake
        </button>
      </div>
    </div>
  );
}
```

### Vesting Timeline Component

```typescript
// src/components/VestingTimeline.tsx
import { VestingSchedule } from '../generated';

interface VestingTimelineProps {
  schedule: VestingSchedule;
}

export function VestingTimeline({ schedule }: VestingTimelineProps) {
  const now = Date.now() / 1000;

  const totalDuration = schedule.end_timestamp - schedule.start_timestamp;
  const elapsed = Math.max(0, now - schedule.start_timestamp);
  const progress = Math.min(100, (elapsed / totalDuration) * 100);

  const cliffProgress = ((schedule.cliff_timestamp - schedule.start_timestamp) / totalDuration) * 100;

  const isBeforeCliff = now < schedule.cliff_timestamp;
  const isFullyVested = now >= schedule.end_timestamp;

  const vestedAmount = isBeforeCliff
    ? 0
    : isFullyVested
    ? schedule.total_amount
    : Math.floor((schedule.total_amount * elapsed) / totalDuration);

  const releasable = vestedAmount - schedule.released_amount;

  return (
    <div className="vesting-timeline">
      <div className="header">
        <h3>Vesting Schedule</h3>
        {schedule.is_revoked && <span className="badge revoked">Revoked</span>}
      </div>

      <div className="progress-bar">
        <div className="cliff-marker" style={{ left: `${cliffProgress}%` }}>
          <span>Cliff</span>
        </div>
        <div className="progress" style={{ width: `${progress}%` }} />
      </div>

      <div className="stats">
        <div className="stat">
          <span className="label">Total</span>
          <span className="value">
            {(schedule.total_amount / 1e9).toLocaleString()} tokens
          </span>
        </div>

        <div className="stat">
          <span className="label">Vested</span>
          <span className="value">
            {(vestedAmount / 1e9).toLocaleString()} tokens
          </span>
        </div>

        <div className="stat">
          <span className="label">Released</span>
          <span className="value">
            {(schedule.released_amount / 1e9).toLocaleString()} tokens
          </span>
        </div>

        <div className="stat highlight">
          <span className="label">Releasable</span>
          <span className="value">
            {(releasable / 1e9).toLocaleString()} tokens
          </span>
        </div>
      </div>

      <div className="dates">
        <div>Start: {new Date(schedule.start_timestamp * 1000).toLocaleDateString()}</div>
        <div>Cliff: {new Date(schedule.cliff_timestamp * 1000).toLocaleDateString()}</div>
        <div>End: {new Date(schedule.end_timestamp * 1000).toLocaleDateString()}</div>
      </div>

      {releasable > 0 && !schedule.is_revoked && (
        <button className="release-button">
          Release {(releasable / 1e9).toFixed(2)} tokens
        </button>
      )}
    </div>
  );
}
```

---

## Resources

### Example Code

- **DeFi Staking Schema:** [`examples/defi-staking/schema.lumos`](../../../examples/defi-staking/schema.lumos)
- **Token Vesting Schema:** [`examples/token-vesting/schema.lumos`](../../../examples/token-vesting/schema.lumos)
- **awesome-lumos DeFi:** [github.com/getlumos/awesome-lumos](https://github.com/getlumos/awesome-lumos)

### Related Guides

**Integration Guides:**
- [LUMOS + Anchor Integration](/guides/anchor-integration) - Build new DeFi programs
- [LUMOS + Solv/Jito Integration](/guides/solv-jito-integration) - Liquid staking & MEV patterns
- [Solana CLI Integration](/guides/solana-cli-integration) - Deploy your protocols
- [web3.js Integration](/guides/web3js-integration) - Frontend patterns

**Other Use Cases:**
- [Gaming Projects](/guides/use-cases/gaming) - Game economy patterns
- [NFT Marketplaces](/guides/use-cases/nft) - NFT staking rewards

**Migration Guides:**
- [Migration: TypeScript → LUMOS](/guides/migration-typescript) - Migrate existing types
- [Migration: Anchor → LUMOS](/guides/migration-anchor) - Add LUMOS to existing protocols

### External Resources

- [Anchor Documentation](https://www.anchor-lang.com/)
- [SPL Token Program](https://spl.solana.com/token)
- [Marinade Finance (Reference)](https://marinade.finance/)
- [Jito Staking (Reference)](https://www.jito.network/)

### DeFi Security

- [Solana Security Best Practices](https://github.com/coral-xyz/sealevel-attacks)
- [Common DeFi Vulnerabilities](https://consensys.github.io/smart-contract-best-practices/)
- [Anchor Security Considerations](https://www.anchor-lang.com/docs/security)

---

## Next Steps

1. **Generate your schemas:**
   ```bash
   lumos generate schemas/defi-staking.lumos
   lumos generate schemas/token-vesting.lumos
   ```

2. **Build the programs:** `anchor build`

3. **Test locally:** `solana-test-validator && anchor test`

4. **Deploy to devnet:** Follow [Solana CLI guide](../solana-cli-integration.md)

5. **Build your frontend:** Use patterns from [web3.js guide](../web3js-integration.md)

6. **Audit before mainnet:** DeFi protocols handling real funds must be audited

---

**Last Updated:** 2025-12-16
**Version:** 1.0.0
