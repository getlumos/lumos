# Error Handling & Validation Patterns

This guide covers comprehensive error handling and validation patterns for LUMOS-generated code. On Solana, there's no transaction rollback - errors must be caught BEFORE state changes. This guide shows you how to build robust, production-ready programs.

```
┌─────────────────────────────────────────────────────────────────┐
│              Error Handling in Solana Programs                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Input ──► Validation ──► State Change ──► Success        │
│                     │                                           │
│                     ▼                                           │
│              ┌──────────┐                                       │
│              │  ERROR   │ ◄── No rollback on Solana!           │
│              │  RETURN  │     Validate BEFORE changing state    │
│              └──────────┘                                       │
│                                                                 │
│  Key Principle: Fail fast, fail early, fail safely             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Table of Contents

1. [Introduction](#introduction)
2. [Validation Patterns in Rust Handlers](#validation-patterns-in-rust-handlers)
3. [Custom Error Types](#custom-error-types)
4. [Anchor Constraint Validation](#anchor-constraint-validation)
5. [TypeScript Client Error Handling](#typescript-client-error-handling)
6. [Testing Error Paths](#testing-error-paths)
7. [Common Errors & Solutions](#common-errors--solutions)
8. [Anti-Patterns & Pitfalls](#anti-patterns--pitfalls)
9. [Related Guides](#related-guides)

---

## Introduction

### Why Validation Matters on Solana

Unlike traditional databases, Solana has **no automatic rollback**:

```
Traditional DB:
  BEGIN TRANSACTION
  UPDATE balance -= 100  ◄── If error here...
  UPDATE inventory += 1
  COMMIT                 ◄── ...this never happens, DB rolls back

Solana:
  Instruction 1: balance -= 100  ◄── Succeeds, state changed!
  Instruction 2: inventory += 1  ◄── Error here = stuck state!
```

**Consequences of poor validation:**
- Lost funds (transferred but item not delivered)
- Stuck accounts (partially initialized)
- Exploitable states (invariants broken)
- Unrecoverable errors (no undo)

### The Validation Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                    Validation Layers                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Layer 1: Schema Validation (LUMOS)                            │
│  ├── Type safety (u64, PublicKey, etc.)                        │
│  ├── Field constraints (#[max(32)])                            │
│  └── Structure validation (required fields)                     │
│                                                                 │
│  Layer 2: Account Constraints (Anchor)                          │
│  ├── Account ownership                                          │
│  ├── Signer requirements                                        │
│  ├── PDA derivation                                             │
│  └── has_one, constraint attributes                            │
│                                                                 │
│  Layer 3: Business Logic Validation (Your Code)                │
│  ├── State requirements                                         │
│  ├── Arithmetic bounds                                          │
│  ├── Permission checks                                          │
│  └── Cross-account invariants                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Error Handling Philosophy

**Rust (Program Side):**
- Use `Result<T, Error>` for all fallible operations
- Prefer `require!()` macro for assertions
- Use `checked_*` arithmetic to prevent overflow
- Return descriptive error codes

**TypeScript (Client Side):**
- Handle null/undefined explicitly
- Use type narrowing for enums
- Wrap network calls in try/catch
- Provide user-friendly error messages

---

## Validation Patterns in Rust Handlers

### Input Validation with require!()

The `require!()` macro is your primary validation tool:

```rust
use anchor_lang::prelude::*;

pub fn transfer(ctx: Context<Transfer>, amount: u64) -> Result<()> {
    let sender = &ctx.accounts.sender;
    let recipient = &ctx.accounts.recipient;

    // Validate amount is non-zero
    require!(amount > 0, ErrorCode::ZeroAmount);

    // Validate sufficient balance
    require!(
        sender.balance >= amount,
        ErrorCode::InsufficientBalance
    );

    // Validate recipient can receive
    require!(
        recipient.balance.checked_add(amount).is_some(),
        ErrorCode::RecipientOverflow
    );

    // Validate not sending to self
    require!(
        sender.key() != recipient.key(),
        ErrorCode::SelfTransfer
    );

    // All validations passed - safe to modify state
    // ... transfer logic

    Ok(())
}
```

### Arithmetic Safety

**NEVER use raw arithmetic on untrusted values:**

```rust
// ❌ DANGEROUS: Can overflow/underflow
pub fn bad_add(ctx: Context<Update>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.account;
    account.balance += amount;  // Panic on overflow!
    Ok(())
}

// ❌ DANGEROUS: Can underflow
pub fn bad_sub(ctx: Context<Update>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.account;
    account.balance -= amount;  // Panic on underflow!
    Ok(())
}
```

**Use checked arithmetic:**

```rust
// ✅ SAFE: Returns error on overflow
pub fn safe_add(ctx: Context<Update>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.account;

    account.balance = account.balance
        .checked_add(amount)
        .ok_or(ErrorCode::Overflow)?;

    Ok(())
}

// ✅ SAFE: Returns error on underflow
pub fn safe_sub(ctx: Context<Update>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.account;

    account.balance = account.balance
        .checked_sub(amount)
        .ok_or(ErrorCode::InsufficientBalance)?;

    Ok(())
}
```

**Arithmetic function reference:**

| Function | Behavior on Overflow | Use Case |
|----------|---------------------|----------|
| `checked_add()` | Returns `None` | When overflow is an error |
| `checked_sub()` | Returns `None` | When underflow is an error |
| `checked_mul()` | Returns `None` | Multiplication with bounds |
| `checked_div()` | Returns `None` | Division (also checks /0) |
| `saturating_add()` | Returns `MAX` | Counters, non-critical values |
| `saturating_sub()` | Returns `0` | Decrements that shouldn't go negative |
| `wrapping_add()` | Wraps around | Rarely needed, be careful! |

**Example: Safe reward calculation**

```rust
pub fn calculate_rewards(ctx: Context<Claim>) -> Result<()> {
    let staker = &mut ctx.accounts.staker;
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    // Safe time difference
    let time_staked = (clock.unix_timestamp as u64)
        .checked_sub(staker.last_claim_time as u64)
        .ok_or(ErrorCode::InvalidTimestamp)?;

    // Safe multiplication (reward_rate * time * amount)
    let rewards = pool.reward_rate
        .checked_mul(time_staked)
        .ok_or(ErrorCode::Overflow)?
        .checked_mul(staker.amount)
        .ok_or(ErrorCode::Overflow)?
        .checked_div(PRECISION)
        .ok_or(ErrorCode::DivisionByZero)?;

    // Safe addition to pending rewards
    staker.pending_rewards = staker.pending_rewards
        .checked_add(rewards)
        .ok_or(ErrorCode::Overflow)?;

    staker.last_claim_time = clock.unix_timestamp;

    Ok(())
}
```

### Array Bounds Validation

Generated arrays have fixed maximum sizes. Always validate:

```rust
// Schema defines: inventory: [u64; 20]
pub const MAX_INVENTORY_SIZE: usize = 20;

pub fn add_item(ctx: Context<AddItem>, item_id: u64) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Validate capacity BEFORE modification
    require!(
        (player.inventory_count as usize) < MAX_INVENTORY_SIZE,
        ErrorCode::InventoryFull
    );

    // Validate item_id is valid
    require!(item_id > 0, ErrorCode::InvalidItemId);

    // Safe to add
    let slot = player.inventory_count as usize;
    player.inventory[slot] = item_id;
    player.inventory_count += 1;

    Ok(())
}

pub fn remove_item(ctx: Context<RemoveItem>, slot: usize) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Validate slot is within bounds
    require!(
        slot < (player.inventory_count as usize),
        ErrorCode::SlotOutOfBounds
    );

    // Shift items left to fill gap
    for i in slot..(player.inventory_count as usize - 1) {
        player.inventory[i] = player.inventory[i + 1];
    }

    // Clear last slot and decrement count
    player.inventory[player.inventory_count as usize - 1] = 0;
    player.inventory_count -= 1;

    Ok(())
}
```

### Option Field Validation

Handle `Option` fields safely:

```rust
// ✅ Pattern 1: require! with is_some/is_none
pub fn join_guild(ctx: Context<JoinGuild>) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Must not already be in a guild
    require!(player.guild.is_none(), ErrorCode::AlreadyInGuild);

    player.guild = Some(ctx.accounts.guild.key());
    Ok(())
}

// ✅ Pattern 2: if let for conditional logic
pub fn guild_action(ctx: Context<GuildAction>) -> Result<()> {
    let player = &ctx.accounts.player;

    if let Some(guild_key) = player.guild {
        // Player is in a guild, validate it matches
        require!(
            guild_key == ctx.accounts.guild.key(),
            ErrorCode::WrongGuild
        );
        // ... guild action
    } else {
        return Err(ErrorCode::NotInGuild.into());
    }

    Ok(())
}

// ✅ Pattern 3: ok_or for conversion to Result
pub fn get_guild(player: &Player) -> Result<Pubkey> {
    player.guild.ok_or(ErrorCode::NotInGuild.into())
}

// ✅ Pattern 4: unwrap_or for default values
pub fn get_guild_or_default(player: &Player) -> Pubkey {
    player.guild.unwrap_or(Pubkey::default())
}

// ✅ Pattern 5: map for transformations
pub fn get_guild_string(player: &Player) -> String {
    player.guild
        .map(|g| g.to_string())
        .unwrap_or_else(|| "No guild".to_string())
}
```

### Enum State Validation

Validate enum states before operations:

```rust
// Generated enum:
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum GameState {
    Lobby { max_players: u8 },
    Playing { current_turn: u8 },
    Finished { winner: Pubkey },
    Cancelled,
}

// ✅ Pattern 1: matches! macro for simple checks
pub fn start_game(ctx: Context<StartGame>) -> Result<()> {
    let game = &mut ctx.accounts.game;

    require!(
        matches!(game.state, GameState::Lobby { .. }),
        ErrorCode::GameNotInLobby
    );

    game.state = GameState::Playing { current_turn: 0 };
    Ok(())
}

// ✅ Pattern 2: match for extracting values
pub fn take_turn(ctx: Context<TakeTurn>) -> Result<()> {
    let game = &mut ctx.accounts.game;

    let current_turn = match &game.state {
        GameState::Playing { current_turn } => *current_turn,
        GameState::Lobby { .. } => return Err(ErrorCode::GameNotStarted.into()),
        GameState::Finished { .. } => return Err(ErrorCode::GameAlreadyFinished.into()),
        GameState::Cancelled => return Err(ErrorCode::GameCancelled.into()),
    };

    // ... turn logic

    game.state = GameState::Playing {
        current_turn: current_turn + 1,
    };

    Ok(())
}

// ✅ Pattern 3: State machine validation
impl GameState {
    pub fn can_transition_to(&self, new_state: &GameState) -> bool {
        match (self, new_state) {
            (GameState::Lobby { .. }, GameState::Playing { .. }) => true,
            (GameState::Lobby { .. }, GameState::Cancelled) => true,
            (GameState::Playing { .. }, GameState::Finished { .. }) => true,
            (GameState::Playing { .. }, GameState::Cancelled) => true,
            _ => false,
        }
    }
}

pub fn transition_state(
    game: &mut Game,
    new_state: GameState,
) -> Result<()> {
    require!(
        game.state.can_transition_to(&new_state),
        ErrorCode::InvalidStateTransition
    );

    game.state = new_state;
    Ok(())
}
```

### String Validation

Validate string inputs:

```rust
pub const MAX_NAME_LENGTH: usize = 32;
pub const MIN_NAME_LENGTH: usize = 3;

pub fn set_name(ctx: Context<SetName>, name: String) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Length validation
    require!(
        name.len() >= MIN_NAME_LENGTH,
        ErrorCode::NameTooShort
    );
    require!(
        name.len() <= MAX_NAME_LENGTH,
        ErrorCode::NameTooLong
    );

    // Character validation (alphanumeric + underscore)
    require!(
        name.chars().all(|c| c.is_alphanumeric() || c == '_'),
        ErrorCode::InvalidNameCharacters
    );

    // No leading/trailing whitespace
    require!(
        name.trim() == name,
        ErrorCode::NameHasWhitespace
    );

    player.name = name;
    Ok(())
}
```

---

## Custom Error Types

### Designing Error Enums

Create descriptive error types with `#[error_code]`:

```rust
use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    // === Authentication Errors (1xx) ===
    #[msg("Unauthorized: caller is not the owner")]
    Unauthorized,

    #[msg("Invalid signer: expected {0}")]
    InvalidSigner(String),

    // === Validation Errors (2xx) ===
    #[msg("Amount must be greater than zero")]
    ZeroAmount,

    #[msg("Insufficient balance: have {0}, need {1}")]
    InsufficientBalance(u64, u64),

    #[msg("Value {0} exceeds maximum {1}")]
    ExceedsMaximum(u64, u64),

    #[msg("Value {0} below minimum {1}")]
    BelowMinimum(u64, u64),

    // === State Errors (3xx) ===
    #[msg("Invalid state: expected {0}, got {1}")]
    InvalidState(String, String),

    #[msg("Cannot transition from {0} to {1}")]
    InvalidStateTransition(String, String),

    #[msg("Account is already initialized")]
    AlreadyInitialized,

    #[msg("Account is not initialized")]
    NotInitialized,

    // === Capacity Errors (4xx) ===
    #[msg("Inventory full: max {0} items")]
    InventoryFull(u8),

    #[msg("Index {0} out of bounds (max {1})")]
    IndexOutOfBounds(usize, usize),

    #[msg("String exceeds max length of {0}")]
    StringTooLong(usize),

    // === Arithmetic Errors (5xx) ===
    #[msg("Arithmetic overflow")]
    Overflow,

    #[msg("Arithmetic underflow")]
    Underflow,

    #[msg("Division by zero")]
    DivisionByZero,

    // === Time Errors (6xx) ===
    #[msg("Action not yet available: wait until {0}")]
    TooEarly(i64),

    #[msg("Deadline passed: was {0}")]
    TooLate(i64),

    #[msg("Invalid timestamp: {0}")]
    InvalidTimestamp(i64),

    // === Account Errors (7xx) ===
    #[msg("Account not found")]
    AccountNotFound,

    #[msg("Invalid account owner")]
    InvalidOwner,

    #[msg("PDA derivation failed")]
    InvalidPDA,
}
```

### Error Message Best Practices

**DO:**
```rust
// ✅ Include relevant values
#[msg("Insufficient balance: have {0} lamports, need {1} lamports")]
InsufficientBalance(u64, u64),

// ✅ Be specific about what failed
#[msg("Player level {0} below required {1} for this item")]
LevelTooLow(u16, u16),

// ✅ Suggest the fix when possible
#[msg("Pool is paused. Use unpause() to resume operations")]
PoolPaused,
```

**DON'T:**
```rust
// ❌ Vague messages
#[msg("Error")]
GenericError,

// ❌ Missing context
#[msg("Invalid")]
Invalid,

// ❌ Too technical for users
#[msg("Deserialization failed at offset 0x2f with discriminant 0x04")]
DeserializationError,
```

### Error Context Pattern

Add context to errors for debugging:

```rust
// Define error context struct
#[derive(Debug)]
pub struct ErrorContext {
    pub operation: &'static str,
    pub account: Option<Pubkey>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

// Use in handlers
pub fn transfer_with_context(
    ctx: Context<Transfer>,
    amount: u64,
) -> Result<()> {
    let sender = &ctx.accounts.sender;

    if sender.balance < amount {
        msg!(
            "Transfer failed: sender {} has {} but needs {}",
            sender.key(),
            sender.balance,
            amount
        );
        return Err(GameError::InsufficientBalance(
            sender.balance,
            amount
        ).into());
    }

    // ... transfer logic
    Ok(())
}
```

### Mapping to HTTP-like Codes

Organize errors by category for client handling:

```rust
impl GameError {
    pub fn error_code(&self) -> u32 {
        match self {
            // 400s - Client errors (bad input)
            GameError::ZeroAmount => 400,
            GameError::InsufficientBalance(_, _) => 400,
            GameError::StringTooLong(_) => 400,

            // 401 - Authentication
            GameError::Unauthorized => 401,
            GameError::InvalidSigner(_) => 401,

            // 403 - Forbidden (valid request, not allowed)
            GameError::InvalidState(_, _) => 403,
            GameError::InvalidStateTransition(_, _) => 403,

            // 404 - Not found
            GameError::AccountNotFound => 404,

            // 409 - Conflict
            GameError::AlreadyInitialized => 409,

            // 422 - Unprocessable (validation failed)
            GameError::IndexOutOfBounds(_, _) => 422,
            GameError::InventoryFull(_) => 422,

            // 500s - Internal errors
            GameError::Overflow => 500,
            GameError::DivisionByZero => 500,

            _ => 500,
        }
    }
}
```

---

## Anchor Constraint Validation

### Account Ownership

```rust
#[derive(Accounts)]
pub struct SecureTransfer<'info> {
    // Verify account owned by our program
    #[account(
        mut,
        owner = crate::ID,  // Must be owned by this program
    )]
    pub vault: Account<'info, Vault>,

    // Verify token account owned by Token program
    #[account(
        mut,
        token::mint = mint,
        token::authority = authority,
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub authority: Signer<'info>,
}
```

### has_one Constraints

Link accounts together:

```rust
#[derive(Accounts)]
pub struct UpdatePlayer<'info> {
    #[account(
        mut,
        has_one = owner @ GameError::Unauthorized,
        has_one = game @ GameError::WrongGame,
    )]
    pub player: Account<'info, Player>,

    pub game: Account<'info, Game>,
    pub owner: Signer<'info>,
}

// Player struct must have these fields:
#[account]
pub struct Player {
    pub owner: Pubkey,  // Must match owner signer
    pub game: Pubkey,   // Must match game account
    // ... other fields
}
```

### Custom Constraints

Add complex validation logic:

```rust
#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(
        mut,
        // Custom constraint with error
        constraint = game.player_count < game.max_players @ GameError::GameFull,
        // Multiple constraints
        constraint = matches!(game.state, GameState::Lobby { .. }) @ GameError::GameNotInLobby,
        constraint = !game.is_paused @ GameError::GamePaused,
    )]
    pub game: Account<'info, Game>,

    #[account(
        init,
        payer = payer,
        space = Player::LEN,
        seeds = [b"player", game.key().as_ref(), payer.key().as_ref()],
        bump,
    )]
    pub player: Account<'info, Player>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
```

### PDA Validation

Ensure PDAs derive correctly:

```rust
#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(
        mut,
        seeds = [b"staker", pool.key().as_ref(), owner.key().as_ref()],
        bump = staker.bump,
        has_one = owner,
        has_one = pool,
    )]
    pub staker: Account<'info, Staker>,

    #[account(
        mut,
        seeds = [b"pool", pool.authority.as_ref()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"vault", pool.key().as_ref()],
        bump = vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub owner: Signer<'info>,
}
```

### Signer Validation

Ensure proper authorization:

```rust
#[derive(Accounts)]
pub struct AdminAction<'info> {
    #[account(
        mut,
        has_one = admin @ GameError::Unauthorized,
    )]
    pub config: Account<'info, Config>,

    // Must be signer AND match config.admin
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct MultiSigAction<'info> {
    #[account(mut)]
    pub multisig: Account<'info, MultiSig>,

    // Require multiple signers
    pub signer1: Signer<'info>,
    pub signer2: Signer<'info>,

    // Validate signers are in allowed list
    #[account(
        constraint = multisig.signers.contains(&signer1.key()) @ GameError::InvalidSigner,
        constraint = multisig.signers.contains(&signer2.key()) @ GameError::InvalidSigner,
    )]
    pub _validation: Account<'info, MultiSig>,
}
```

### Close Account Safely

Handle account closure with validation:

```rust
#[derive(Accounts)]
pub struct CloseAccount<'info> {
    #[account(
        mut,
        close = destination,
        has_one = owner @ GameError::Unauthorized,
        // Only allow closing if empty
        constraint = account.balance == 0 @ GameError::AccountNotEmpty,
        constraint = account.items.is_empty() @ GameError::AccountNotEmpty,
    )]
    pub account: Account<'info, PlayerAccount>,

    #[account(mut)]
    pub destination: SystemAccount<'info>,

    pub owner: Signer<'info>,
}
```

---

## TypeScript Client Error Handling

### Deserialization Errors

Handle account deserialization safely:

```typescript
import { Connection, PublicKey } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';
import { Player, PlayerSchema } from './generated';

class DeserializationError extends Error {
  constructor(
    message: string,
    public accountKey: PublicKey,
    public cause?: Error
  ) {
    super(message);
    this.name = 'DeserializationError';
  }
}

async function fetchPlayer(
  connection: Connection,
  playerPDA: PublicKey
): Promise<Player> {
  // 1. Fetch account
  const accountInfo = await connection.getAccountInfo(playerPDA);

  // 2. Handle not found
  if (!accountInfo) {
    throw new DeserializationError(
      `Player account not found`,
      playerPDA
    );
  }

  // 3. Validate minimum size
  const DISCRIMINATOR_SIZE = 8;
  const MIN_PLAYER_SIZE = DISCRIMINATOR_SIZE + 32; // At least discriminator + pubkey

  if (accountInfo.data.length < MIN_PLAYER_SIZE) {
    throw new DeserializationError(
      `Account data too small: ${accountInfo.data.length} bytes`,
      playerPDA
    );
  }

  // 4. Deserialize with error handling
  try {
    const data = accountInfo.data.slice(DISCRIMINATOR_SIZE);
    return PlayerSchema.decode(data) as Player;
  } catch (error) {
    throw new DeserializationError(
      `Failed to deserialize player account`,
      playerPDA,
      error as Error
    );
  }
}
```

### Account Not Found Handling

```typescript
// Pattern 1: Return null for optional accounts
async function fetchPlayerOrNull(
  connection: Connection,
  playerPDA: PublicKey
): Promise<Player | null> {
  const accountInfo = await connection.getAccountInfo(playerPDA);
  if (!accountInfo) return null;

  const data = accountInfo.data.slice(8);
  return PlayerSchema.decode(data) as Player;
}

// Pattern 2: Throw descriptive error
async function fetchPlayerOrThrow(
  connection: Connection,
  playerPDA: PublicKey
): Promise<Player> {
  const player = await fetchPlayerOrNull(connection, playerPDA);

  if (!player) {
    throw new Error(
      `Player not found at ${playerPDA.toBase58()}. ` +
      `Please create a player account first.`
    );
  }

  return player;
}

// Pattern 3: Check existence before operation
async function ensurePlayerExists(
  connection: Connection,
  playerPDA: PublicKey
): Promise<boolean> {
  const accountInfo = await connection.getAccountInfo(playerPDA);
  return accountInfo !== null;
}

// Usage
async function performAction(playerPDA: PublicKey) {
  const exists = await ensurePlayerExists(connection, playerPDA);

  if (!exists) {
    // Offer to create account
    const shouldCreate = await promptUser('Create player account?');
    if (shouldCreate) {
      await createPlayer(playerPDA);
    } else {
      return;
    }
  }

  // Now safe to fetch
  const player = await fetchPlayerOrThrow(connection, playerPDA);
  // ... use player
}
```

### Transaction Error Handling

```typescript
import {
  Connection,
  Transaction,
  SendTransactionError,
  TransactionExpiredBlockheightExceededError,
} from '@solana/web3.js';
import { AnchorError } from '@coral-xyz/anchor';

// Custom error types
class TransactionError extends Error {
  constructor(
    message: string,
    public code?: number,
    public logs?: string[]
  ) {
    super(message);
    this.name = 'TransactionError';
  }
}

async function sendTransactionSafely(
  connection: Connection,
  transaction: Transaction,
  signers: Keypair[]
): Promise<string> {
  try {
    const signature = await connection.sendTransaction(
      transaction,
      signers,
      { skipPreflight: false }
    );

    // Wait for confirmation
    const confirmation = await connection.confirmTransaction(
      signature,
      'confirmed'
    );

    if (confirmation.value.err) {
      throw new TransactionError(
        `Transaction failed: ${JSON.stringify(confirmation.value.err)}`
      );
    }

    return signature;

  } catch (error) {
    // Handle specific error types
    if (error instanceof SendTransactionError) {
      const logs = error.logs || [];

      // Parse Anchor error from logs
      const anchorError = parseAnchorError(logs);
      if (anchorError) {
        throw new TransactionError(
          anchorError.message,
          anchorError.code,
          logs
        );
      }

      throw new TransactionError(
        'Transaction failed to send',
        undefined,
        logs
      );
    }

    if (error instanceof TransactionExpiredBlockheightExceededError) {
      throw new TransactionError(
        'Transaction expired. Please try again.',
        408 // Timeout
      );
    }

    throw error;
  }
}

// Parse Anchor error from transaction logs
function parseAnchorError(logs: string[]): { code: number; message: string } | null {
  for (const log of logs) {
    // Anchor error format: "Program log: AnchorError occurred. Error Code: {code}. Error Message: {msg}"
    const match = log.match(
      /AnchorError.*Error Code: (\w+).*Error Message: (.+)/
    );
    if (match) {
      return {
        code: parseInt(match[1]) || 0,
        message: match[2],
      };
    }
  }
  return null;
}
```

### u64 Precision Validation

```typescript
import { BN } from '@coral-xyz/anchor';

// Maximum safe integer in JavaScript
const MAX_SAFE_INTEGER = BigInt(Number.MAX_SAFE_INTEGER); // 2^53 - 1

class PrecisionError extends Error {
  constructor(value: bigint | BN) {
    super(
      `Value ${value.toString()} exceeds JavaScript safe integer limit. ` +
      `Use BN or BigInt for arithmetic operations.`
    );
    this.name = 'PrecisionError';
  }
}

// Validate before converting to number
function safeToNumber(value: BN | bigint): number {
  const bigValue = typeof value === 'bigint' ? value : BigInt(value.toString());

  if (bigValue > MAX_SAFE_INTEGER) {
    throw new PrecisionError(bigValue);
  }

  return Number(bigValue);
}

// Safe display formatting
function formatLargeNumber(value: BN | bigint | number): string {
  const bn = new BN(value.toString());

  // For display, always use string representation
  if (bn.gt(new BN(Number.MAX_SAFE_INTEGER.toString()))) {
    // Format with commas for readability
    return bn.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
  }

  return bn.toNumber().toLocaleString();
}

// Safe arithmetic
function safeAdd(a: BN, b: BN): BN {
  const result = a.add(b);

  // Check for overflow (shouldn't happen with BN, but good practice)
  if (result.lt(a) || result.lt(b)) {
    throw new Error('Arithmetic overflow');
  }

  return result;
}

// Usage with generated types
interface Player {
  balance: number; // u64 in schema
  experience: number;
}

function displayPlayerBalance(player: Player): string {
  // Treat as BN for safety
  const balance = new BN(player.balance);
  const sol = balance.div(new BN(1_000_000_000));
  const lamports = balance.mod(new BN(1_000_000_000));

  return `${sol.toString()}.${lamports.toString().padStart(9, '0')} SOL`;
}
```

### Enum Type Narrowing Errors

```typescript
type PlayerStatus =
  | { kind: 'Active' }
  | { kind: 'Resting'; until: number }
  | { kind: 'Banned'; reason: string; until: number }
  | { kind: 'Dead' };

// Exhaustive switch with error for unknown variants
function handleStatus(status: PlayerStatus): string {
  switch (status.kind) {
    case 'Active':
      return 'Ready to play';

    case 'Resting':
      const restEnd = new Date(status.until * 1000);
      return `Resting until ${restEnd.toLocaleString()}`;

    case 'Banned':
      return `Banned: ${status.reason}`;

    case 'Dead':
      return 'Dead - needs revival';

    default:
      // TypeScript will error if we miss a case
      const _exhaustive: never = status;
      throw new Error(`Unknown status: ${JSON.stringify(status)}`);
  }
}

// Type guard with validation
function isBanned(status: PlayerStatus): status is { kind: 'Banned'; reason: string; until: number } {
  return status.kind === 'Banned';
}

// Safe field access
function getBanReason(status: PlayerStatus): string | null {
  if (isBanned(status)) {
    return status.reason;
  }
  return null;
}
```

### React Error Boundaries

```typescript
import React, { Component, ReactNode } from 'react';

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error) => void;
}

class SolanaErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('Solana error caught:', error, info);
    this.props.onError?.(error);
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="error-container">
          <h2>Something went wrong</h2>
          <p>{this.state.error?.message}</p>
          <button onClick={() => this.setState({ hasError: false, error: null })}>
            Try Again
          </button>
        </div>
      );
    }

    return this.props.children;
  }
}

// Usage
function App() {
  return (
    <SolanaErrorBoundary
      fallback={<ErrorFallback />}
      onError={(error) => logToService(error)}
    >
      <GameComponent />
    </SolanaErrorBoundary>
  );
}
```

### Network Error Retry Pattern

```typescript
interface RetryOptions {
  maxRetries: number;
  baseDelay: number;
  maxDelay: number;
}

const DEFAULT_RETRY_OPTIONS: RetryOptions = {
  maxRetries: 3,
  baseDelay: 1000,
  maxDelay: 10000,
};

async function withRetry<T>(
  operation: () => Promise<T>,
  options: Partial<RetryOptions> = {}
): Promise<T> {
  const { maxRetries, baseDelay, maxDelay } = {
    ...DEFAULT_RETRY_OPTIONS,
    ...options,
  };

  let lastError: Error | null = null;

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await operation();
    } catch (error) {
      lastError = error as Error;

      // Don't retry on non-retryable errors
      if (!isRetryable(error)) {
        throw error;
      }

      // Calculate delay with exponential backoff
      const delay = Math.min(
        baseDelay * Math.pow(2, attempt),
        maxDelay
      );

      console.log(
        `Attempt ${attempt + 1} failed, retrying in ${delay}ms...`
      );

      await sleep(delay);
    }
  }

  throw lastError;
}

function isRetryable(error: unknown): boolean {
  // Network errors are retryable
  if (error instanceof TypeError && error.message.includes('fetch')) {
    return true;
  }

  // Rate limiting
  if (error instanceof Error && error.message.includes('429')) {
    return true;
  }

  // Timeout
  if (error instanceof Error && error.message.includes('timeout')) {
    return true;
  }

  return false;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Usage
const player = await withRetry(
  () => fetchPlayer(connection, playerPDA),
  { maxRetries: 5, baseDelay: 500 }
);
```

---

## Testing Error Paths

### Unit Testing Errors in Rust

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insufficient_balance_error() {
        let mut player = Player {
            balance: 100,
            ..Default::default()
        };

        // Attempt to withdraw more than balance
        let result = withdraw(&mut player, 200);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GameError::InsufficientBalance(100, 200).into()
        );
    }

    #[test]
    fn test_inventory_full_error() {
        let mut player = Player {
            inventory_count: 20, // Max capacity
            ..Default::default()
        };

        let result = add_item(&mut player, 42);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            GameError::InventoryFull(20).into()
        );
    }

    #[test]
    fn test_state_transition_error() {
        let game = Game {
            state: GameState::Finished { winner: Pubkey::default() },
            ..Default::default()
        };

        let result = start_game(&game);

        assert!(result.is_err());
        // Game can't start if already finished
    }

    #[test]
    fn test_overflow_prevention() {
        let mut account = Account {
            balance: u64::MAX - 10,
        };

        let result = deposit(&mut account, 100);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), GameError::Overflow.into());
    }

    #[test]
    fn test_invalid_name_errors() {
        // Too short
        assert!(validate_name("ab").is_err());

        // Too long
        assert!(validate_name(&"a".repeat(33)).is_err());

        // Invalid characters
        assert!(validate_name("name with spaces").is_err());
        assert!(validate_name("name@special").is_err());

        // Valid
        assert!(validate_name("valid_name123").is_ok());
    }
}
```

### Integration Testing with Anchor

```rust
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::InstructionError;
use anchor_lang::error::ErrorCode as AnchorErrorCode;

#[tokio::test]
async fn test_unauthorized_access() {
    let mut context = setup_test_context().await;

    // Create game with owner A
    let owner_a = Keypair::new();
    let game = create_game(&mut context, &owner_a).await.unwrap();

    // Try to modify with owner B (should fail)
    let owner_b = Keypair::new();
    let result = update_game(&mut context, &game, &owner_b).await;

    // Assert specific error
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(
        err.unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(GameError::Unauthorized as u32)
        )
    );
}

#[tokio::test]
async fn test_constraint_violation() {
    let mut context = setup_test_context().await;

    let game = create_game(&mut context, &context.payer).await.unwrap();

    // Fill game to capacity
    for i in 0..MAX_PLAYERS {
        join_game(&mut context, &game, i).await.unwrap();
    }

    // Try to add one more (should fail)
    let result = join_game(&mut context, &game, MAX_PLAYERS).await;

    assert!(result.is_err());
    // Check for GameFull error
}
```

### TypeScript Error Testing

```typescript
import { describe, it, expect } from 'vitest';
import { PublicKey } from '@solana/web3.js';

describe('Error Handling', () => {
  describe('fetchPlayer', () => {
    it('should throw DeserializationError for non-existent account', async () => {
      const fakePDA = PublicKey.unique();

      await expect(
        fetchPlayer(connection, fakePDA)
      ).rejects.toThrow(DeserializationError);
    });

    it('should throw DeserializationError for corrupted data', async () => {
      // Create account with invalid data
      const invalidAccount = await createAccountWithData(
        Buffer.from([0, 0, 0]) // Too short
      );

      await expect(
        fetchPlayer(connection, invalidAccount)
      ).rejects.toThrow('Account data too small');
    });
  });

  describe('Transaction Errors', () => {
    it('should handle insufficient balance error', async () => {
      const player = await createPlayer(100); // 100 lamports

      await expect(
        transferFunds(player, 1000) // More than balance
      ).rejects.toThrow(/Insufficient balance/);
    });

    it('should handle expired transaction', async () => {
      const tx = await buildTransaction();

      // Wait for blockhash to expire
      await sleep(60000);

      await expect(
        sendTransaction(tx)
      ).rejects.toThrow('Transaction expired');
    });
  });

  describe('Type Safety', () => {
    it('should narrow enum types correctly', () => {
      const banned: PlayerStatus = {
        kind: 'Banned',
        reason: 'Cheating',
        until: Date.now() / 1000 + 3600,
      };

      expect(handleStatus(banned)).toContain('Banned');
      expect(getBanReason(banned)).toBe('Cheating');
    });

    it('should return null for non-banned status', () => {
      const active: PlayerStatus = { kind: 'Active' };

      expect(getBanReason(active)).toBeNull();
    });
  });
});
```

---

## Common Errors & Solutions

### Error Reference Table

| Error | Cause | Solution |
|-------|-------|----------|
| `AccountNotFound` | PDA doesn't exist | Create account first, check derivation |
| `InsufficientBalance` | Not enough funds | Check balance before transfer |
| `Overflow` | Arithmetic exceeded u64 | Use `checked_add()` |
| `Underflow` | Subtraction went negative | Validate amount <= balance |
| `Unauthorized` | Wrong signer | Ensure correct wallet signs |
| `InvalidPDA` | Seeds don't match | Verify seed order and values |
| `InventoryFull` | Array at capacity | Check length before push |
| `InvalidState` | Wrong enum variant | Check state before action |
| `DeserializationFailed` | Data format mismatch | Skip discriminator, check schema |
| `AccountOwnedByWrongProgram` | Wrong program owns account | Check account derivation |

### Debugging Techniques

**1. Enable Program Logs**

```rust
pub fn debug_handler(ctx: Context<Debug>) -> Result<()> {
    let player = &ctx.accounts.player;

    msg!("=== Debug Info ===");
    msg!("Player key: {}", player.key());
    msg!("Balance: {}", player.balance);
    msg!("Inventory count: {}", player.inventory_count);
    msg!("Status: {:?}", player.status);

    Ok(())
}
```

**2. Client-Side Log Parsing**

```typescript
async function debugTransaction(signature: string) {
  const tx = await connection.getTransaction(signature, {
    commitment: 'confirmed',
  });

  console.log('=== Transaction Logs ===');
  tx?.meta?.logMessages?.forEach((log, i) => {
    console.log(`${i}: ${log}`);
  });

  if (tx?.meta?.err) {
    console.log('Error:', JSON.stringify(tx.meta.err, null, 2));
  }
}
```

**3. Simulate Before Send**

```typescript
async function simulateAndSend(tx: Transaction): Promise<string> {
  // Simulate first
  const simulation = await connection.simulateTransaction(tx);

  if (simulation.value.err) {
    console.error('Simulation failed:', simulation.value.err);
    console.error('Logs:', simulation.value.logs);
    throw new Error(`Simulation failed: ${JSON.stringify(simulation.value.err)}`);
  }

  // Only send if simulation passed
  return await connection.sendTransaction(tx, signers);
}
```

---

## Anti-Patterns & Pitfalls

### What NOT to Do

```rust
// ❌ DON'T: Unwrap without checking
pub fn bad_unwrap(player: &Player) -> Pubkey {
    player.guild.unwrap()  // Panics if None!
}

// ❌ DON'T: Use raw arithmetic
pub fn bad_math(account: &mut Account, amount: u64) {
    account.balance += amount;  // Can overflow!
}

// ❌ DON'T: Ignore capacity limits
pub fn bad_push(player: &mut Player, item: u64) {
    player.inventory.push(item);  // May exceed account size!
}

// ❌ DON'T: Change state before validation
pub fn bad_order(ctx: Context<Transfer>, amount: u64) -> Result<()> {
    let sender = &mut ctx.accounts.sender;

    // State changed before validation!
    sender.balance -= amount;

    // Now we check - too late if this fails!
    require!(sender.balance >= 0, ErrorCode::Underflow);

    Ok(())
}

// ❌ DON'T: Vague error messages
#[error_code]
pub enum BadErrors {
    #[msg("Error")]
    Error,

    #[msg("Failed")]
    Failed,
}
```

### What TO Do

```rust
// ✅ DO: Safe Option handling
pub fn good_option(player: &Player) -> Result<Pubkey> {
    player.guild.ok_or(GameError::NotInGuild.into())
}

// ✅ DO: Checked arithmetic
pub fn good_math(account: &mut Account, amount: u64) -> Result<()> {
    account.balance = account.balance
        .checked_add(amount)
        .ok_or(GameError::Overflow)?;
    Ok(())
}

// ✅ DO: Validate capacity
pub fn good_push(player: &mut Player, item: u64) -> Result<()> {
    require!(
        player.inventory.len() < MAX_ITEMS,
        GameError::InventoryFull
    );
    player.inventory.push(item);
    Ok(())
}

// ✅ DO: Validate before state change
pub fn good_order(ctx: Context<Transfer>, amount: u64) -> Result<()> {
    let sender = &mut ctx.accounts.sender;

    // Validate first
    require!(
        sender.balance >= amount,
        GameError::InsufficientBalance
    );

    // Safe to modify
    sender.balance = sender.balance
        .checked_sub(amount)
        .ok_or(GameError::Underflow)?;

    Ok(())
}

// ✅ DO: Descriptive error messages
#[error_code]
pub enum GoodErrors {
    #[msg("Insufficient balance: have {0}, need {1}")]
    InsufficientBalance(u64, u64),

    #[msg("Transfer failed: sender and recipient cannot be the same")]
    SelfTransfer,
}
```

### Performance Considerations

```rust
// ✅ DO: Validate early (fail fast)
pub fn efficient_handler(ctx: Context<Action>, amount: u64) -> Result<()> {
    // Cheap validations first
    require!(amount > 0, GameError::ZeroAmount);

    let account = &ctx.accounts.account;

    // Check before expensive operations
    require!(account.balance >= amount, GameError::InsufficientBalance);

    // Now do expensive work (CPI, complex calculations)
    // ...

    Ok(())
}

// ❌ DON'T: Expensive operations before validation
pub fn inefficient_handler(ctx: Context<Action>, amount: u64) -> Result<()> {
    // Expensive CPI first
    do_expensive_operation()?;

    // Validation after (wasted compute if this fails!)
    require!(amount > 0, GameError::ZeroAmount);

    Ok(())
}
```

---

## Related Guides

After mastering error handling, explore these guides:

**Foundation Guides:**
- [Usage Examples for Generated Code](/guides/usage-examples) - Practical usage patterns
- [LUMOS + Anchor Integration](/guides/anchor-integration) - Schema-first development
- [LUMOS + web3.js Integration](/guides/web3js-integration) - Client patterns

**Use Case Guides:**
- [DeFi Protocols](/guides/use-cases/defi) - Financial error handling
- [Gaming Projects](/guides/use-cases/gaming) - State machine patterns
- [NFT Marketplaces](/guides/use-cases/nft) - Transfer validation

**Advanced Guides:**
- [LUMOS + Solv/Jito Integration](/guides/solv-jito-integration) - Precision handling
- [Migration: Anchor → LUMOS](/guides/migration-anchor) - Error migration

---

## Summary

| Category | Key Pattern | Example |
|----------|-------------|---------|
| Arithmetic | Use checked_* | `checked_add().ok_or()?` |
| Arrays | Validate capacity | `require!(len < MAX)` |
| Options | Use ok_or/if let | `guild.ok_or(Error)?` |
| Enums | matches! + match | `matches!(state, Active)` |
| Constraints | has_one + constraint | `has_one = owner` |
| TypeScript | Type narrowing | `if (status.kind === 'X')` |
| Testing | Test error paths | `assert!(result.is_err())` |

**Key Takeaways:**
1. **Validate before modifying state** - No rollback on Solana
2. **Use checked arithmetic** - Never raw +/-/*
3. **Design clear error types** - Include context values
4. **Handle all Option/Enum cases** - No unwrap without check
5. **Test error paths** - Coverage for failure scenarios
6. **Fail fast** - Cheap validations first

Build robust programs by treating errors as first-class citizens in your design!
