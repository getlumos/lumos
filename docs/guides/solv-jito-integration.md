# LUMOS + Solv/Jito Integration Guide

This guide covers integrating LUMOS with **liquid staking protocols** (like Solv, Marinade, Jito) and **MEV infrastructure** (Jito bundles and tips). These are critical DeFi primitives on Solana that benefit enormously from type-safe schema definitions.

```
┌─────────────────────────────────────────────────────────────────┐
│                    LUMOS + LST/MEV Architecture                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    generate    ┌──────────────────────────┐  │
│  │              │ ─────────────► │  Rust (Anchor)           │  │
│  │   schema     │                │  - LiquidStakingPool     │  │
│  │   .lumos     │                │  - ExchangeRateState     │  │
│  │              │                │  - MEVRewardsPool        │  │
│  │  LST + MEV   │                │  - RedemptionTicket      │  │
│  │  Accounts    │                └──────────────────────────┘  │
│  │              │                                               │
│  │              │    generate    ┌──────────────────────────┐  │
│  │              │ ─────────────► │  TypeScript              │  │
│  └──────────────┘                │  - Borsh serialization   │  │
│                                  │  - Exchange rate helpers │  │
│                                  │  - MEV distribution calc │  │
│                                  └──────────────────────────┘  │
│                                                                 │
│  Key Advantages:                                                │
│  ✓ Type-safe exchange rate calculations (u64 precision)        │
│  ✓ Guaranteed Rust ↔ TypeScript serialization match            │
│  ✓ Automatic account size calculation for complex LST state    │
│  ✓ PDA derivation helpers for nested LST accounts              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Table of Contents

1. [Overview & Why LUMOS for LST/MEV](#overview--why-lumos-for-lstmev)
2. [Prerequisites](#prerequisites)
3. [Liquid Staking Token Architecture](#liquid-staking-token-architecture)
4. [Core Schema Design](#core-schema-design)
5. [MEV Integration Patterns](#mev-integration-patterns)
6. [Instruction Contexts](#instruction-contexts)
7. [Exchange Rate Calculations](#exchange-rate-calculations)
8. [TypeScript Client Integration](#typescript-client-integration)
9. [Complete Example: Mini LST Protocol](#complete-example-mini-lst-protocol)
10. [Best Practices & Security](#best-practices--security)
11. [Related Guides](#related-guides)

---

## Prerequisites

Before starting this guide, you should have:

- **DeFi fundamentals** - Understanding of staking, liquid staking, and yield mechanics
- **Anchor experience** - Familiarity with Anchor's account model and PDAs
- **SPL Token knowledge** - How token mints, accounts, and transfers work
- **LUMOS CLI installed** - `cargo install lumos-cli`

**Recommended reading:**
- [LUMOS + Anchor Integration](/guides/anchor-integration) - Schema-first Anchor development
- [DeFi Projects Use Case](/guides/use-cases/defi) - Staking pool fundamentals
- [LUMOS + web3.js Integration](/guides/web3js-integration) - TypeScript client patterns

---

## Overview & Why LUMOS for LST/MEV

### What is Liquid Staking?

Liquid staking allows users to stake assets (SOL, BTC) while receiving a **liquid derivative token** (LST) that can be traded, used in DeFi, or held for yield. Key protocols:

| Protocol | LST Token | TVL | Focus |
|----------|-----------|-----|-------|
| Jito | JitoSOL | $2B+ | MEV-enhanced staking |
| Marinade | mSOL | $1B+ | Decentralized staking |
| Solv | SolvBTC | $500M+ | Bitcoin liquid staking |
| Lido | stSOL | $300M+ | Multi-chain staking |

### What is MEV on Solana?

**MEV (Maximal Extractable Value)** refers to profit extracted by ordering, including, or excluding transactions. Jito provides infrastructure for:

- **Bundle submission** - Atomic transaction bundles
- **Tip distribution** - Rewards for block builders
- **MEV auctions** - Competitive ordering markets

### Why LUMOS for LST/MEV?

**1. Exchange Rate Precision**

LST protocols live and die by exchange rate accuracy. A single bit error means lost funds:

```lumos
#[solana]
#[account]
struct ExchangeRateState {
    // LUMOS generates JSDoc warnings for u64 precision limits
    rate_numerator: u64,    // Total SOL value (with rewards)
    rate_denominator: u64,  // Total LST supply
    last_update_slot: u64,
    precision: u64,         // Typically 1_000_000_000 (1e9)
}
```

**2. Complex Account Hierarchies**

LST protocols have deeply nested account structures:

```
LiquidStakingPool (root)
├── ExchangeRateState (PDA)
├── SOLVault (token account)
├── LSTMint (mint authority)
├── RedemptionQueue (PDA)
│   └── RedemptionTicket[] (user PDAs)
└── ValidatorStake[] (delegation tracking)
```

LUMOS ensures all these types serialize identically in Rust and TypeScript.

**3. MEV Amounts Are Large**

MEV tips can be substantial (100+ SOL per block). LUMOS automatically warns about u64 precision:

```typescript
// Generated TypeScript includes precision warnings
export interface MEVRewardsPool {
  /**
   * Warning: u64 values above 2^53-1 lose precision in JavaScript.
   * For MEV amounts, consider using BigInt or string representation.
   */
  totalMevCollected: number;
  pendingTips: number;
}
```

**4. Audit-Ready Code**

Generated code follows consistent patterns, making audits easier:

- Predictable account layouts
- Automatic discriminator handling
- Clear type boundaries

---

## Liquid Staking Token Architecture

### Core Components

Every LST protocol needs these fundamental accounts:

```
┌─────────────────────────────────────────────────────────────────┐
│                    LST Protocol Architecture                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  User SOL ──► ┌────────────────┐ ──► LST Token                  │
│               │ LiquidStaking  │                                 │
│               │     Pool       │                                 │
│               └───────┬────────┘                                 │
│                       │                                          │
│         ┌─────────────┼─────────────┐                           │
│         ▼             ▼             ▼                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │ SOL      │  │ Exchange │  │ LST      │                      │
│  │ Vault    │  │ Rate     │  │ Mint     │                      │
│  └──────────┘  └──────────┘  └──────────┘                      │
│                                                                  │
│  User LST ──► ┌────────────────┐ ──► Redemption                 │
│               │ Redemption     │     Ticket                      │
│               │ Queue          │                                 │
│               └────────────────┘                                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Account Model

**1. LiquidStakingPool** - The central pool managing SOL deposits and LST minting

**2. ExchangeRateState** - Tracks the SOL:LST conversion ratio (updates as rewards accrue)

**3. SOLVault** - Native SOL or wrapped SOL storage

**4. LSTMint** - The liquid token mint (pool has mint authority)

**5. RedemptionQueue** - Manages unstaking requests (may have cooldown period)

**6. RedemptionTicket** - Individual user's unstaking request

### PDA Strategy

LST protocols use hierarchical PDAs:

```rust
// Pool PDA
seeds = [b"lst_pool", authority.key()]

// Exchange rate PDA (derived from pool)
seeds = [b"exchange_rate", pool.key()]

// Redemption queue PDA
seeds = [b"redemption_queue", pool.key()]

// User's redemption ticket
seeds = [b"ticket", queue.key(), user.key(), ticket_id.to_le_bytes()]
```

### Exchange Rate Mechanics

The exchange rate determines how much SOL each LST is worth:

```
exchange_rate = total_sol_value / total_lst_supply

Example:
- Pool has 1,000,000 SOL (including staking rewards)
- 900,000 LST tokens exist
- Rate = 1,000,000 / 900,000 = 1.111... SOL per LST
```

As staking rewards accrue, the rate increases (LST appreciates).

---

## Core Schema Design

### Complete LST Schema

```lumos
// schema.lumos - Liquid Staking Token Protocol

#[solana]
#[account]
struct LiquidStakingPool {
    /// Pool authority (can update parameters)
    authority: PublicKey,

    /// The liquid staking token mint
    lst_mint: PublicKey,

    /// SOL vault (native SOL storage)
    sol_vault: PublicKey,

    /// Total SOL deposited (excluding rewards)
    #[key]
    total_sol_deposited: u64,

    /// Total SOL value (deposits + rewards)
    total_sol_value: u64,

    /// Total LST tokens minted
    total_lst_minted: u64,

    /// Fee for instant unstaking (basis points, max 10000)
    instant_unstake_fee_bps: u16,

    /// Fee for delayed unstaking (basis points)
    delayed_unstake_fee_bps: u16,

    /// Minimum stake amount (in lamports)
    min_stake_amount: u64,

    /// Maximum pool capacity (0 = unlimited)
    max_pool_capacity: u64,

    /// Pool state
    is_paused: bool,

    /// Bump seed for PDA
    bump: u8,

    /// Reserved for future upgrades
    _reserved: [u8; 64],
}

#[solana]
#[account]
struct ExchangeRateState {
    /// Parent pool
    pool: PublicKey,

    /// Rate numerator (total SOL value * precision)
    rate_numerator: u64,

    /// Rate denominator (total LST supply)
    rate_denominator: u64,

    /// Precision multiplier (typically 1e9)
    precision: u64,

    /// Last update slot
    last_update_slot: u64,

    /// Last update Unix timestamp
    last_update_timestamp: i64,

    /// Rate change since last epoch (basis points)
    epoch_rate_change_bps: i16,

    /// Bump seed
    bump: u8,
}

#[solana]
#[account]
struct RedemptionQueue {
    /// Parent pool
    pool: PublicKey,

    /// Total LST pending redemption
    total_pending_lst: u64,

    /// Total SOL reserved for redemptions
    total_reserved_sol: u64,

    /// Next ticket ID
    next_ticket_id: u64,

    /// Cooldown period in seconds (e.g., 2-3 days)
    cooldown_seconds: u64,

    /// Queue is accepting new redemptions
    is_active: bool,

    /// Bump seed
    bump: u8,
}

#[solana]
#[account]
struct RedemptionTicket {
    /// Owner of this ticket
    owner: PublicKey,

    /// Parent queue
    queue: PublicKey,

    /// Ticket ID (unique within queue)
    ticket_id: u64,

    /// LST amount being redeemed
    lst_amount: u64,

    /// SOL amount to receive (calculated at creation)
    sol_amount: u64,

    /// Exchange rate at time of redemption request
    exchange_rate_at_request: u64,

    /// Unix timestamp when ticket was created
    created_at: i64,

    /// Unix timestamp when redemption can be claimed
    claimable_at: i64,

    /// Ticket state
    state: RedemptionState,

    /// Bump seed
    bump: u8,
}

#[solana]
enum RedemptionState {
    /// Waiting for cooldown period
    Pending,
    /// Ready to claim
    Claimable,
    /// Already claimed
    Claimed,
    /// Cancelled by user
    Cancelled,
}

#[solana]
#[account]
struct ValidatorStake {
    /// Parent pool
    pool: PublicKey,

    /// Validator vote account
    validator_vote: PublicKey,

    /// Stake account for this validator
    stake_account: PublicKey,

    /// Amount delegated to this validator
    delegated_amount: u64,

    /// Rewards earned from this validator
    rewards_earned: u64,

    /// Last epoch rewards were collected
    last_reward_epoch: u64,

    /// Validator performance score (0-10000)
    performance_score: u16,

    /// Is this validator active for new delegations
    is_active: bool,

    /// Bump seed
    bump: u8,
}

#[solana]
#[account]
struct UserStakeInfo {
    /// User's wallet
    owner: PublicKey,

    /// Pool this info is for
    pool: PublicKey,

    /// Total LST tokens held by user
    lst_balance: u64,

    /// User's share of pool (for accurate reward tracking)
    pool_share_bps: u64,

    /// Total SOL ever deposited
    total_deposited: u64,

    /// Total SOL ever withdrawn
    total_withdrawn: u64,

    /// First deposit timestamp
    first_deposit_at: i64,

    /// Bump seed
    bump: u8,
}
```

### Generated Rust Code

Running `lumos generate schema.lumos` produces:

```rust
// generated.rs
use anchor_lang::prelude::*;

#[account]
pub struct LiquidStakingPool {
    /// Pool authority (can update parameters)
    pub authority: Pubkey,
    /// The liquid staking token mint
    pub lst_mint: Pubkey,
    /// SOL vault (native SOL storage)
    pub sol_vault: Pubkey,
    /// Total SOL deposited (excluding rewards)
    pub total_sol_deposited: u64,
    /// Total SOL value (deposits + rewards)
    pub total_sol_value: u64,
    /// Total LST tokens minted
    pub total_lst_minted: u64,
    /// Fee for instant unstaking (basis points, max 10000)
    pub instant_unstake_fee_bps: u16,
    /// Fee for delayed unstaking (basis points)
    pub delayed_unstake_fee_bps: u16,
    /// Minimum stake amount (in lamports)
    pub min_stake_amount: u64,
    /// Maximum pool capacity (0 = unlimited)
    pub max_pool_capacity: u64,
    /// Pool state
    pub is_paused: bool,
    /// Bump seed for PDA
    pub bump: u8,
    /// Reserved for future upgrades
    pub _reserved: [u8; 64],
}

impl LiquidStakingPool {
    pub const LEN: usize = 8 + // discriminator
        32 +  // authority
        32 +  // lst_mint
        32 +  // sol_vault
        8 +   // total_sol_deposited
        8 +   // total_sol_value
        8 +   // total_lst_minted
        2 +   // instant_unstake_fee_bps
        2 +   // delayed_unstake_fee_bps
        8 +   // min_stake_amount
        8 +   // max_pool_capacity
        1 +   // is_paused
        1 +   // bump
        64;   // _reserved
}

#[account]
pub struct ExchangeRateState {
    pub pool: Pubkey,
    pub rate_numerator: u64,
    pub rate_denominator: u64,
    pub precision: u64,
    pub last_update_slot: u64,
    pub last_update_timestamp: i64,
    pub epoch_rate_change_bps: i16,
    pub bump: u8,
}

impl ExchangeRateState {
    pub const LEN: usize = 8 + 32 + 8 + 8 + 8 + 8 + 8 + 2 + 1;
}

// ... additional structs
```

### Generated TypeScript

```typescript
// generated.ts
import { PublicKey } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';

export interface LiquidStakingPool {
  authority: PublicKey;
  lstMint: PublicKey;
  solVault: PublicKey;
  /** Warning: u64 values above 2^53-1 (9,007,199,254,740,991) lose precision */
  totalSolDeposited: number;
  /** Warning: u64 values above 2^53-1 (9,007,199,254,740,991) lose precision */
  totalSolValue: number;
  /** Warning: u64 values above 2^53-1 (9,007,199,254,740,991) lose precision */
  totalLstMinted: number;
  instantUnstakeFeeBps: number;
  delayedUnstakeFeeBps: number;
  /** Warning: u64 values above 2^53-1 (9,007,199,254,740,991) lose precision */
  minStakeAmount: number;
  /** Warning: u64 values above 2^53-1 (9,007,199,254,740,991) lose precision */
  maxPoolCapacity: number;
  isPaused: boolean;
  bump: number;
  reserved: number[];
}

export const LiquidStakingPoolSchema = borsh.struct([
  borsh.publicKey('authority'),
  borsh.publicKey('lstMint'),
  borsh.publicKey('solVault'),
  borsh.u64('totalSolDeposited'),
  borsh.u64('totalSolValue'),
  borsh.u64('totalLstMinted'),
  borsh.u16('instantUnstakeFeeBps'),
  borsh.u16('delayedUnstakeFeeBps'),
  borsh.u64('minStakeAmount'),
  borsh.u64('maxPoolCapacity'),
  borsh.bool('isPaused'),
  borsh.u8('bump'),
  borsh.array(borsh.u8(), 64, 'reserved'),
]);

export interface ExchangeRateState {
  pool: PublicKey;
  /** Warning: u64 values above 2^53-1 lose precision. Use BigInt for rate calculations. */
  rateNumerator: number;
  /** Warning: u64 values above 2^53-1 lose precision. Use BigInt for rate calculations. */
  rateDenominator: number;
  precision: number;
  lastUpdateSlot: number;
  lastUpdateTimestamp: number;
  epochRateChangeBps: number;
  bump: number;
}

export const ExchangeRateStateSchema = borsh.struct([
  borsh.publicKey('pool'),
  borsh.u64('rateNumerator'),
  borsh.u64('rateDenominator'),
  borsh.u64('precision'),
  borsh.u64('lastUpdateSlot'),
  borsh.i64('lastUpdateTimestamp'),
  borsh.i16('epochRateChangeBps'),
  borsh.u8('bump'),
]);

// Redemption state enum
export type RedemptionState =
  | { kind: 'Pending' }
  | { kind: 'Claimable' }
  | { kind: 'Claimed' }
  | { kind: 'Cancelled' };

export const RedemptionStateSchema = borsh.rustEnum([
  borsh.struct([], 'Pending'),
  borsh.struct([], 'Claimable'),
  borsh.struct([], 'Claimed'),
  borsh.struct([], 'Cancelled'),
]);
```

---

## MEV Integration Patterns

### MEV on Solana Overview

Jito provides MEV infrastructure with these components:

1. **Block Engine** - Accepts transaction bundles
2. **Relayer** - Routes transactions to block builders
3. **Tips** - Payments to block builders for inclusion priority

### MEV Account Schema

```lumos
// mev-schema.lumos - MEV Integration Types

#[solana]
#[account]
struct MEVRewardsPool {
    /// Pool authority (typically protocol multisig)
    authority: PublicKey,

    /// Associated liquid staking pool
    staking_pool: PublicKey,

    /// Total MEV collected (lifetime)
    total_mev_collected: u64,

    /// MEV distributed to stakers (lifetime)
    total_mev_distributed: u64,

    /// Pending MEV awaiting distribution
    pending_mev: u64,

    /// Protocol's share of MEV (basis points)
    protocol_fee_bps: u16,

    /// Last distribution slot
    last_distribution_slot: u64,

    /// Distribution frequency (slots)
    distribution_interval: u64,

    /// Minimum MEV amount to trigger distribution
    min_distribution_amount: u64,

    /// Bump seed
    bump: u8,
}

#[solana]
#[account]
struct ValidatorMEVAccount {
    /// Validator identity
    validator: PublicKey,

    /// Parent MEV pool
    mev_pool: PublicKey,

    /// MEV earned by this validator
    mev_earned: u64,

    /// MEV claimed by this validator
    mev_claimed: u64,

    /// Commission rate for this validator (basis points)
    commission_bps: u16,

    /// Blocks produced with MEV
    blocks_with_mev: u64,

    /// Total tips received
    total_tips_received: u64,

    /// Last MEV claim epoch
    last_claim_epoch: u64,

    /// Bump seed
    bump: u8,
}

#[solana]
#[account]
struct BundleTipRecord {
    /// Block slot this tip was for
    slot: u64,

    /// Tip amount in lamports
    tip_amount: u64,

    /// Number of transactions in bundle
    tx_count: u8,

    /// Bundle landed successfully
    landed: bool,

    /// Timestamp
    timestamp: i64,
}

#[solana]
enum MEVDistributionStrategy {
    /// Distribute proportionally to stake
    ProRata,
    /// Distribute to validator that produced block
    ValidatorOnly,
    /// Split between validator and delegators
    Split { validator_share_bps: u16 },
    /// Compound into staking pool
    Compound,
}

#[solana]
#[account]
struct MEVDistributionConfig {
    /// Parent MEV pool
    mev_pool: PublicKey,

    /// Distribution strategy
    strategy: MEVDistributionStrategy,

    /// Minimum stake to receive MEV (prevents dust accounts)
    min_stake_for_mev: u64,

    /// Auto-compound threshold (if strategy supports)
    auto_compound_threshold: u64,

    /// Bump seed
    bump: u8,
}
```

### MEV Distribution Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    MEV Distribution Flow                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Bundle Tips ──► ┌────────────────┐                             │
│                  │ MEVRewardsPool │                             │
│  Block Rewards ─►│                │                             │
│                  └───────┬────────┘                             │
│                          │                                       │
│                          ▼                                       │
│                  ┌───────────────┐                              │
│                  │ Distribution  │                              │
│                  │ Engine        │                              │
│                  └───────┬───────┘                              │
│                          │                                       │
│         ┌────────────────┼────────────────┐                     │
│         ▼                ▼                ▼                     │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐                  │
│  │ Protocol │    │ Validator│    │ Stakers  │                  │
│  │ Treasury │    │ Share    │    │ Rewards  │                  │
│  │ (5-10%)  │    │ (10-20%) │    │ (70-85%) │                  │
│  └──────────┘    └──────────┘    └──────────┘                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Jito Integration Example

```lumos
// jito-integration.lumos - Jito-specific types

#[solana]
#[account]
struct JitoStakePool {
    /// Standard LST pool fields
    authority: PublicKey,
    jitosol_mint: PublicKey,
    sol_vault: PublicKey,

    /// Jito-specific: MEV rewards integration
    mev_rewards_pool: PublicKey,

    /// Total SOL including MEV rewards
    total_sol_with_mev: u64,

    /// JitoSOL supply
    jitosol_supply: u64,

    /// MEV boost APY (basis points, updated periodically)
    mev_apy_bps: u16,

    /// Base staking APY (basis points)
    base_apy_bps: u16,

    /// Combined APY (base + MEV)
    combined_apy_bps: u16,

    /// Last APY update timestamp
    last_apy_update: i64,

    /// Bump seed
    bump: u8,
}

#[solana]
#[account]
struct JitoValidatorHistory {
    /// Validator identity
    validator: PublicKey,

    /// Historical MEV performance (last 32 epochs)
    mev_performance: [u64; 32],

    /// Historical block production
    blocks_produced: [u32; 32],

    /// Current epoch index in circular buffer
    current_index: u8,

    /// Average MEV per block (lamports)
    avg_mev_per_block: u64,

    /// Bump seed
    bump: u8,
}
```

---

## Instruction Contexts

### Deposit SOL → Mint LST

```lumos
// Instructions for LST operations

#[solana]
#[instruction]
struct DepositSol {
    /// Amount of SOL to deposit (in lamports)
    amount: u64,
}

#[solana]
#[instruction]
struct DepositSolAccounts {
    #[account(mut)]
    user: PublicKey,           // Signer, pays SOL

    #[account(mut)]
    pool: PublicKey,           // LiquidStakingPool

    #[account(mut)]
    sol_vault: PublicKey,      // Pool's SOL vault

    #[account(mut)]
    lst_mint: PublicKey,       // LST token mint

    #[account(mut)]
    user_lst_account: PublicKey, // User's LST token account

    exchange_rate: PublicKey,  // Current exchange rate

    token_program: PublicKey,  // SPL Token program
    system_program: PublicKey, // System program
}
```

**Anchor Implementation:**

```rust
#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"lst_pool", pool.authority.as_ref()],
        bump = pool.bump,
        constraint = !pool.is_paused @ ErrorCode::PoolPaused,
    )]
    pub pool: Account<'info, LiquidStakingPool>,

    #[account(
        mut,
        constraint = sol_vault.key() == pool.sol_vault @ ErrorCode::InvalidVault,
    )]
    pub sol_vault: SystemAccount<'info>,

    #[account(
        mut,
        constraint = lst_mint.key() == pool.lst_mint @ ErrorCode::InvalidMint,
    )]
    pub lst_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = lst_mint,
        associated_token::authority = user,
    )]
    pub user_lst_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"exchange_rate", pool.key().as_ref()],
        bump = exchange_rate.bump,
    )]
    pub exchange_rate: Account<'info, ExchangeRateState>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> DepositSol<'info> {
    pub fn process(&mut self, amount: u64) -> Result<()> {
        // Validate minimum stake
        require!(
            amount >= self.pool.min_stake_amount,
            ErrorCode::BelowMinimumStake
        );

        // Check pool capacity
        if self.pool.max_pool_capacity > 0 {
            require!(
                self.pool.total_sol_value.checked_add(amount).unwrap()
                    <= self.pool.max_pool_capacity,
                ErrorCode::PoolCapacityExceeded
            );
        }

        // Calculate LST to mint using exchange rate
        let lst_amount = self.calculate_lst_amount(amount)?;

        // Transfer SOL to vault
        let transfer_ix = system_instruction::transfer(
            &self.user.key(),
            &self.sol_vault.key(),
            amount,
        );
        invoke(
            &transfer_ix,
            &[
                self.user.to_account_info(),
                self.sol_vault.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        // Mint LST to user
        let seeds = &[
            b"lst_pool",
            self.pool.authority.as_ref(),
            &[self.pool.bump],
        ];
        let signer = &[&seeds[..]];

        mint_to(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                MintTo {
                    mint: self.lst_mint.to_account_info(),
                    to: self.user_lst_account.to_account_info(),
                    authority: self.pool.to_account_info(),
                },
                signer,
            ),
            lst_amount,
        )?;

        // Update pool state
        self.pool.total_sol_deposited = self.pool.total_sol_deposited
            .checked_add(amount)
            .unwrap();
        self.pool.total_sol_value = self.pool.total_sol_value
            .checked_add(amount)
            .unwrap();
        self.pool.total_lst_minted = self.pool.total_lst_minted
            .checked_add(lst_amount)
            .unwrap();

        Ok(())
    }

    fn calculate_lst_amount(&self, sol_amount: u64) -> Result<u64> {
        let rate = &self.exchange_rate;

        // LST = SOL * denominator / numerator
        // (inverse because rate = SOL_value / LST_supply)
        let lst_amount = (sol_amount as u128)
            .checked_mul(rate.rate_denominator as u128)
            .unwrap()
            .checked_mul(rate.precision as u128)
            .unwrap()
            .checked_div(rate.rate_numerator as u128)
            .unwrap()
            .checked_div(rate.precision as u128)
            .unwrap() as u64;

        Ok(lst_amount)
    }
}
```

### Request Redemption (Delayed Unstake)

```rust
#[derive(Accounts)]
pub struct RequestRedemption<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"lst_pool", pool.authority.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, LiquidStakingPool>,

    #[account(
        mut,
        seeds = [b"redemption_queue", pool.key().as_ref()],
        bump = queue.bump,
        constraint = queue.is_active @ ErrorCode::QueueInactive,
    )]
    pub queue: Account<'info, RedemptionQueue>,

    #[account(
        init,
        payer = user,
        space = RedemptionTicket::LEN,
        seeds = [
            b"ticket",
            queue.key().as_ref(),
            user.key().as_ref(),
            queue.next_ticket_id.to_le_bytes().as_ref(),
        ],
        bump,
    )]
    pub ticket: Account<'info, RedemptionTicket>,

    #[account(
        mut,
        associated_token::mint = pool.lst_mint,
        associated_token::authority = user,
    )]
    pub user_lst_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = lst_mint.key() == pool.lst_mint,
    )]
    pub lst_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"exchange_rate", pool.key().as_ref()],
        bump = exchange_rate.bump,
    )]
    pub exchange_rate: Account<'info, ExchangeRateState>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> RequestRedemption<'info> {
    pub fn process(&mut self, lst_amount: u64, bump: u8) -> Result<()> {
        let clock = Clock::get()?;

        // Calculate SOL to receive
        let sol_amount = self.calculate_sol_amount(lst_amount)?;

        // Apply delayed unstake fee
        let fee = sol_amount
            .checked_mul(self.pool.delayed_unstake_fee_bps as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        let sol_after_fee = sol_amount.checked_sub(fee).unwrap();

        // Burn LST from user
        burn(
            CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.lst_mint.to_account_info(),
                    from: self.user_lst_account.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            ),
            lst_amount,
        )?;

        // Initialize ticket
        let ticket = &mut self.ticket;
        ticket.owner = self.user.key();
        ticket.queue = self.queue.key();
        ticket.ticket_id = self.queue.next_ticket_id;
        ticket.lst_amount = lst_amount;
        ticket.sol_amount = sol_after_fee;
        ticket.exchange_rate_at_request = self.exchange_rate.rate_numerator;
        ticket.created_at = clock.unix_timestamp;
        ticket.claimable_at = clock.unix_timestamp
            .checked_add(self.queue.cooldown_seconds as i64)
            .unwrap();
        ticket.state = RedemptionState::Pending;
        ticket.bump = bump;

        // Update queue state
        self.queue.total_pending_lst = self.queue.total_pending_lst
            .checked_add(lst_amount)
            .unwrap();
        self.queue.total_reserved_sol = self.queue.total_reserved_sol
            .checked_add(sol_after_fee)
            .unwrap();
        self.queue.next_ticket_id = self.queue.next_ticket_id
            .checked_add(1)
            .unwrap();

        // Update pool state
        self.pool.total_lst_minted = self.pool.total_lst_minted
            .checked_sub(lst_amount)
            .unwrap();

        Ok(())
    }

    fn calculate_sol_amount(&self, lst_amount: u64) -> Result<u64> {
        let rate = &self.exchange_rate;

        // SOL = LST * numerator / denominator
        let sol_amount = (lst_amount as u128)
            .checked_mul(rate.rate_numerator as u128)
            .unwrap()
            .checked_div(rate.rate_denominator as u128)
            .unwrap()
            .checked_div(rate.precision as u128)
            .unwrap() as u64;

        Ok(sol_amount)
    }
}
```

### Distribute MEV Rewards

```rust
#[derive(Accounts)]
pub struct DistributeMEV<'info> {
    #[account(
        constraint = authority.key() == mev_pool.authority @ ErrorCode::Unauthorized,
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"mev_pool", staking_pool.key().as_ref()],
        bump = mev_pool.bump,
    )]
    pub mev_pool: Account<'info, MEVRewardsPool>,

    #[account(
        mut,
        seeds = [b"lst_pool", staking_pool.authority.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, LiquidStakingPool>,

    #[account(
        mut,
        seeds = [b"exchange_rate", staking_pool.key().as_ref()],
        bump = exchange_rate.bump,
    )]
    pub exchange_rate: Account<'info, ExchangeRateState>,

    /// CHECK: Protocol treasury for fees
    #[account(mut)]
    pub protocol_treasury: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> DistributeMEV<'info> {
    pub fn process(&mut self) -> Result<()> {
        let clock = Clock::get()?;
        let slot = clock.slot;

        // Check distribution interval
        require!(
            slot >= self.mev_pool.last_distribution_slot
                .checked_add(self.mev_pool.distribution_interval)
                .unwrap(),
            ErrorCode::TooEarlyForDistribution
        );

        // Check minimum amount
        require!(
            self.mev_pool.pending_mev >= self.mev_pool.min_distribution_amount,
            ErrorCode::BelowMinimumDistribution
        );

        let pending = self.mev_pool.pending_mev;

        // Calculate protocol fee
        let protocol_fee = pending
            .checked_mul(self.mev_pool.protocol_fee_bps as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();

        // Remaining goes to stakers (via exchange rate increase)
        let staker_rewards = pending.checked_sub(protocol_fee).unwrap();

        // Transfer protocol fee to treasury
        **self.mev_pool.to_account_info().try_borrow_mut_lamports()? -= protocol_fee;
        **self.protocol_treasury.try_borrow_mut_lamports()? += protocol_fee;

        // Update staking pool's total SOL value (increases exchange rate)
        self.staking_pool.total_sol_value = self.staking_pool.total_sol_value
            .checked_add(staker_rewards)
            .unwrap();

        // Update exchange rate
        let rate = &mut self.exchange_rate;
        rate.rate_numerator = self.staking_pool.total_sol_value
            .checked_mul(rate.precision)
            .unwrap();
        rate.rate_denominator = self.staking_pool.total_lst_minted;
        rate.last_update_slot = slot;
        rate.last_update_timestamp = clock.unix_timestamp;

        // Update MEV pool
        self.mev_pool.total_mev_distributed = self.mev_pool.total_mev_distributed
            .checked_add(pending)
            .unwrap();
        self.mev_pool.pending_mev = 0;
        self.mev_pool.last_distribution_slot = slot;

        Ok(())
    }
}
```

---

## Exchange Rate Calculations

### Precision Strategy

Exchange rates require careful precision handling to avoid rounding errors:

```rust
// Constants
pub const RATE_PRECISION: u64 = 1_000_000_000; // 1e9
pub const BPS_PRECISION: u64 = 10_000;         // Basis points

// Exchange rate calculation
pub fn calculate_exchange_rate(
    total_sol_value: u64,
    total_lst_supply: u64,
) -> (u64, u64) {
    // rate = (total_sol_value * PRECISION) / total_lst_supply
    let numerator = (total_sol_value as u128)
        .checked_mul(RATE_PRECISION as u128)
        .unwrap();
    let denominator = total_lst_supply as u128;

    (numerator as u64, denominator as u64)
}

// SOL to LST conversion (for deposits)
pub fn sol_to_lst(
    sol_amount: u64,
    rate_numerator: u64,
    rate_denominator: u64,
) -> u64 {
    // lst = sol * denominator / numerator
    // (we want more LST when rate is lower)
    (sol_amount as u128)
        .checked_mul(rate_denominator as u128)
        .unwrap()
        .checked_mul(RATE_PRECISION as u128)
        .unwrap()
        .checked_div(rate_numerator as u128)
        .unwrap()
        .checked_div(RATE_PRECISION as u128)
        .unwrap() as u64
}

// LST to SOL conversion (for withdrawals)
pub fn lst_to_sol(
    lst_amount: u64,
    rate_numerator: u64,
    rate_denominator: u64,
) -> u64 {
    // sol = lst * numerator / denominator
    (lst_amount as u128)
        .checked_mul(rate_numerator as u128)
        .unwrap()
        .checked_div(rate_denominator as u128)
        .unwrap()
        .checked_div(RATE_PRECISION as u128)
        .unwrap() as u64
}
```

### Rounding Strategies

```rust
// For deposits: Round DOWN (user gets slightly less LST)
// This protects the pool from being drained
pub fn sol_to_lst_floor(sol_amount: u64, rate: &ExchangeRateState) -> u64 {
    let result = (sol_amount as u128)
        .checked_mul(rate.rate_denominator as u128)
        .unwrap()
        .checked_div(rate.rate_numerator as u128)
        .unwrap();

    result as u64 // Implicit floor
}

// For withdrawals: Round DOWN (user gets slightly less SOL)
// This also protects the pool
pub fn lst_to_sol_floor(lst_amount: u64, rate: &ExchangeRateState) -> u64 {
    let result = (lst_amount as u128)
        .checked_mul(rate.rate_numerator as u128)
        .unwrap()
        .checked_div(rate.rate_denominator as u128)
        .unwrap();

    result as u64 // Implicit floor
}
```

### APY Calculation

```rust
// Calculate APY from exchange rate change
pub fn calculate_apy(
    current_rate: u64,
    previous_rate: u64,
    time_elapsed_seconds: u64,
) -> u16 {
    // rate_change = (current - previous) / previous
    let rate_change_bps = (current_rate as u128)
        .checked_sub(previous_rate as u128)
        .unwrap()
        .checked_mul(BPS_PRECISION as u128)
        .unwrap()
        .checked_div(previous_rate as u128)
        .unwrap() as u64;

    // Annualize: APY = rate_change * (seconds_per_year / time_elapsed)
    let seconds_per_year: u64 = 365 * 24 * 60 * 60;
    let apy_bps = rate_change_bps
        .checked_mul(seconds_per_year)
        .unwrap()
        .checked_div(time_elapsed_seconds)
        .unwrap();

    apy_bps as u16
}
```

---

## TypeScript Client Integration

### Exchange Rate Helpers

```typescript
// exchange-rate.ts
import { BN } from '@coral-xyz/anchor';

export const RATE_PRECISION = new BN(1_000_000_000);
export const BPS_PRECISION = new BN(10_000);

export interface ExchangeRate {
  numerator: BN;
  denominator: BN;
  precision: BN;
}

/**
 * Convert SOL amount to LST amount using current exchange rate
 */
export function solToLst(
  solAmount: BN,
  rate: ExchangeRate
): BN {
  return solAmount
    .mul(rate.denominator)
    .mul(rate.precision)
    .div(rate.numerator)
    .div(rate.precision);
}

/**
 * Convert LST amount to SOL amount using current exchange rate
 */
export function lstToSol(
  lstAmount: BN,
  rate: ExchangeRate
): BN {
  return lstAmount
    .mul(rate.numerator)
    .div(rate.denominator)
    .div(rate.precision);
}

/**
 * Calculate current exchange rate from pool state
 */
export function calculateRate(
  totalSolValue: BN,
  totalLstSupply: BN
): ExchangeRate {
  return {
    numerator: totalSolValue.mul(RATE_PRECISION),
    denominator: totalLstSupply,
    precision: RATE_PRECISION,
  };
}

/**
 * Format exchange rate for display (e.g., "1 LST = 1.0523 SOL")
 */
export function formatExchangeRate(rate: ExchangeRate): string {
  const rateValue = rate.numerator
    .mul(new BN(10000))
    .div(rate.denominator)
    .div(rate.precision);

  return `1 LST = ${(rateValue.toNumber() / 10000).toFixed(4)} SOL`;
}

/**
 * Calculate APY from rate change
 */
export function calculateAPY(
  currentRate: BN,
  previousRate: BN,
  timeElapsedSeconds: number
): number {
  const SECONDS_PER_YEAR = 365 * 24 * 60 * 60;

  const rateChange = currentRate.sub(previousRate);
  const rateChangeBps = rateChange
    .mul(BPS_PRECISION)
    .div(previousRate);

  const annualizedBps = rateChangeBps
    .muln(SECONDS_PER_YEAR)
    .divn(timeElapsedSeconds);

  return annualizedBps.toNumber() / 100; // Convert bps to percentage
}
```

### React Hooks for LST Operations

```typescript
// hooks/useLiquidStaking.ts
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { BN } from '@coral-xyz/anchor';
import { PublicKey, Transaction } from '@solana/web3.js';
import {
  LiquidStakingPool,
  ExchangeRateState,
  LiquidStakingPoolSchema,
  ExchangeRateStateSchema
} from '../generated';
import { solToLst, lstToSol, calculateAPY, formatExchangeRate } from './exchange-rate';

export function useLiquidStakingPool(poolAddress: PublicKey) {
  const { connection } = useConnection();

  return useQuery({
    queryKey: ['lst-pool', poolAddress.toString()],
    queryFn: async () => {
      const accountInfo = await connection.getAccountInfo(poolAddress);
      if (!accountInfo) throw new Error('Pool not found');

      // Skip 8-byte discriminator
      const data = accountInfo.data.slice(8);
      const pool = LiquidStakingPoolSchema.decode(data) as LiquidStakingPool;

      // Fetch exchange rate
      const [exchangeRatePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('exchange_rate'), poolAddress.toBuffer()],
        PROGRAM_ID
      );

      const rateInfo = await connection.getAccountInfo(exchangeRatePDA);
      const rateData = rateInfo!.data.slice(8);
      const exchangeRate = ExchangeRateStateSchema.decode(rateData) as ExchangeRateState;

      return {
        pool,
        exchangeRate,
        formattedRate: formatExchangeRate({
          numerator: new BN(exchangeRate.rateNumerator),
          denominator: new BN(exchangeRate.rateDenominator),
          precision: new BN(exchangeRate.precision),
        }),
      };
    },
    refetchInterval: 30000, // Refresh every 30 seconds
  });
}

export function useDeposit(poolAddress: PublicKey) {
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (solAmount: number) => {
      if (!publicKey) throw new Error('Wallet not connected');

      const lamports = new BN(solAmount * 1e9);

      // Build deposit instruction
      const ix = await program.methods
        .depositSol(lamports)
        .accounts({
          user: publicKey,
          pool: poolAddress,
          // ... other accounts
        })
        .instruction();

      const tx = new Transaction().add(ix);
      const signature = await sendTransaction(tx, connection);
      await connection.confirmTransaction(signature);

      return signature;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['lst-pool', poolAddress.toString()] });
      queryClient.invalidateQueries({ queryKey: ['user-balance'] });
    },
  });
}

export function useRequestRedemption(poolAddress: PublicKey) {
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (lstAmount: number) => {
      if (!publicKey) throw new Error('Wallet not connected');

      const amount = new BN(lstAmount * 1e9);

      // Get next ticket ID for PDA derivation
      const poolData = await program.account.liquidStakingPool.fetch(poolAddress);
      const [queuePDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('redemption_queue'), poolAddress.toBuffer()],
        PROGRAM_ID
      );
      const queueData = await program.account.redemptionQueue.fetch(queuePDA);

      const [ticketPDA] = PublicKey.findProgramAddressSync(
        [
          Buffer.from('ticket'),
          queuePDA.toBuffer(),
          publicKey.toBuffer(),
          new BN(queueData.nextTicketId).toArrayLike(Buffer, 'le', 8),
        ],
        PROGRAM_ID
      );

      const ix = await program.methods
        .requestRedemption(amount)
        .accounts({
          user: publicKey,
          pool: poolAddress,
          queue: queuePDA,
          ticket: ticketPDA,
          // ... other accounts
        })
        .instruction();

      const tx = new Transaction().add(ix);
      const signature = await sendTransaction(tx, connection);
      await connection.confirmTransaction(signature);

      return { signature, ticketAddress: ticketPDA };
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['lst-pool'] });
      queryClient.invalidateQueries({ queryKey: ['redemption-tickets'] });
    },
  });
}

export function useRedemptionTickets(userAddress: PublicKey, queueAddress: PublicKey) {
  const { connection } = useConnection();

  return useQuery({
    queryKey: ['redemption-tickets', userAddress.toString()],
    queryFn: async () => {
      // Fetch all tickets for this user
      const tickets = await program.account.redemptionTicket.all([
        {
          memcmp: {
            offset: 8, // After discriminator
            bytes: userAddress.toBase58(),
          },
        },
        {
          memcmp: {
            offset: 8 + 32, // After owner
            bytes: queueAddress.toBase58(),
          },
        },
      ]);

      return tickets.map(t => ({
        address: t.publicKey,
        ...t.account,
        isClaimable: t.account.state.kind === 'Pending' &&
          Date.now() / 1000 >= t.account.claimableAt,
      }));
    },
  });
}
```

### MEV Dashboard Component

```typescript
// components/MEVDashboard.tsx
import React from 'react';
import { useQuery } from '@tanstack/react-query';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';

interface MEVDashboardProps {
  mevPoolAddress: PublicKey;
}

export function MEVDashboard({ mevPoolAddress }: MEVDashboardProps) {
  const { data: mevData, isLoading } = useQuery({
    queryKey: ['mev-pool', mevPoolAddress.toString()],
    queryFn: async () => {
      const pool = await program.account.mevRewardsPool.fetch(mevPoolAddress);

      return {
        totalCollected: new BN(pool.totalMevCollected).toNumber() / 1e9,
        totalDistributed: new BN(pool.totalMevDistributed).toNumber() / 1e9,
        pending: new BN(pool.pendingMev).toNumber() / 1e9,
        protocolFeeBps: pool.protocolFeeBps,
        lastDistributionSlot: pool.lastDistributionSlot,
      };
    },
    refetchInterval: 10000,
  });

  if (isLoading) return <div>Loading MEV data...</div>;

  return (
    <div className="mev-dashboard">
      <h3>MEV Rewards</h3>

      <div className="stat-grid">
        <div className="stat">
          <label>Total MEV Collected</label>
          <value>{mevData?.totalCollected.toFixed(4)} SOL</value>
        </div>

        <div className="stat">
          <label>Total Distributed</label>
          <value>{mevData?.totalDistributed.toFixed(4)} SOL</value>
        </div>

        <div className="stat">
          <label>Pending Distribution</label>
          <value>{mevData?.pending.toFixed(4)} SOL</value>
        </div>

        <div className="stat">
          <label>Protocol Fee</label>
          <value>{(mevData?.protocolFeeBps ?? 0) / 100}%</value>
        </div>
      </div>

      <div className="mev-chart">
        {/* Add chart showing MEV over time */}
      </div>
    </div>
  );
}
```

---

## Complete Example: Mini LST Protocol

### Full Schema File

```lumos
// mini-lst/schema.lumos
// A minimal but complete LST protocol schema

#[solana]
#[account]
struct MiniLSTPool {
    /// Pool authority
    authority: PublicKey,
    /// LST mint address
    lst_mint: PublicKey,
    /// Native SOL vault
    sol_vault: PublicKey,
    /// Total SOL in pool (deposits + rewards)
    total_sol: u64,
    /// Total LST minted
    total_lst: u64,
    /// Annual fee in basis points (e.g., 100 = 1%)
    annual_fee_bps: u16,
    /// Instant unstake fee (basis points)
    instant_fee_bps: u16,
    /// Delayed unstake cooldown (seconds)
    cooldown_seconds: u64,
    /// Pool is active
    is_active: bool,
    /// Bump seed
    bump: u8,
}

#[solana]
#[account]
struct MiniLSTRate {
    /// Parent pool
    pool: PublicKey,
    /// Rate = numerator / denominator (scaled by precision)
    numerator: u64,
    denominator: u64,
    precision: u64,
    /// Last update
    last_update: i64,
    /// Bump seed
    bump: u8,
}

#[solana]
#[account]
struct UnstakeTicket {
    /// Ticket owner
    owner: PublicKey,
    /// Parent pool
    pool: PublicKey,
    /// LST amount burned
    lst_amount: u64,
    /// SOL amount to receive
    sol_amount: u64,
    /// When ticket becomes claimable
    claimable_at: i64,
    /// Ticket state
    state: TicketState,
    /// Bump seed
    bump: u8,
}

#[solana]
enum TicketState {
    Pending,
    Claimable,
    Claimed,
}

// MEV integration
#[solana]
#[account]
struct MiniMEVPool {
    /// Associated LST pool
    lst_pool: PublicKey,
    /// Pending MEV to distribute
    pending_mev: u64,
    /// Total MEV distributed
    total_distributed: u64,
    /// Protocol fee (basis points)
    fee_bps: u16,
    /// Bump seed
    bump: u8,
}
```

### Generated Code Usage

```rust
// programs/mini-lst/src/lib.rs
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn};

// Import generated types
mod generated;
use generated::*;

declare_id!("MiniLST111111111111111111111111111111111111");

#[program]
pub mod mini_lst {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        annual_fee_bps: u16,
        instant_fee_bps: u16,
        cooldown_seconds: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.lst_mint = ctx.accounts.lst_mint.key();
        pool.sol_vault = ctx.accounts.sol_vault.key();
        pool.total_sol = 0;
        pool.total_lst = 0;
        pool.annual_fee_bps = annual_fee_bps;
        pool.instant_fee_bps = instant_fee_bps;
        pool.cooldown_seconds = cooldown_seconds;
        pool.is_active = true;
        pool.bump = ctx.bumps.pool;

        // Initialize rate at 1:1
        let rate = &mut ctx.accounts.rate;
        rate.pool = pool.key();
        rate.numerator = RATE_PRECISION;
        rate.denominator = RATE_PRECISION;
        rate.precision = RATE_PRECISION;
        rate.last_update = Clock::get()?.unix_timestamp;
        rate.bump = ctx.bumps.rate;

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, sol_amount: u64) -> Result<()> {
        require!(ctx.accounts.pool.is_active, ErrorCode::PoolInactive);
        require!(sol_amount > 0, ErrorCode::ZeroAmount);

        // Calculate LST to mint
        let rate = &ctx.accounts.rate;
        let lst_amount = sol_to_lst(sol_amount, rate.numerator, rate.denominator);

        // Transfer SOL to vault
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.user.to_account_info(),
                    to: ctx.accounts.sol_vault.to_account_info(),
                },
            ),
            sol_amount,
        )?;

        // Mint LST
        let seeds = &[b"pool", ctx.accounts.pool.authority.as_ref(), &[ctx.accounts.pool.bump]];
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.lst_mint.to_account_info(),
                    to: ctx.accounts.user_lst.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                &[seeds],
            ),
            lst_amount,
        )?;

        // Update state
        let pool = &mut ctx.accounts.pool;
        pool.total_sol = pool.total_sol.checked_add(sol_amount).unwrap();
        pool.total_lst = pool.total_lst.checked_add(lst_amount).unwrap();

        emit!(StakeEvent {
            user: ctx.accounts.user.key(),
            sol_amount,
            lst_amount,
            rate: rate.numerator,
        });

        Ok(())
    }

    pub fn instant_unstake(ctx: Context<InstantUnstake>, lst_amount: u64) -> Result<()> {
        require!(lst_amount > 0, ErrorCode::ZeroAmount);

        let rate = &ctx.accounts.rate;
        let gross_sol = lst_to_sol(lst_amount, rate.numerator, rate.denominator);

        // Apply instant unstake fee
        let fee = gross_sol
            .checked_mul(ctx.accounts.pool.instant_fee_bps as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        let net_sol = gross_sol.checked_sub(fee).unwrap();

        // Burn LST
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.lst_mint.to_account_info(),
                    from: ctx.accounts.user_lst.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            lst_amount,
        )?;

        // Transfer SOL to user
        **ctx.accounts.sol_vault.try_borrow_mut_lamports()? -= net_sol;
        **ctx.accounts.user.try_borrow_mut_lamports()? += net_sol;

        // Update state
        let pool = &mut ctx.accounts.pool;
        pool.total_sol = pool.total_sol.checked_sub(gross_sol).unwrap();
        pool.total_lst = pool.total_lst.checked_sub(lst_amount).unwrap();

        Ok(())
    }

    pub fn distribute_mev(ctx: Context<DistributeMEV>) -> Result<()> {
        let mev_pool = &mut ctx.accounts.mev_pool;
        let pending = mev_pool.pending_mev;

        require!(pending > 0, ErrorCode::NoMEVToDistribute);

        // Protocol fee
        let fee = pending
            .checked_mul(mev_pool.fee_bps as u64)
            .unwrap()
            .checked_div(10000)
            .unwrap();
        let to_stakers = pending.checked_sub(fee).unwrap();

        // Increase pool's total SOL (increases exchange rate)
        let pool = &mut ctx.accounts.pool;
        pool.total_sol = pool.total_sol.checked_add(to_stakers).unwrap();

        // Update exchange rate
        let rate = &mut ctx.accounts.rate;
        rate.numerator = pool.total_sol.checked_mul(rate.precision).unwrap();
        rate.denominator = pool.total_lst;
        rate.last_update = Clock::get()?.unix_timestamp;

        // Clear pending
        mev_pool.total_distributed = mev_pool.total_distributed
            .checked_add(pending)
            .unwrap();
        mev_pool.pending_mev = 0;

        Ok(())
    }
}

// Helper functions
const RATE_PRECISION: u64 = 1_000_000_000;

fn sol_to_lst(sol: u64, rate_num: u64, rate_denom: u64) -> u64 {
    (sol as u128)
        .checked_mul(rate_denom as u128)
        .unwrap()
        .checked_div(rate_num as u128)
        .unwrap() as u64
}

fn lst_to_sol(lst: u64, rate_num: u64, rate_denom: u64) -> u64 {
    (lst as u128)
        .checked_mul(rate_num as u128)
        .unwrap()
        .checked_div(rate_denom as u128)
        .unwrap() as u64
}

// Events
#[event]
pub struct StakeEvent {
    pub user: Pubkey,
    pub sol_amount: u64,
    pub lst_amount: u64,
    pub rate: u64,
}
```

### TypeScript Client

```typescript
// client/mini-lst.ts
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from '@solana/spl-token';

export class MiniLSTClient {
  constructor(
    private program: Program,
    private poolAddress: PublicKey
  ) {}

  static async initialize(
    program: Program,
    authority: PublicKey,
    lstMint: PublicKey,
    annualFeeBps: number,
    instantFeeBps: number,
    cooldownSeconds: number
  ): Promise<MiniLSTClient> {
    const [poolPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('pool'), authority.toBuffer()],
      program.programId
    );

    const [ratePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('rate'), poolPDA.toBuffer()],
      program.programId
    );

    const [solVaultPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault'), poolPDA.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize(annualFeeBps, instantFeeBps, new BN(cooldownSeconds))
      .accounts({
        authority,
        pool: poolPDA,
        rate: ratePDA,
        lstMint,
        solVault: solVaultPDA,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return new MiniLSTClient(program, poolPDA);
  }

  async stake(user: PublicKey, solAmount: number): Promise<string> {
    const pool = await this.program.account.miniLstPool.fetch(this.poolAddress);

    const [ratePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('rate'), this.poolAddress.toBuffer()],
      this.program.programId
    );

    const userLstAccount = await getAssociatedTokenAddress(
      pool.lstMint,
      user
    );

    const lamports = new BN(solAmount * LAMPORTS_PER_SOL);

    return this.program.methods
      .stake(lamports)
      .accounts({
        user,
        pool: this.poolAddress,
        rate: ratePDA,
        lstMint: pool.lstMint,
        solVault: pool.solVault,
        userLst: userLstAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  }

  async instantUnstake(user: PublicKey, lstAmount: number): Promise<string> {
    const pool = await this.program.account.miniLstPool.fetch(this.poolAddress);

    const [ratePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('rate'), this.poolAddress.toBuffer()],
      this.program.programId
    );

    const userLstAccount = await getAssociatedTokenAddress(
      pool.lstMint,
      user
    );

    const amount = new BN(lstAmount * LAMPORTS_PER_SOL);

    return this.program.methods
      .instantUnstake(amount)
      .accounts({
        user,
        pool: this.poolAddress,
        rate: ratePDA,
        lstMint: pool.lstMint,
        solVault: pool.solVault,
        userLst: userLstAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  }

  async getPoolState(): Promise<{
    totalSol: number;
    totalLst: number;
    exchangeRate: number;
    apy: number;
  }> {
    const pool = await this.program.account.miniLstPool.fetch(this.poolAddress);

    const [ratePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from('rate'), this.poolAddress.toBuffer()],
      this.program.programId
    );
    const rate = await this.program.account.miniLstRate.fetch(ratePDA);

    const exchangeRate = new BN(rate.numerator)
      .mul(new BN(10000))
      .div(new BN(rate.denominator))
      .toNumber() / 10000;

    return {
      totalSol: new BN(pool.totalSol).toNumber() / LAMPORTS_PER_SOL,
      totalLst: new BN(pool.totalLst).toNumber() / LAMPORTS_PER_SOL,
      exchangeRate,
      apy: 0, // Calculate from historical rate changes
    };
  }
}
```

---

## Best Practices & Security

### Rate Manipulation Prevention

```rust
// 1. Limit rate change per update
pub const MAX_RATE_CHANGE_PER_UPDATE_BPS: u64 = 100; // 1% max change

pub fn update_rate_safely(
    current_rate: u64,
    new_rate: u64,
) -> Result<u64> {
    let max_increase = current_rate
        .checked_mul(MAX_RATE_CHANGE_PER_UPDATE_BPS)
        .unwrap()
        .checked_div(10000)
        .unwrap();

    let max_rate = current_rate.checked_add(max_increase).unwrap();

    require!(new_rate <= max_rate, ErrorCode::RateChangeExceedsLimit);

    Ok(new_rate)
}

// 2. Time-locked rate updates
pub const MIN_RATE_UPDATE_INTERVAL: i64 = 3600; // 1 hour minimum

pub fn check_rate_update_allowed(last_update: i64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        now >= last_update + MIN_RATE_UPDATE_INTERVAL,
        ErrorCode::RateUpdateTooSoon
    );
    Ok(())
}
```

### Slashing Protection

```rust
// Track slashing events and pause if necessary
#[account]
pub struct SlashingProtection {
    pub pool: Pubkey,
    pub total_slashed: u64,
    pub slash_events: u32,
    pub last_slash_epoch: u64,
    pub auto_pause_threshold: u64, // Pause if slashed more than this
    pub is_paused_due_to_slash: bool,
}

pub fn handle_slash_event(
    protection: &mut SlashingProtection,
    slash_amount: u64,
    pool: &mut LiquidStakingPool,
) -> Result<()> {
    protection.total_slashed = protection.total_slashed
        .checked_add(slash_amount)
        .unwrap();
    protection.slash_events += 1;
    protection.last_slash_epoch = Clock::get()?.epoch;

    // Auto-pause if threshold exceeded
    if protection.total_slashed >= protection.auto_pause_threshold {
        pool.is_paused = true;
        protection.is_paused_due_to_slash = true;
    }

    Ok(())
}
```

### MEV Sandwich Attack Considerations

```rust
// 1. Use deadline for swap operations
pub fn stake_with_deadline(
    ctx: Context<Stake>,
    sol_amount: u64,
    min_lst_out: u64,  // Slippage protection
    deadline: i64,      // Transaction deadline
) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(now <= deadline, ErrorCode::TransactionExpired);

    let lst_amount = calculate_lst_amount(sol_amount, &ctx.accounts.rate);
    require!(lst_amount >= min_lst_out, ErrorCode::SlippageExceeded);

    // ... rest of stake logic
    Ok(())
}

// 2. Commit-reveal for large operations
#[account]
pub struct StakeCommitment {
    pub user: Pubkey,
    pub commitment_hash: [u8; 32], // hash(sol_amount, secret)
    pub commit_slot: u64,
    pub is_revealed: bool,
}

pub const COMMIT_REVEAL_DELAY: u64 = 2; // 2 slots minimum
```

### Audit Checklist for LST Protocols

- [ ] Exchange rate calculation uses checked arithmetic
- [ ] Rounding favors the protocol (floor for both deposit and withdraw)
- [ ] Rate updates are time-limited and change-limited
- [ ] Slippage protection on all swap operations
- [ ] Minimum stake amounts prevent dust attacks
- [ ] Pool capacity limits prevent concentration risk
- [ ] Redemption queue prevents instant bank runs
- [ ] MEV distribution is atomic and manipulation-resistant
- [ ] Authority keys use multisig or timelock
- [ ] Upgrade authority is properly secured
- [ ] Emergency pause functionality exists
- [ ] Slashing events are handled gracefully

---

## Related Guides

After mastering LST/MEV integration, explore these related guides:

**Foundation Guides:**
- [LUMOS + Anchor Integration](/guides/anchor-integration) - Schema-first Anchor development
- [LUMOS + web3.js Integration](/guides/web3js-integration) - TypeScript client patterns
- [LUMOS + Solana CLI Integration](/guides/solana-cli-integration) - Deployment workflows

**Use Case Guides:**
- [DeFi Projects Use Case](/guides/use-cases/defi) - Staking pool fundamentals
- [NFT Marketplaces](/guides/use-cases/nft) - Token standards and metadata

**Migration Guides:**
- [Migration: TypeScript → LUMOS](/guides/migration-typescript) - Migrate existing clients
- [Migration: Anchor → LUMOS](/guides/migration-anchor) - Add LUMOS to existing programs

---

## Summary

LUMOS provides critical advantages for LST and MEV protocols:

| Challenge | LUMOS Solution |
|-----------|---------------|
| Exchange rate precision | u64 warnings, consistent serialization |
| Complex account hierarchies | Automatic PDA helpers, type safety |
| MEV amount handling | BigInt warnings, JSDoc documentation |
| Audit readiness | Predictable patterns, clear types |
| Client synchronization | Identical Rust/TypeScript types |

By defining your LST and MEV schemas in LUMOS, you get:
- **Type-safe exchange rates** that work identically in Rust and TypeScript
- **Automatic account size calculation** for complex nested structures
- **Built-in precision warnings** for large MEV amounts
- **Consistent serialization** that eliminates a class of bugs

Start with the Mini LST example and expand based on your protocol's needs.
