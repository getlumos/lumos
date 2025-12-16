# Usage Examples for Generated Code

This guide shows you HOW to actually USE the Rust and TypeScript code that LUMOS generates. While other guides cover schema design and generation, this guide focuses on practical usage patterns in your Anchor handlers and frontend applications.

```
┌─────────────────────────────────────────────────────────────────┐
│                 From Generated Code to Working App              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  schema.lumos ──► lumos generate ──► generated.rs + generated.ts│
│                                              │                  │
│                          ┌───────────────────┴──────────────┐   │
│                          ▼                                  ▼   │
│                   ┌─────────────┐                  ┌───────────┐│
│                   │ Anchor      │                  │ Frontend  ││
│                   │ Handlers    │                  │ Client    ││
│                   │             │                  │           ││
│                   │ - Access    │                  │ - Fetch   ││
│                   │ - Modify    │                  │ - Decode  ││
│                   │ - Validate  │                  │ - Display ││
│                   └─────────────┘                  └───────────┘│
│                                                                 │
│  This guide covers the "how to use" part ─────────────────────►│
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Using Generated Rust in Anchor Handlers](#using-generated-rust-in-anchor-handlers)
3. [Using Generated TypeScript in Frontend](#using-generated-typescript-in-frontend)
4. [Testing Generated Code](#testing-generated-code)
5. [Common Patterns & Anti-Patterns](#common-patterns--anti-patterns)
6. [Complete Working Example](#complete-working-example)
7. [Troubleshooting](#troubleshooting)
8. [Related Guides](#related-guides)

---

## Quick Reference

### What LUMOS Generates

When you run `lumos generate schema.lumos`, you get:

**Rust (`generated.rs`):**
```rust
// Struct with Anchor #[account] macro
#[account]
pub struct PlayerAccount {
    pub wallet: Pubkey,
    pub level: u16,
    pub experience: u64,
    pub inventory: Vec<u64>,
    pub guild: Option<Pubkey>,
    pub status: PlayerStatus,
}

// Automatic space calculation
impl PlayerAccount {
    pub const LEN: usize = 8 + 32 + 2 + 8 + (4 + 10 * 8) + 33 + 1;
}

// Enums with Borsh serialization
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PlayerStatus {
    Active,
    Inactive,
    Banned { reason: String, until: i64 },
}
```

**TypeScript (`generated.ts`):**
```typescript
// Interface matching Rust struct
export interface PlayerAccount {
  wallet: PublicKey;
  level: number;
  experience: number;
  inventory: number[];
  guild: PublicKey | null;
  status: PlayerStatus;
}

// Borsh schema for deserialization
export const PlayerAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.u16('level'),
  borsh.u64('experience'),
  borsh.vec(borsh.u64(), 'inventory'),
  borsh.option(borsh.publicKey(), 'guild'),
  PlayerStatusSchema,
]);

// Discriminated union for enums
export type PlayerStatus =
  | { kind: 'Active' }
  | { kind: 'Inactive' }
  | { kind: 'Banned'; reason: string; until: number };
```

### File Structure After Generation

```
your-project/
├── schemas/
│   └── schema.lumos          # Your source schema
├── programs/your-program/
│   └── src/
│       ├── lib.rs            # Your program (imports generated)
│       └── generated.rs      # Generated Rust code
└── app/
    └── src/
        └── generated.ts      # Generated TypeScript
```

### The Basic Workflow

```bash
# 1. Write schema
echo '#[solana]
#[account]
struct Player { level: u16 }' > schema.lumos

# 2. Generate code
lumos generate schema.lumos

# 3. Use in Rust handlers (see section 2)
# 4. Use in TypeScript frontend (see section 3)
```

---

## Using Generated Rust in Anchor Handlers

### Accessing Account Fields

Generated structs are regular Rust structs - access fields directly:

```rust
use crate::generated::*;

#[derive(Accounts)]
pub struct GetPlayerInfo<'info> {
    pub player: Account<'info, PlayerAccount>,
}

pub fn get_player_info(ctx: Context<GetPlayerInfo>) -> Result<()> {
    let player = &ctx.accounts.player;

    // Direct field access
    msg!("Wallet: {}", player.wallet);
    msg!("Level: {}", player.level);
    msg!("Experience: {}", player.experience);

    // Access array length
    msg!("Inventory items: {}", player.inventory.len());

    // Access Option fields
    match &player.guild {
        Some(guild_key) => msg!("Guild: {}", guild_key),
        None => msg!("No guild"),
    }

    Ok(())
}
```

### Modifying Account State

Use mutable references to modify state:

```rust
#[derive(Accounts)]
pub struct LevelUp<'info> {
    #[account(mut)]
    pub player: Account<'info, PlayerAccount>,
}

pub fn level_up(ctx: Context<LevelUp>, xp_gained: u64) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Modify primitive fields
    player.experience = player.experience
        .checked_add(xp_gained)
        .ok_or(ErrorCode::Overflow)?;

    // Level up logic
    let xp_for_next_level = (player.level as u64 + 1) * 1000;
    if player.experience >= xp_for_next_level {
        player.level = player.level
            .checked_add(1)
            .ok_or(ErrorCode::Overflow)?;
        msg!("Level up! Now level {}", player.level);
    }

    Ok(())
}
```

### Working with Arrays (Vec)

Generated `Vec` fields support all standard Rust operations:

```rust
#[derive(Accounts)]
pub struct ManageInventory<'info> {
    #[account(mut)]
    pub player: Account<'info, PlayerAccount>,
}

pub fn add_item(ctx: Context<ManageInventory>, item_id: u64) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Check capacity (important for account size!)
    require!(
        player.inventory.len() < 10,  // Max 10 items defined in schema
        ErrorCode::InventoryFull
    );

    // Add item
    player.inventory.push(item_id);
    msg!("Added item {}. Inventory size: {}", item_id, player.inventory.len());

    Ok(())
}

pub fn remove_item(ctx: Context<ManageInventory>, item_id: u64) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Find and remove item
    if let Some(index) = player.inventory.iter().position(|&id| id == item_id) {
        player.inventory.remove(index);
        msg!("Removed item {}", item_id);
        Ok(())
    } else {
        Err(ErrorCode::ItemNotFound.into())
    }
}

pub fn has_item(ctx: Context<ManageInventory>, item_id: u64) -> Result<bool> {
    let player = &ctx.accounts.player;

    // Check if item exists
    Ok(player.inventory.contains(&item_id))
}

pub fn get_item_at(ctx: Context<ManageInventory>, index: usize) -> Result<u64> {
    let player = &ctx.accounts.player;

    // Safe access with bounds checking
    player.inventory
        .get(index)
        .copied()
        .ok_or(ErrorCode::IndexOutOfBounds.into())
}

pub fn clear_inventory(ctx: Context<ManageInventory>) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Clear all items
    player.inventory.clear();
    msg!("Inventory cleared");

    Ok(())
}
```

### Working with Option Fields

Handle optional fields safely:

```rust
pub fn join_guild(ctx: Context<JoinGuild>) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let guild = &ctx.accounts.guild;

    // Check if already in a guild
    require!(player.guild.is_none(), ErrorCode::AlreadyInGuild);

    // Set the Option field
    player.guild = Some(guild.key());
    msg!("Joined guild {}", guild.key());

    Ok(())
}

pub fn leave_guild(ctx: Context<LeaveGuild>) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Check if in a guild
    require!(player.guild.is_some(), ErrorCode::NotInGuild);

    // Clear the Option field
    player.guild = None;
    msg!("Left guild");

    Ok(())
}

pub fn get_guild_or_default(ctx: Context<GetGuild>) -> Result<Pubkey> {
    let player = &ctx.accounts.player;

    // Use unwrap_or for default value
    Ok(player.guild.unwrap_or(Pubkey::default()))
}

pub fn transfer_if_in_guild(ctx: Context<Transfer>, amount: u64) -> Result<()> {
    let player = &ctx.accounts.player;

    // Use if let for conditional logic
    if let Some(guild_key) = player.guild {
        msg!("Transferring {} to guild {}", amount, guild_key);
        // ... transfer logic
    } else {
        msg!("No guild to transfer to");
    }

    Ok(())
}
```

### Enum Pattern Matching

Generated enums use Rust's powerful pattern matching:

```rust
// Given this generated enum:
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PlayerStatus {
    Active,
    Inactive,
    Banned { reason: String, until: i64 },
}

// Match on enum variants
pub fn check_status(ctx: Context<CheckStatus>) -> Result<()> {
    let player = &ctx.accounts.player;

    match &player.status {
        PlayerStatus::Active => {
            msg!("Player is active");
        }
        PlayerStatus::Inactive => {
            msg!("Player is inactive");
        }
        PlayerStatus::Banned { reason, until } => {
            let now = Clock::get()?.unix_timestamp;
            if now < *until {
                msg!("Player banned for: {}. Until: {}", reason, until);
                return Err(ErrorCode::PlayerBanned.into());
            } else {
                msg!("Ban expired, reactivating");
            }
        }
    }

    Ok(())
}

// Change enum state
pub fn ban_player(
    ctx: Context<BanPlayer>,
    reason: String,
    duration_seconds: i64,
) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let now = Clock::get()?.unix_timestamp;

    player.status = PlayerStatus::Banned {
        reason,
        until: now + duration_seconds,
    };

    Ok(())
}

pub fn activate_player(ctx: Context<ActivatePlayer>) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Change to Active variant
    player.status = PlayerStatus::Active;

    Ok(())
}

// Check specific variant
pub fn is_active(player: &PlayerAccount) -> bool {
    matches!(player.status, PlayerStatus::Active)
}

pub fn is_banned(player: &PlayerAccount) -> bool {
    matches!(player.status, PlayerStatus::Banned { .. })
}
```

### Complete Handler Example

Here's a complete handler showing all patterns together:

```rust
use anchor_lang::prelude::*;
use crate::generated::{PlayerAccount, PlayerStatus, GameItem};

#[derive(Accounts)]
pub struct EquipItem<'info> {
    #[account(mut)]
    pub player: Account<'info, PlayerAccount>,

    pub item: Account<'info, GameItem>,

    #[account(
        constraint = authority.key() == player.wallet @ ErrorCode::Unauthorized
    )]
    pub authority: Signer<'info>,
}

pub fn equip_item(ctx: Context<EquipItem>, slot: u8) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let item = &ctx.accounts.item;

    // 1. Check player status (enum matching)
    match &player.status {
        PlayerStatus::Banned { reason, .. } => {
            msg!("Cannot equip: player banned for {}", reason);
            return Err(ErrorCode::PlayerBanned.into());
        }
        PlayerStatus::Inactive => {
            msg!("Reactivating inactive player");
            player.status = PlayerStatus::Active;
        }
        PlayerStatus::Active => {}
    }

    // 2. Check level requirement (primitive field)
    require!(
        player.level >= item.required_level,
        ErrorCode::LevelTooLow
    );

    // 3. Check if item is in inventory (array operation)
    let item_id = item.key().to_bytes()[0] as u64; // Simplified ID
    require!(
        player.inventory.contains(&item_id),
        ErrorCode::ItemNotOwned
    );

    // 4. Remove from inventory
    if let Some(index) = player.inventory.iter().position(|&id| id == item_id) {
        player.inventory.remove(index);
    }

    // 5. Add to equipped slot (Option field would be used here)
    // player.equipped_items[slot as usize] = Some(item.key());

    // 6. Grant XP for equipping (modify primitive)
    player.experience = player.experience.saturating_add(10);

    msg!(
        "Player {} equipped item. XP: {}, Inventory: {} items",
        player.wallet,
        player.experience,
        player.inventory.len()
    );

    Ok(())
}
```

---

## Using Generated TypeScript in Frontend

### Fetching and Deserializing Accounts

The core pattern for using generated TypeScript types:

```typescript
import { Connection, PublicKey } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';
import { PlayerAccount, PlayerAccountSchema } from './generated';

// IMPORTANT: Anchor adds an 8-byte discriminator prefix
const ANCHOR_DISCRIMINATOR_SIZE = 8;

async function fetchPlayerAccount(
  connection: Connection,
  playerPDA: PublicKey
): Promise<PlayerAccount> {
  // 1. Fetch raw account data
  const accountInfo = await connection.getAccountInfo(playerPDA);

  if (!accountInfo) {
    throw new Error('Player account not found');
  }

  // 2. Skip the 8-byte Anchor discriminator
  const data = accountInfo.data.slice(ANCHOR_DISCRIMINATOR_SIZE);

  // 3. Deserialize using generated schema
  const player = PlayerAccountSchema.decode(data) as PlayerAccount;

  return player;
}

// Usage
const player = await fetchPlayerAccount(connection, playerPDA);
console.log(`Level: ${player.level}`);
console.log(`XP: ${player.experience}`);
```

### Type-Safe Field Access

Generated interfaces give you full TypeScript type safety:

```typescript
import { PlayerAccount, PlayerStatus } from './generated';

function displayPlayer(player: PlayerAccount) {
  // Primitive fields - direct access
  console.log(`Wallet: ${player.wallet.toBase58()}`);
  console.log(`Level: ${player.level}`);
  console.log(`Experience: ${player.experience}`);

  // Array fields - full array methods available
  console.log(`Inventory size: ${player.inventory.length}`);
  player.inventory.forEach((itemId, index) => {
    console.log(`  Slot ${index}: Item #${itemId}`);
  });

  // Check if inventory has specific item
  const hasItem42 = player.inventory.includes(42);

  // Option fields - null check required
  if (player.guild) {
    console.log(`Guild: ${player.guild.toBase58()}`);
  } else {
    console.log('No guild');
  }
}
```

### Enum Type Narrowing (Discriminated Unions)

Generated enums use TypeScript discriminated unions with a `kind` field:

```typescript
// Generated type:
type PlayerStatus =
  | { kind: 'Active' }
  | { kind: 'Inactive' }
  | { kind: 'Banned'; reason: string; until: number };

function handlePlayerStatus(status: PlayerStatus): string {
  // TypeScript narrows the type based on 'kind'
  switch (status.kind) {
    case 'Active':
      return 'Player is active and ready to play';

    case 'Inactive':
      return 'Player is inactive';

    case 'Banned':
      // TypeScript knows 'reason' and 'until' exist here
      const banEnd = new Date(status.until * 1000);
      return `Banned for: ${status.reason}. Until: ${banEnd.toISOString()}`;
  }
}

// Type guards for specific variants
function isActive(status: PlayerStatus): status is { kind: 'Active' } {
  return status.kind === 'Active';
}

function isBanned(status: PlayerStatus): status is { kind: 'Banned'; reason: string; until: number } {
  return status.kind === 'Banned';
}

// Usage with type guards
function canPlay(player: PlayerAccount): boolean {
  if (isBanned(player.status)) {
    const now = Date.now() / 1000;
    return now > player.status.until; // TypeScript knows 'until' exists
  }
  return isActive(player.status);
}
```

### Fetching Multiple Related Accounts

Real apps often need to fetch related accounts:

```typescript
import { Connection, PublicKey } from '@solana/web3.js';
import {
  PlayerAccount,
  PlayerAccountSchema,
  GameItem,
  GameItemSchema,
  Guild,
  GuildSchema,
} from './generated';

interface PlayerWithRelations {
  player: PlayerAccount;
  inventoryItems: GameItem[];
  guild: Guild | null;
}

async function fetchPlayerWithRelations(
  connection: Connection,
  playerPDA: PublicKey
): Promise<PlayerWithRelations> {
  // 1. Fetch player account
  const playerInfo = await connection.getAccountInfo(playerPDA);
  if (!playerInfo) throw new Error('Player not found');

  const player = PlayerAccountSchema.decode(
    playerInfo.data.slice(8)
  ) as PlayerAccount;

  // 2. Derive item PDAs from inventory IDs and fetch
  const itemPDAs = player.inventory.map(itemId =>
    PublicKey.findProgramAddressSync(
      [Buffer.from('item'), Buffer.from(itemId.toString())],
      PROGRAM_ID
    )[0]
  );

  const itemInfos = await connection.getMultipleAccountsInfo(itemPDAs);
  const inventoryItems = itemInfos
    .filter((info): info is NonNullable<typeof info> => info !== null)
    .map(info => GameItemSchema.decode(info.data.slice(8)) as GameItem);

  // 3. Fetch guild if player has one
  let guild: Guild | null = null;
  if (player.guild) {
    const guildInfo = await connection.getAccountInfo(player.guild);
    if (guildInfo) {
      guild = GuildSchema.decode(guildInfo.data.slice(8)) as Guild;
    }
  }

  return { player, inventoryItems, guild };
}
```

### React Hook Pattern

Integrate with React using hooks:

```typescript
import { useConnection } from '@solana/wallet-adapter-react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { PublicKey } from '@solana/web3.js';
import { PlayerAccount, PlayerAccountSchema } from './generated';

// Fetch hook
export function usePlayerAccount(playerPDA: PublicKey | null) {
  const { connection } = useConnection();

  return useQuery({
    queryKey: ['player', playerPDA?.toBase58()],
    queryFn: async () => {
      if (!playerPDA) return null;

      const accountInfo = await connection.getAccountInfo(playerPDA);
      if (!accountInfo) return null;

      return PlayerAccountSchema.decode(
        accountInfo.data.slice(8)
      ) as PlayerAccount;
    },
    enabled: !!playerPDA,
    refetchInterval: 10000, // Refresh every 10 seconds
  });
}

// Usage in component
function PlayerProfile({ playerPDA }: { playerPDA: PublicKey }) {
  const { data: player, isLoading, error } = usePlayerAccount(playerPDA);

  if (isLoading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;
  if (!player) return <div>Player not found</div>;

  return (
    <div>
      <h2>Level {player.level}</h2>
      <p>XP: {player.experience}</p>
      <p>Items: {player.inventory.length}</p>
      <StatusBadge status={player.status} />
    </div>
  );
}

function StatusBadge({ status }: { status: PlayerStatus }) {
  switch (status.kind) {
    case 'Active':
      return <span className="badge green">Active</span>;
    case 'Inactive':
      return <span className="badge gray">Inactive</span>;
    case 'Banned':
      return <span className="badge red">Banned: {status.reason}</span>;
  }
}
```

### Handling u64 Precision

LUMOS generates warnings for u64 fields. Here's how to handle them:

```typescript
import { BN } from '@coral-xyz/anchor';

// Generated interface includes precision warning:
export interface PlayerAccount {
  /** Warning: u64 values above 2^53-1 lose precision in JavaScript */
  experience: number;
  /** Warning: u64 values above 2^53-1 lose precision in JavaScript */
  balance: number;
}

// For large values, use BN (BigNumber)
function formatBalance(balance: number | BN): string {
  // Convert to BN for safe arithmetic
  const bn = new BN(balance);

  // Format as SOL (9 decimals)
  const sol = bn.div(new BN(1_000_000_000));
  const lamports = bn.mod(new BN(1_000_000_000));

  return `${sol.toString()}.${lamports.toString().padStart(9, '0')} SOL`;
}

// Safe comparison for large values
function hasEnoughBalance(player: PlayerAccount, required: number): boolean {
  const playerBalance = new BN(player.balance);
  const requiredBN = new BN(required);
  return playerBalance.gte(requiredBN);
}
```

### Subscribing to Account Changes

Listen for real-time updates:

```typescript
import { Connection, PublicKey } from '@solana/web3.js';
import { PlayerAccount, PlayerAccountSchema } from './generated';

function subscribeToPlayer(
  connection: Connection,
  playerPDA: PublicKey,
  onUpdate: (player: PlayerAccount) => void
): number {
  return connection.onAccountChange(
    playerPDA,
    (accountInfo) => {
      const player = PlayerAccountSchema.decode(
        accountInfo.data.slice(8)
      ) as PlayerAccount;
      onUpdate(player);
    },
    'confirmed'
  );
}

// Usage
const subscriptionId = subscribeToPlayer(
  connection,
  playerPDA,
  (player) => {
    console.log(`Player updated! New level: ${player.level}`);
  }
);

// Cleanup
connection.removeAccountChangeListener(subscriptionId);
```

---

## Testing Generated Code

### Unit Testing Rust Handlers

Test your handlers with mock contexts:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;

    // Helper to create test player
    fn create_test_player() -> PlayerAccount {
        PlayerAccount {
            wallet: Pubkey::new_unique(),
            level: 1,
            experience: 0,
            inventory: vec![],
            guild: None,
            status: PlayerStatus::Active,
        }
    }

    #[test]
    fn test_level_up_adds_experience() {
        let mut player = create_test_player();

        // Simulate adding XP
        player.experience += 100;

        assert_eq!(player.experience, 100);
    }

    #[test]
    fn test_level_up_threshold() {
        let mut player = create_test_player();
        player.experience = 999;

        // Add XP to cross threshold
        player.experience += 1;

        // Check level up logic
        let xp_for_level_2 = 1000;
        if player.experience >= xp_for_level_2 {
            player.level += 1;
        }

        assert_eq!(player.level, 2);
    }

    #[test]
    fn test_inventory_operations() {
        let mut player = create_test_player();

        // Add items
        player.inventory.push(1);
        player.inventory.push(2);
        player.inventory.push(3);

        assert_eq!(player.inventory.len(), 3);
        assert!(player.inventory.contains(&2));

        // Remove item
        player.inventory.retain(|&id| id != 2);

        assert_eq!(player.inventory.len(), 2);
        assert!(!player.inventory.contains(&2));
    }

    #[test]
    fn test_guild_join_leave() {
        let mut player = create_test_player();
        let guild_key = Pubkey::new_unique();

        // Initially no guild
        assert!(player.guild.is_none());

        // Join guild
        player.guild = Some(guild_key);
        assert!(player.guild.is_some());
        assert_eq!(player.guild.unwrap(), guild_key);

        // Leave guild
        player.guild = None;
        assert!(player.guild.is_none());
    }

    #[test]
    fn test_status_transitions() {
        let mut player = create_test_player();

        // Active by default
        assert!(matches!(player.status, PlayerStatus::Active));

        // Ban player
        player.status = PlayerStatus::Banned {
            reason: "Cheating".to_string(),
            until: 1234567890,
        };

        assert!(matches!(player.status, PlayerStatus::Banned { .. }));

        if let PlayerStatus::Banned { reason, until } = &player.status {
            assert_eq!(reason, "Cheating");
            assert_eq!(*until, 1234567890);
        }

        // Reactivate
        player.status = PlayerStatus::Active;
        assert!(matches!(player.status, PlayerStatus::Active));
    }
}
```

### Integration Testing with Bankrun

Test full program flow:

```rust
#[cfg(test)]
mod integration_tests {
    use anchor_lang::prelude::*;
    use solana_program_test::*;
    use solana_sdk::{signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn test_create_and_level_up_player() {
        // Setup test environment
        let program_id = Pubkey::new_unique();
        let mut program_test = ProgramTest::new(
            "your_program",
            program_id,
            processor!(process_instruction),
        );

        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

        // Create player account
        let player_keypair = Keypair::new();

        // ... build and send create_player transaction

        // Verify player was created
        let player_account = banks_client
            .get_account(player_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();

        let player: PlayerAccount = AccountDeserialize::try_deserialize(
            &mut player_account.data.as_ref()
        ).unwrap();

        assert_eq!(player.level, 1);
        assert_eq!(player.experience, 0);

        // ... send level_up transaction

        // Verify level up
        let updated_account = banks_client
            .get_account(player_keypair.pubkey())
            .await
            .unwrap()
            .unwrap();

        let updated_player: PlayerAccount = AccountDeserialize::try_deserialize(
            &mut updated_account.data.as_ref()
        ).unwrap();

        assert!(updated_player.experience > 0);
    }
}
```

### TypeScript Client Testing

Test your frontend code:

```typescript
import { describe, it, expect, beforeAll } from 'vitest';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { PlayerAccount, PlayerAccountSchema, PlayerStatus } from './generated';

describe('PlayerAccount Deserialization', () => {
  it('should deserialize active player correctly', () => {
    // Create mock account data
    const mockData = Buffer.alloc(200);
    let offset = 0;

    // Write wallet (32 bytes)
    const wallet = Keypair.generate().publicKey;
    wallet.toBuffer().copy(mockData, offset);
    offset += 32;

    // Write level (u16 = 2 bytes)
    mockData.writeUInt16LE(5, offset);
    offset += 2;

    // Write experience (u64 = 8 bytes)
    mockData.writeBigUInt64LE(BigInt(1500), offset);
    offset += 8;

    // ... continue for other fields

    // Deserialize
    const player = PlayerAccountSchema.decode(mockData) as PlayerAccount;

    expect(player.level).toBe(5);
    expect(player.experience).toBe(1500);
  });

  it('should handle enum variants correctly', () => {
    const activeStatus: PlayerStatus = { kind: 'Active' };
    const bannedStatus: PlayerStatus = {
      kind: 'Banned',
      reason: 'Test ban',
      until: 1234567890,
    };

    expect(activeStatus.kind).toBe('Active');
    expect(bannedStatus.kind).toBe('Banned');

    if (bannedStatus.kind === 'Banned') {
      expect(bannedStatus.reason).toBe('Test ban');
    }
  });
});

describe('PlayerAccount Type Guards', () => {
  it('should narrow types with switch statement', () => {
    const status: PlayerStatus = { kind: 'Banned', reason: 'Test', until: 0 };

    let message = '';
    switch (status.kind) {
      case 'Active':
        message = 'active';
        break;
      case 'Banned':
        message = `banned: ${status.reason}`; // TypeScript knows reason exists
        break;
      case 'Inactive':
        message = 'inactive';
        break;
    }

    expect(message).toBe('banned: Test');
  });
});
```

---

## Common Patterns & Anti-Patterns

### DO's and DON'Ts

#### Account Space

```rust
// ✅ DO: Use generated LEN constant
#[account(
    init,
    payer = authority,
    space = PlayerAccount::LEN,
)]
pub player: Account<'info, PlayerAccount>,

// ❌ DON'T: Hardcode space calculations
#[account(
    init,
    payer = authority,
    space = 8 + 32 + 2 + 8 + 84 + 33 + 1,  // Error-prone!
)]
pub player: Account<'info, PlayerAccount>,
```

#### Discriminator Handling

```typescript
// ✅ DO: Skip Anchor's 8-byte discriminator
const data = accountInfo.data.slice(8);
const player = PlayerAccountSchema.decode(data);

// ❌ DON'T: Deserialize full buffer
const player = PlayerAccountSchema.decode(accountInfo.data); // Wrong!
```

#### Enum Checking

```typescript
// ✅ DO: Check kind before accessing variant fields
if (status.kind === 'Banned') {
  console.log(status.reason);  // TypeScript knows reason exists
}

// ❌ DON'T: Access variant fields without checking
console.log((status as any).reason);  // Unsafe!
```

#### Array Capacity

```rust
// ✅ DO: Check array capacity before push
require!(
    player.inventory.len() < MAX_INVENTORY_SIZE,
    ErrorCode::InventoryFull
);
player.inventory.push(item_id);

// ❌ DON'T: Push without checking (can exceed account space)
player.inventory.push(item_id);  // May cause realloc failure!
```

#### Option Handling

```rust
// ✅ DO: Use pattern matching or if let
if let Some(guild) = &player.guild {
    msg!("In guild: {}", guild);
}

// ✅ DO: Use unwrap_or for defaults
let guild = player.guild.unwrap_or(Pubkey::default());

// ❌ DON'T: Unwrap without checking
let guild = player.guild.unwrap();  // Panics if None!
```

### Safe Array Operations

```rust
// Pattern: Safe array access with bounds checking
pub fn get_inventory_item(player: &PlayerAccount, index: usize) -> Option<u64> {
    player.inventory.get(index).copied()
}

// Pattern: Safe removal by value
pub fn remove_item(player: &mut PlayerAccount, item_id: u64) -> bool {
    if let Some(index) = player.inventory.iter().position(|&id| id == item_id) {
        player.inventory.remove(index);
        true
    } else {
        false
    }
}

// Pattern: Safe iteration with modification
pub fn remove_broken_items(player: &mut PlayerAccount, broken_ids: &[u64]) {
    player.inventory.retain(|id| !broken_ids.contains(id));
}
```

### State Machine with Enums

```rust
// Pattern: Valid state transitions
impl PlayerStatus {
    pub fn can_transition_to(&self, new_status: &PlayerStatus) -> bool {
        match (self, new_status) {
            // Active can go to Inactive or Banned
            (PlayerStatus::Active, PlayerStatus::Inactive) => true,
            (PlayerStatus::Active, PlayerStatus::Banned { .. }) => true,

            // Inactive can go to Active
            (PlayerStatus::Inactive, PlayerStatus::Active) => true,

            // Banned can only go to Active (after ban expires)
            (PlayerStatus::Banned { .. }, PlayerStatus::Active) => true,

            // Same state is always valid
            _ if std::mem::discriminant(self) == std::mem::discriminant(new_status) => true,

            _ => false,
        }
    }
}

// Usage
pub fn change_status(
    player: &mut PlayerAccount,
    new_status: PlayerStatus,
) -> Result<()> {
    require!(
        player.status.can_transition_to(&new_status),
        ErrorCode::InvalidStatusTransition
    );

    player.status = new_status;
    Ok(())
}
```

### PDA Derivation with Generated Types

```rust
// Pattern: Derive PDAs using generated type fields
impl PlayerAccount {
    pub fn find_pda(wallet: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"player", wallet.as_ref()],
            program_id,
        )
    }
}

impl GameItem {
    pub fn find_pda(
        player: &Pubkey,
        item_id: u64,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"item",
                player.as_ref(),
                &item_id.to_le_bytes(),
            ],
            program_id,
        )
    }
}
```

```typescript
// TypeScript equivalent
function findPlayerPDA(wallet: PublicKey, programId: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('player'), wallet.toBuffer()],
    programId
  );
}

function findItemPDA(
  player: PublicKey,
  itemId: number,
  programId: PublicKey
): [PublicKey, number] {
  const itemIdBuffer = Buffer.alloc(8);
  itemIdBuffer.writeBigUInt64LE(BigInt(itemId));

  return PublicKey.findProgramAddressSync(
    [Buffer.from('item'), player.toBuffer(), itemIdBuffer],
    programId
  );
}
```

---

## Complete Working Example

Let's build a complete mini-game with schema, handlers, and client.

### Schema (schema.lumos)

```lumos
// Mini RPG Game Schema

#[solana]
#[account]
struct Player {
    /// Player's wallet address
    wallet: PublicKey,
    /// Player name (max 32 chars)
    #[max(32)]
    name: String,
    /// Current level (1-100)
    level: u16,
    /// Experience points
    experience: u64,
    /// Gold balance
    gold: u64,
    /// Equipped weapon ID (0 = none)
    equipped_weapon: u64,
    /// Inventory item IDs (max 20)
    inventory: [u64; 20],
    /// Number of items in inventory
    inventory_count: u8,
    /// Player status
    status: PlayerStatus,
    /// Account bump seed
    bump: u8,
}

#[solana]
enum PlayerStatus {
    /// Active and can play
    Active,
    /// Resting (regenerating)
    Resting { until: i64 },
    /// In combat
    InCombat { enemy_id: u64 },
    /// Dead (needs revival)
    Dead,
}

#[solana]
#[account]
struct Weapon {
    /// Weapon ID
    id: u64,
    /// Weapon name
    #[max(32)]
    name: String,
    /// Damage value
    damage: u16,
    /// Required level to equip
    required_level: u16,
    /// Price in gold
    price: u64,
    /// Bump seed
    bump: u8,
}
```

### Generated Rust (generated.rs)

```rust
// Auto-generated by LUMOS - DO NOT EDIT
use anchor_lang::prelude::*;

#[account]
pub struct Player {
    pub wallet: Pubkey,
    pub name: String,
    pub level: u16,
    pub experience: u64,
    pub gold: u64,
    pub equipped_weapon: u64,
    pub inventory: [u64; 20],
    pub inventory_count: u8,
    pub status: PlayerStatus,
    pub bump: u8,
}

impl Player {
    pub const LEN: usize = 8 +  // discriminator
        32 +                     // wallet
        (4 + 32) +              // name (string)
        2 +                      // level
        8 +                      // experience
        8 +                      // gold
        8 +                      // equipped_weapon
        (20 * 8) +              // inventory
        1 +                      // inventory_count
        (1 + 8) +               // status (max variant size)
        1;                       // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PlayerStatus {
    Active,
    Resting { until: i64 },
    InCombat { enemy_id: u64 },
    Dead,
}

#[account]
pub struct Weapon {
    pub id: u64,
    pub name: String,
    pub damage: u16,
    pub required_level: u16,
    pub price: u64,
    pub bump: u8,
}

impl Weapon {
    pub const LEN: usize = 8 + 8 + (4 + 32) + 2 + 2 + 8 + 1;
}
```

### Handler Implementation (lib.rs)

```rust
use anchor_lang::prelude::*;

mod generated;
use generated::{Player, PlayerStatus, Weapon};

declare_id!("Game111111111111111111111111111111111111111");

#[program]
pub mod mini_rpg {
    use super::*;

    pub fn create_player(
        ctx: Context<CreatePlayer>,
        name: String,
    ) -> Result<()> {
        require!(name.len() <= 32, ErrorCode::NameTooLong);

        let player = &mut ctx.accounts.player;
        player.wallet = ctx.accounts.authority.key();
        player.name = name;
        player.level = 1;
        player.experience = 0;
        player.gold = 100; // Starting gold
        player.equipped_weapon = 0;
        player.inventory = [0u64; 20];
        player.inventory_count = 0;
        player.status = PlayerStatus::Active;
        player.bump = ctx.bumps.player;

        msg!("Created player: {}", player.name);
        Ok(())
    }

    pub fn gain_experience(
        ctx: Context<UpdatePlayer>,
        xp: u64,
    ) -> Result<()> {
        let player = &mut ctx.accounts.player;

        // Must be active
        require!(
            matches!(player.status, PlayerStatus::Active),
            ErrorCode::PlayerNotActive
        );

        // Add XP
        player.experience = player.experience.saturating_add(xp);

        // Check for level up
        let xp_for_next = (player.level as u64) * 1000;
        while player.experience >= xp_for_next && player.level < 100 {
            player.level += 1;
            msg!("Level up! Now level {}", player.level);
        }

        Ok(())
    }

    pub fn buy_weapon(ctx: Context<BuyWeapon>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        let weapon = &ctx.accounts.weapon;

        // Check gold
        require!(player.gold >= weapon.price, ErrorCode::NotEnoughGold);

        // Check level requirement
        require!(
            player.level >= weapon.required_level,
            ErrorCode::LevelTooLow
        );

        // Check inventory space
        require!(
            (player.inventory_count as usize) < 20,
            ErrorCode::InventoryFull
        );

        // Deduct gold
        player.gold = player.gold.saturating_sub(weapon.price);

        // Add to inventory
        let slot = player.inventory_count as usize;
        player.inventory[slot] = weapon.id;
        player.inventory_count += 1;

        msg!("Bought weapon: {} for {} gold", weapon.name, weapon.price);
        Ok(())
    }

    pub fn equip_weapon(
        ctx: Context<UpdatePlayer>,
        weapon_id: u64,
    ) -> Result<()> {
        let player = &mut ctx.accounts.player;

        // Check if weapon is in inventory
        let has_weapon = player.inventory[..player.inventory_count as usize]
            .contains(&weapon_id);
        require!(has_weapon, ErrorCode::WeaponNotOwned);

        // Equip
        player.equipped_weapon = weapon_id;
        msg!("Equipped weapon {}", weapon_id);

        Ok(())
    }

    pub fn start_rest(ctx: Context<UpdatePlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        let clock = Clock::get()?;

        require!(
            matches!(player.status, PlayerStatus::Active),
            ErrorCode::PlayerNotActive
        );

        // Rest for 1 hour
        player.status = PlayerStatus::Resting {
            until: clock.unix_timestamp + 3600,
        };

        msg!("Player started resting");
        Ok(())
    }

    pub fn wake_up(ctx: Context<UpdatePlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        let clock = Clock::get()?;

        if let PlayerStatus::Resting { until } = player.status {
            require!(
                clock.unix_timestamp >= until,
                ErrorCode::StillResting
            );

            player.status = PlayerStatus::Active;
            // Bonus XP for resting
            player.experience = player.experience.saturating_add(50);

            msg!("Player woke up and gained 50 XP");
        } else {
            return Err(ErrorCode::NotResting.into());
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePlayer<'info> {
    #[account(
        init,
        payer = authority,
        space = Player::LEN,
        seeds = [b"player", authority.key().as_ref()],
        bump,
    )]
    pub player: Account<'info, Player>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePlayer<'info> {
    #[account(
        mut,
        seeds = [b"player", authority.key().as_ref()],
        bump = player.bump,
    )]
    pub player: Account<'info, Player>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct BuyWeapon<'info> {
    #[account(
        mut,
        seeds = [b"player", authority.key().as_ref()],
        bump = player.bump,
    )]
    pub player: Account<'info, Player>,

    pub weapon: Account<'info, Weapon>,

    pub authority: Signer<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Name exceeds 32 characters")]
    NameTooLong,
    #[msg("Player is not active")]
    PlayerNotActive,
    #[msg("Not enough gold")]
    NotEnoughGold,
    #[msg("Level requirement not met")]
    LevelTooLow,
    #[msg("Inventory is full")]
    InventoryFull,
    #[msg("Weapon not in inventory")]
    WeaponNotOwned,
    #[msg("Player is not resting")]
    NotResting,
    #[msg("Still resting")]
    StillResting,
}
```

### TypeScript Client (client.ts)

```typescript
import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import * as borsh from '@coral-xyz/borsh';

// Generated types
export interface Player {
  wallet: PublicKey;
  name: string;
  level: number;
  experience: number;
  gold: number;
  equippedWeapon: number;
  inventory: number[];
  inventoryCount: number;
  status: PlayerStatus;
  bump: number;
}

export type PlayerStatus =
  | { kind: 'Active' }
  | { kind: 'Resting'; until: number }
  | { kind: 'InCombat'; enemyId: number }
  | { kind: 'Dead' };

export interface Weapon {
  id: number;
  name: string;
  damage: number;
  requiredLevel: number;
  price: number;
  bump: number;
}

// Borsh schemas
const PlayerStatusSchema = borsh.rustEnum([
  borsh.struct([], 'Active'),
  borsh.struct([borsh.i64('until')], 'Resting'),
  borsh.struct([borsh.u64('enemyId')], 'InCombat'),
  borsh.struct([], 'Dead'),
]);

export const PlayerSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.str('name'),
  borsh.u16('level'),
  borsh.u64('experience'),
  borsh.u64('gold'),
  borsh.u64('equippedWeapon'),
  borsh.array(borsh.u64(), 20, 'inventory'),
  borsh.u8('inventoryCount'),
  PlayerStatusSchema,
  borsh.u8('bump'),
]);

// Client class
export class MiniRPGClient {
  constructor(
    private program: Program,
    private connection: Connection
  ) {}

  // Find player PDA
  findPlayerPDA(wallet: PublicKey): [PublicKey, number] {
    return PublicKey.findProgramAddressSync(
      [Buffer.from('player'), wallet.toBuffer()],
      this.program.programId
    );
  }

  // Fetch player account
  async fetchPlayer(wallet: PublicKey): Promise<Player | null> {
    const [playerPDA] = this.findPlayerPDA(wallet);
    const accountInfo = await this.connection.getAccountInfo(playerPDA);

    if (!accountInfo) return null;

    return PlayerSchema.decode(accountInfo.data.slice(8)) as Player;
  }

  // Create new player
  async createPlayer(name: string): Promise<string> {
    const [playerPDA] = this.findPlayerPDA(
      this.program.provider.publicKey!
    );

    return this.program.methods
      .createPlayer(name)
      .accounts({ player: playerPDA })
      .rpc();
  }

  // Gain experience
  async gainExperience(xp: number): Promise<string> {
    const [playerPDA] = this.findPlayerPDA(
      this.program.provider.publicKey!
    );

    return this.program.methods
      .gainExperience(new BN(xp))
      .accounts({ player: playerPDA })
      .rpc();
  }

  // Display player info
  displayPlayer(player: Player): void {
    console.log('=== Player Info ===');
    console.log(`Name: ${player.name}`);
    console.log(`Level: ${player.level}`);
    console.log(`XP: ${player.experience}`);
    console.log(`Gold: ${player.gold}`);
    console.log(`Equipped: ${player.equippedWeapon || 'None'}`);
    console.log(`Inventory: ${player.inventoryCount} items`);

    // Display status with type narrowing
    switch (player.status.kind) {
      case 'Active':
        console.log('Status: Active');
        break;
      case 'Resting':
        const wakeTime = new Date(player.status.until * 1000);
        console.log(`Status: Resting until ${wakeTime.toLocaleString()}`);
        break;
      case 'InCombat':
        console.log(`Status: Fighting enemy #${player.status.enemyId}`);
        break;
      case 'Dead':
        console.log('Status: Dead (needs revival)');
        break;
    }
  }

  // Get inventory items
  getInventoryItems(player: Player): number[] {
    return player.inventory.slice(0, player.inventoryCount);
  }

  // Check if can perform action
  canPerformAction(player: Player): boolean {
    return player.status.kind === 'Active';
  }
}

// Usage example
async function main() {
  const connection = new Connection('http://localhost:8899');
  const wallet = Keypair.generate();

  // Initialize client
  const provider = new AnchorProvider(connection, wallet as any, {});
  const program = new Program(IDL, PROGRAM_ID, provider);
  const client = new MiniRPGClient(program, connection);

  // Create player
  await client.createPlayer('Hero');

  // Fetch and display
  const player = await client.fetchPlayer(wallet.publicKey);
  if (player) {
    client.displayPlayer(player);

    // Check status before action
    if (client.canPerformAction(player)) {
      await client.gainExperience(500);
    }
  }
}
```

---

## Troubleshooting

### Deserialization Errors

**Problem:** `Error: Deserialization failed`

```typescript
// ❌ Wrong: Not skipping discriminator
const player = PlayerSchema.decode(accountInfo.data);

// ✅ Correct: Skip 8-byte Anchor discriminator
const player = PlayerSchema.decode(accountInfo.data.slice(8));
```

**Problem:** `Error: Invalid enum discriminant`

```rust
// Check that enum variants match exactly between schema and code
// The order of variants determines discriminant values:
enum Status {
    Active,    // discriminant = 0
    Inactive,  // discriminant = 1
    Banned,    // discriminant = 2
}
```

### Account Size Errors

**Problem:** `Error: Account data too small`

```rust
// ❌ Wrong: Manual size calculation
space = 100,

// ✅ Correct: Use generated LEN constant
space = Player::LEN,
```

**Problem:** `Error: Account size exceeded` when adding to array

```rust
// Always check capacity before modifying arrays
require!(
    player.inventory.len() < MAX_SIZE,
    ErrorCode::CapacityExceeded
);
```

### Type Mismatch Errors

**Problem:** TypeScript type doesn't match Rust

```typescript
// Check field order matches schema exactly
// LUMOS generates fields in schema order

// If schema has:
// struct Player { wallet: PublicKey, level: u16 }

// TypeScript must match:
const schema = borsh.struct([
  borsh.publicKey('wallet'),  // First
  borsh.u16('level'),         // Second
]);
```

### Option Field Issues

**Problem:** Option field serialization mismatch

```typescript
// Borsh Option encoding:
// - None: [0]
// - Some(value): [1, ...value_bytes]

// Use borsh.option() wrapper
borsh.option(borsh.publicKey(), 'guild')
```

### Enum Variant Access

**Problem:** TypeScript error accessing enum variant fields

```typescript
// ❌ Wrong: Direct access without narrowing
console.log(status.until);  // Error: Property 'until' doesn't exist

// ✅ Correct: Narrow type first
if (status.kind === 'Resting') {
  console.log(status.until);  // OK: TypeScript knows 'until' exists
}
```

---

## Related Guides

After mastering usage patterns, explore these guides:

**Foundation Guides:**
- [LUMOS + Anchor Integration](/guides/anchor-integration) - Schema design and generation
- [LUMOS + web3.js Integration](/guides/web3js-integration) - Connection and transaction patterns
- [LUMOS + Solana CLI Integration](/guides/solana-cli-integration) - Deployment workflows

**Use Case Guides:**
- [Gaming Projects](/guides/use-cases/gaming) - Game-specific patterns
- [DeFi Protocols](/guides/use-cases/defi) - Financial application patterns
- [NFT Marketplaces](/guides/use-cases/nft) - NFT and metadata patterns

**Advanced Guides:**
- [Error Handling & Validation Patterns](/guides/error-handling-validation) - Comprehensive error handling
- [LUMOS + Solv/Jito Integration](/guides/solv-jito-integration) - Liquid staking and MEV
- [Migration: TypeScript → LUMOS](/guides/migration-typescript) - Migrate existing types
- [Migration: Anchor → LUMOS](/guides/migration-anchor) - Add LUMOS to existing programs

---

## Summary

This guide covered practical usage patterns for LUMOS-generated code:

| Pattern | Rust | TypeScript |
|---------|------|------------|
| Field access | `player.level` | `player.level` |
| Array ops | `inventory.push(id)` | `inventory.push(id)` |
| Option handling | `if let Some(g) = guild` | `if (guild !== null)` |
| Enum matching | `match status { ... }` | `switch (status.kind)` |
| Space calc | `Player::LEN` | N/A |
| Deserialize | Auto (Anchor) | `Schema.decode(data.slice(8))` |

**Key Takeaways:**
1. Always skip the 8-byte discriminator in TypeScript
2. Use generated `LEN` constants for account space
3. Check array capacity before push operations
4. Use type narrowing for enum variant access
5. Handle Option fields with null checks

Start with the complete example and adapt patterns to your use case!
