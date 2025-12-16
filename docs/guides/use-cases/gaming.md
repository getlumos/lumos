# Gaming Projects with LUMOS

**Purpose:** Complete guide for building on-chain games with LUMOS on Solana

**Last Updated:** 2025-12-16

---

## Overview

This guide walks through building a complete on-chain RPG using LUMOS, covering:

- Player accounts with stats, inventory, and progression
- Item system with rarity, equipment, and trading
- Seasonal leaderboards and rankings
- Match system for PvP and PvE gameplay
- Performance optimization for games

**What we'll build:**

```
┌─────────────────────────────────────────────────────────────┐
│                    On-Chain RPG Game                        │
├─────────────────────────────────────────────────────────────┤
│  • Player Accounts (level, XP, stats, currency)            │
│  • Item System (weapons, armor, consumables)               │
│  • Inventory Management (equip, unequip, trade)            │
│  • Leaderboards (seasonal rankings)                        │
│  • Match System (PvP battles, PvE quests)                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Table of Contents

1. [Why LUMOS for Gaming](#why-lumos-for-gaming)
2. [Game Architecture](#game-architecture)
3. [Player Account System](#player-account-system)
4. [Item & Inventory System](#item--inventory-system)
5. [Leaderboard System](#leaderboard-system)
6. [Match System](#match-system)
7. [State Management Patterns](#state-management-patterns)
8. [Performance Considerations](#performance-considerations)
9. [Complete Implementation](#complete-implementation)
10. [Frontend Integration](#frontend-integration)
11. [Resources](#resources)

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

---

## Why LUMOS for Gaming

### The Challenge

On-chain games require:
- **Type safety** across Rust programs and TypeScript clients
- **Rapid iteration** as game mechanics evolve
- **Complex data structures** for inventory, stats, achievements
- **Zero type drift** between on-chain and off-chain

### The Solution

LUMOS provides:

| Benefit | Impact on Gaming |
|---------|------------------|
| Single source of truth | One schema defines all player/item types |
| Generated Borsh schemas | Automatic serialization for game state |
| Type-safe TypeScript | Catch bugs before deployment |
| Anchor integration | Native `#[account]` support |

### Gaming on Solana

Solana is ideal for games because:
- **400ms block times** - Near real-time updates
- **Low fees** - Microtransactions viable
- **High throughput** - Handle many concurrent players
- **Composability** - Integrate with DeFi, NFTs

---

## Game Architecture

### Account Model

```
┌─────────────────┐         ┌─────────────────┐
│  PlayerAccount  │────────▶│    GameItem     │
│     (PDA)       │  owns   │     (PDA)       │
│                 │         │                 │
│ • wallet (key)  │         │ • id (key)      │
│ • username      │         │ • owner         │
│ • level, xp     │         │ • name, rarity  │
│ • gold, gems    │         │ • power, defense│
│ • equipped[]    │         │ • is_equipped   │
│ • inventory[]   │         │ • durability    │
└────────┬────────┘         └─────────────────┘
         │
         │ participates
         ▼
┌─────────────────┐         ┌─────────────────┐
│   Leaderboard   │         │   MatchResult   │
│     (PDA)       │         │   (event data)  │
│                 │         │                 │
│ • season (key)  │         │ • player        │
│ • top_players[] │         │ • opponent      │
│ • top_scores[]  │         │ • score         │
│ • is_active     │         │ • rewards       │
└─────────────────┘         └─────────────────┘
```

### PDA Derivation Strategy

```rust
// Player PDA - one per wallet
seeds = [b"player", wallet.as_ref()]

// Item PDA - unique per item ID
seeds = [b"item", item_id.to_le_bytes().as_ref()]

// Leaderboard PDA - one per season
seeds = [b"leaderboard", season.to_le_bytes().as_ref()]

// Player inventory PDA - links player to items
seeds = [b"inventory", player.as_ref()]
```

### Program Structure

```
programs/gaming/
├── src/
│   ├── lib.rs              # Program entry point
│   ├── state/
│   │   ├── mod.rs
│   │   ├── player.rs       # PlayerAccount (from LUMOS)
│   │   ├── item.rs         # GameItem (from LUMOS)
│   │   └── leaderboard.rs  # Leaderboard (from LUMOS)
│   ├── instructions/
│   │   ├── mod.rs
│   │   ├── player.rs       # create, update, delete
│   │   ├── item.rs         # mint, equip, trade
│   │   ├── match.rs        # start, complete
│   │   └── leaderboard.rs  # update, archive
│   └── errors.rs           # Custom errors
└── Cargo.toml
```

---

## Player Account System

### Complete Schema

```rust
// schemas/gaming.lumos

#[solana]
#[account]
struct PlayerAccount {
    // === Identity ===
    #[key]
    wallet: PublicKey,          // Owner's wallet, PDA seed
    #[max(20)]
    username: String,           // Display name (max 20 chars)

    // === RPG Stats ===
    level: u16,                 // Current level (1-65535)
    experience: u64,            // Total XP earned
    health: u16,                // Current HP
    mana: u16,                  // Current MP

    // === Economy ===
    gold: u64,                  // Soft currency (earned in-game)
    gems: u32,                  // Hard currency (purchased/rare)

    // === Inventory References ===
    equipped_items: [PublicKey],    // Currently equipped item PDAs
    inventory_items: [PublicKey],   // All owned item PDAs

    // === Progress ===
    quests_completed: u32,      // Total quests finished
    achievements: [u32],        // Achievement IDs unlocked

    // === Timestamps ===
    created_at: i64,            // Account creation time
    last_login: i64,            // Last activity timestamp
    total_playtime: i64,        // Cumulative seconds played
}
```

### Generate Code

```bash
lumos generate schemas/gaming.lumos --output programs/gaming/src/state/
```

### Level Progression System

```rust
// programs/gaming/src/instructions/player.rs

use anchor_lang::prelude::*;
use crate::state::PlayerAccount;

// XP thresholds for each level
pub const XP_PER_LEVEL: u64 = 1000;
pub const MAX_LEVEL: u16 = 100;

// Stats gained per level
pub const HP_PER_LEVEL: u16 = 10;
pub const MP_PER_LEVEL: u16 = 5;

pub fn gain_experience(ctx: Context<GainExperience>, amount: u64) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Add experience
    player.experience = player.experience.saturating_add(amount);

    // Calculate new level
    let new_level = calculate_level(player.experience);

    // Level up if needed
    if new_level > player.level && new_level <= MAX_LEVEL {
        let levels_gained = new_level - player.level;

        // Increase stats
        player.health = player.health.saturating_add(HP_PER_LEVEL * levels_gained);
        player.mana = player.mana.saturating_add(MP_PER_LEVEL * levels_gained);
        player.level = new_level;

        msg!("Level up! {} -> {}", player.level - levels_gained, new_level);
    }

    Ok(())
}

fn calculate_level(experience: u64) -> u16 {
    // Level = floor(XP / 1000) + 1, capped at MAX_LEVEL
    let level = (experience / XP_PER_LEVEL) as u16 + 1;
    level.min(MAX_LEVEL)
}

#[derive(Accounts)]
pub struct GainExperience<'info> {
    #[account(
        mut,
        seeds = [b"player", authority.key().as_ref()],
        bump,
        has_one = wallet @ GameError::Unauthorized,
    )]
    pub player: Account<'info, PlayerAccount>,
    pub authority: Signer<'info>,
}
```

### Player Creation

```rust
pub fn create_player(
    ctx: Context<CreatePlayer>,
    username: String,
) -> Result<()> {
    require!(username.len() <= 20, GameError::UsernameTooLong);
    require!(username.len() >= 3, GameError::UsernameTooShort);

    let player = &mut ctx.accounts.player;
    let clock = Clock::get()?;

    // Initialize player
    player.wallet = ctx.accounts.authority.key();
    player.username = username;

    // Starting stats
    player.level = 1;
    player.experience = 0;
    player.health = 100;
    player.mana = 50;

    // Starting currency
    player.gold = 100;  // Starter gold
    player.gems = 0;

    // Empty inventory
    player.equipped_items = vec![];
    player.inventory_items = vec![];

    // Progress
    player.quests_completed = 0;
    player.achievements = vec![];

    // Timestamps
    player.created_at = clock.unix_timestamp;
    player.last_login = clock.unix_timestamp;
    player.total_playtime = 0;

    msg!("Player {} created!", player.username);
    Ok(())
}

#[derive(Accounts)]
pub struct CreatePlayer<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PlayerAccount::INIT_SPACE,
        seeds = [b"player", authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, PlayerAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

### Account Size Calculation

```rust
impl PlayerAccount {
    // Fixed size estimation for rent calculation
    pub const INIT_SPACE: usize =
        32 +        // wallet: PublicKey
        4 + 20 +    // username: String (4 byte len + max 20 chars)
        2 +         // level: u16
        8 +         // experience: u64
        2 +         // health: u16
        2 +         // mana: u16
        8 +         // gold: u64
        4 +         // gems: u32
        4 + (32 * 10) +  // equipped_items: Vec<Pubkey> (max 10)
        4 + (32 * 50) +  // inventory_items: Vec<Pubkey> (max 50)
        4 +         // quests_completed: u32
        4 + (4 * 100) +  // achievements: Vec<u32> (max 100)
        8 +         // created_at: i64
        8 +         // last_login: i64
        8;          // total_playtime: i64
    // Total: ~2,500 bytes
}
```

---

## Item & Inventory System

### Item Schema

```rust
#[solana]
#[account]
struct GameItem {
    // === Identity ===
    #[key]
    id: u64,                    // Unique item ID
    owner: PublicKey,           // Current owner's player PDA

    // === Item Details ===
    #[max(50)]
    name: String,               // Item name
    item_type: u8,              // See ItemType enum
    rarity: u8,                 // See Rarity enum

    // === Combat Stats ===
    power: u16,                 // Attack power
    defense: u16,               // Defense rating
    speed: u16,                 // Speed bonus

    // === State ===
    is_equipped: bool,          // Currently equipped?
    is_tradeable: bool,         // Can be traded?
    durability: u16,            // Current durability (0 = broken)

    // === Timestamps ===
    acquired_at: i64,           // When player got this item
}
```

### Item Type Constants

```rust
// programs/gaming/src/state/item.rs

// Item Types
pub mod item_type {
    pub const WEAPON: u8 = 0;
    pub const ARMOR: u8 = 1;
    pub const HELMET: u8 = 2;
    pub const BOOTS: u8 = 3;
    pub const ACCESSORY: u8 = 4;
    pub const CONSUMABLE: u8 = 5;
}

// Rarity Levels
pub mod rarity {
    pub const COMMON: u8 = 0;
    pub const UNCOMMON: u8 = 1;
    pub const RARE: u8 = 2;
    pub const EPIC: u8 = 3;
    pub const LEGENDARY: u8 = 4;
}

// Stat multipliers by rarity
pub fn get_rarity_multiplier(rarity: u8) -> f32 {
    match rarity {
        0 => 1.0,   // Common: base stats
        1 => 1.25,  // Uncommon: +25%
        2 => 1.5,   // Rare: +50%
        3 => 2.0,   // Epic: +100%
        4 => 3.0,   // Legendary: +200%
        _ => 1.0,
    }
}

// Base durability by item type
pub fn get_base_durability(item_type: u8) -> u16 {
    match item_type {
        0 => 100,   // Weapon
        1 => 150,   // Armor
        2 => 80,    // Helmet
        3 => 80,    // Boots
        4 => 200,   // Accessory
        5 => 1,     // Consumable (single use)
        _ => 100,
    }
}
```

### Minting Items

```rust
pub fn mint_item(
    ctx: Context<MintItem>,
    item_id: u64,
    name: String,
    item_type: u8,
    rarity: u8,
    base_power: u16,
    base_defense: u16,
    base_speed: u16,
) -> Result<()> {
    require!(name.len() <= 50, GameError::NameTooLong);
    require!(item_type <= 5, GameError::InvalidItemType);
    require!(rarity <= 4, GameError::InvalidRarity);

    let item = &mut ctx.accounts.item;
    let clock = Clock::get()?;
    let multiplier = get_rarity_multiplier(rarity);

    // Set item properties
    item.id = item_id;
    item.owner = ctx.accounts.player.key();
    item.name = name;
    item.item_type = item_type;
    item.rarity = rarity;

    // Apply rarity multiplier to stats
    item.power = (base_power as f32 * multiplier) as u16;
    item.defense = (base_defense as f32 * multiplier) as u16;
    item.speed = (base_speed as f32 * multiplier) as u16;

    // Initial state
    item.is_equipped = false;
    item.is_tradeable = true;
    item.durability = get_base_durability(item_type);
    item.acquired_at = clock.unix_timestamp;

    // Add to player's inventory
    let player = &mut ctx.accounts.player;
    player.inventory_items.push(item.key());

    msg!("Minted {} (Rarity: {})", item.name, rarity);
    Ok(())
}

#[derive(Accounts)]
#[instruction(item_id: u64)]
pub struct MintItem<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GameItem::INIT_SPACE,
        seeds = [b"item", item_id.to_le_bytes().as_ref()],
        bump
    )]
    pub item: Account<'info, GameItem>,

    #[account(
        mut,
        seeds = [b"player", authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, PlayerAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

### Equip/Unequip System

```rust
pub fn equip_item(ctx: Context<EquipItem>) -> Result<()> {
    let item = &mut ctx.accounts.item;
    let player = &mut ctx.accounts.player;

    // Validate ownership
    require!(item.owner == player.key(), GameError::NotItemOwner);
    require!(!item.is_equipped, GameError::AlreadyEquipped);
    require!(item.durability > 0, GameError::ItemBroken);

    // Check equipment slot availability
    let slot_count = player.equipped_items.iter()
        .filter(|&&pubkey| {
            // Would need to load each item to check type
            // Simplified: limit total equipped items
            true
        })
        .count();
    require!(slot_count < 6, GameError::EquipmentSlotsFull);

    // Equip the item
    item.is_equipped = true;
    player.equipped_items.push(item.key());

    // Remove from general inventory
    player.inventory_items.retain(|&pubkey| pubkey != item.key());

    msg!("Equipped {}", item.name);
    Ok(())
}

pub fn unequip_item(ctx: Context<UnequipItem>) -> Result<()> {
    let item = &mut ctx.accounts.item;
    let player = &mut ctx.accounts.player;

    require!(item.owner == player.key(), GameError::NotItemOwner);
    require!(item.is_equipped, GameError::NotEquipped);

    // Unequip
    item.is_equipped = false;
    player.equipped_items.retain(|&pubkey| pubkey != item.key());
    player.inventory_items.push(item.key());

    msg!("Unequipped {}", item.name);
    Ok(())
}

#[derive(Accounts)]
pub struct EquipItem<'info> {
    #[account(
        mut,
        constraint = item.owner == player.key() @ GameError::NotItemOwner
    )]
    pub item: Account<'info, GameItem>,

    #[account(
        mut,
        seeds = [b"player", authority.key().as_ref()],
        bump
    )]
    pub player: Account<'info, PlayerAccount>,

    pub authority: Signer<'info>,
}
```

### Item Trading

```rust
pub fn transfer_item(ctx: Context<TransferItem>) -> Result<()> {
    let item = &mut ctx.accounts.item;
    let from_player = &mut ctx.accounts.from_player;
    let to_player = &mut ctx.accounts.to_player;

    // Validate
    require!(item.owner == from_player.key(), GameError::NotItemOwner);
    require!(item.is_tradeable, GameError::ItemNotTradeable);
    require!(!item.is_equipped, GameError::CannotTradeEquipped);

    // Transfer ownership
    item.owner = to_player.key();

    // Update inventories
    from_player.inventory_items.retain(|&pubkey| pubkey != item.key());
    to_player.inventory_items.push(item.key());

    msg!("Transferred {} to new owner", item.name);
    Ok(())
}

#[derive(Accounts)]
pub struct TransferItem<'info> {
    #[account(mut)]
    pub item: Account<'info, GameItem>,

    #[account(
        mut,
        seeds = [b"player", from_authority.key().as_ref()],
        bump
    )]
    pub from_player: Account<'info, PlayerAccount>,

    #[account(mut)]
    pub to_player: Account<'info, PlayerAccount>,

    pub from_authority: Signer<'info>,
}
```

### Durability System

```rust
pub fn use_item(ctx: Context<UseItem>, durability_cost: u16) -> Result<()> {
    let item = &mut ctx.accounts.item;

    require!(item.durability > 0, GameError::ItemBroken);

    // Reduce durability
    item.durability = item.durability.saturating_sub(durability_cost);

    if item.durability == 0 {
        msg!("{} has broken!", item.name);
    }

    Ok(())
}

pub fn repair_item(ctx: Context<RepairItem>) -> Result<()> {
    let item = &mut ctx.accounts.item;
    let player = &mut ctx.accounts.player;

    let max_durability = get_base_durability(item.item_type);
    let repair_needed = max_durability - item.durability;

    // Cost: 1 gold per durability point
    let repair_cost = repair_needed as u64;
    require!(player.gold >= repair_cost, GameError::InsufficientGold);

    // Pay and repair
    player.gold -= repair_cost;
    item.durability = max_durability;

    msg!("Repaired {} for {} gold", item.name, repair_cost);
    Ok(())
}
```

---

## Leaderboard System

### Leaderboard Schema

```rust
#[solana]
#[account]
struct Leaderboard {
    #[key]
    season: u32,                // Season number (1, 2, 3...)

    // Rankings (parallel arrays)
    top_players: [PublicKey],   // Player PDAs
    top_scores: [u64],          // Corresponding scores

    // Season timing
    season_start: i64,          // Unix timestamp
    season_end: i64,            // Unix timestamp

    is_active: bool,            // Currently running?
}
```

### Initialize Season

```rust
pub const MAX_LEADERBOARD_SIZE: usize = 100;
pub const SEASON_DURATION: i64 = 30 * 24 * 60 * 60; // 30 days

pub fn initialize_season(
    ctx: Context<InitializeSeason>,
    season: u32,
) -> Result<()> {
    let leaderboard = &mut ctx.accounts.leaderboard;
    let clock = Clock::get()?;

    leaderboard.season = season;
    leaderboard.top_players = vec![];
    leaderboard.top_scores = vec![];
    leaderboard.season_start = clock.unix_timestamp;
    leaderboard.season_end = clock.unix_timestamp + SEASON_DURATION;
    leaderboard.is_active = true;

    msg!("Season {} started!", season);
    Ok(())
}

#[derive(Accounts)]
#[instruction(season: u32)]
pub struct InitializeSeason<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Leaderboard::INIT_SPACE,
        seeds = [b"leaderboard", season.to_le_bytes().as_ref()],
        bump
    )]
    pub leaderboard: Account<'info, Leaderboard>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```

### Update Leaderboard

```rust
pub fn submit_score(
    ctx: Context<SubmitScore>,
    score: u64,
) -> Result<()> {
    let leaderboard = &mut ctx.accounts.leaderboard;
    let player = &ctx.accounts.player;
    let clock = Clock::get()?;

    // Validate season is active
    require!(leaderboard.is_active, GameError::SeasonEnded);
    require!(
        clock.unix_timestamp < leaderboard.season_end,
        GameError::SeasonEnded
    );

    let player_key = player.key();

    // Check if player already on leaderboard
    let existing_idx = leaderboard.top_players
        .iter()
        .position(|&p| p == player_key);

    match existing_idx {
        Some(idx) => {
            // Update if new score is higher
            if score > leaderboard.top_scores[idx] {
                leaderboard.top_scores[idx] = score;
                // Re-sort after update
                sort_leaderboard(leaderboard);
            }
        }
        None => {
            // Add new entry
            if leaderboard.top_players.len() < MAX_LEADERBOARD_SIZE {
                leaderboard.top_players.push(player_key);
                leaderboard.top_scores.push(score);
                sort_leaderboard(leaderboard);
            } else {
                // Check if score beats last place
                let last_score = *leaderboard.top_scores.last().unwrap_or(&0);
                if score > last_score {
                    // Replace last place
                    let last_idx = leaderboard.top_players.len() - 1;
                    leaderboard.top_players[last_idx] = player_key;
                    leaderboard.top_scores[last_idx] = score;
                    sort_leaderboard(leaderboard);
                }
            }
        }
    }

    Ok(())
}

fn sort_leaderboard(leaderboard: &mut Leaderboard) {
    // Create paired indices and sort by score descending
    let mut pairs: Vec<(PublicKey, u64)> = leaderboard.top_players
        .iter()
        .zip(leaderboard.top_scores.iter())
        .map(|(&p, &s)| (p, s))
        .collect();

    pairs.sort_by(|a, b| b.1.cmp(&a.1));

    leaderboard.top_players = pairs.iter().map(|(p, _)| *p).collect();
    leaderboard.top_scores = pairs.iter().map(|(_, s)| *s).collect();
}
```

### End Season & Distribute Rewards

```rust
pub fn end_season(ctx: Context<EndSeason>) -> Result<()> {
    let leaderboard = &mut ctx.accounts.leaderboard;
    let clock = Clock::get()?;

    require!(
        clock.unix_timestamp >= leaderboard.season_end,
        GameError::SeasonNotEnded
    );

    leaderboard.is_active = false;

    msg!("Season {} ended! Final rankings locked.", leaderboard.season);
    Ok(())
}

// Reward tiers
pub fn get_season_rewards(rank: usize) -> (u64, u32) {
    // Returns (gold, gems)
    match rank {
        0 => (10000, 100),      // 1st place
        1 => (7500, 75),        // 2nd place
        2 => (5000, 50),        // 3rd place
        3..=9 => (2500, 25),    // Top 10
        10..=49 => (1000, 10),  // Top 50
        50..=99 => (500, 5),    // Top 100
        _ => (0, 0),
    }
}

pub fn claim_season_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let leaderboard = &ctx.accounts.leaderboard;
    let player = &mut ctx.accounts.player;

    require!(!leaderboard.is_active, GameError::SeasonStillActive);

    // Find player's rank
    let rank = leaderboard.top_players
        .iter()
        .position(|&p| p == player.key())
        .ok_or(GameError::NotOnLeaderboard)?;

    let (gold_reward, gem_reward) = get_season_rewards(rank);

    player.gold = player.gold.saturating_add(gold_reward);
    player.gems = player.gems.saturating_add(gem_reward);

    msg!(
        "Claimed rank {} rewards: {} gold, {} gems",
        rank + 1, gold_reward, gem_reward
    );

    Ok(())
}
```

---

## Match System

### Match Result Schema

```rust
// Non-account struct - used for events/logs
#[solana]
struct MatchResult {
    player: PublicKey,              // Player who completed match
    opponent: Option<PublicKey>,    // None = PvE, Some = PvP

    score: u64,                     // Match score
    rewards_earned: u64,            // Gold earned
    experience_gained: u32,         // XP earned

    match_duration: i64,            // Seconds
    timestamp: i64,                 // When match ended
    is_victory: bool,               // Win or loss
}
```

### PvE Match (Quest)

```rust
pub fn complete_quest(
    ctx: Context<CompleteQuest>,
    quest_id: u32,
    score: u64,
    duration: i64,
) -> Result<()> {
    let player = &mut ctx.accounts.player;
    let clock = Clock::get()?;

    // Calculate rewards based on performance
    let base_gold = 50;
    let base_xp = 100;
    let score_multiplier = (score as f64 / 1000.0).min(3.0);

    let gold_reward = (base_gold as f64 * score_multiplier) as u64;
    let xp_reward = (base_xp as f64 * score_multiplier) as u64;

    // Apply rewards
    player.gold = player.gold.saturating_add(gold_reward);
    player.experience = player.experience.saturating_add(xp_reward);
    player.quests_completed += 1;

    // Check for level up
    let new_level = calculate_level(player.experience);
    if new_level > player.level {
        player.level = new_level;
        player.health += HP_PER_LEVEL;
        player.mana += MP_PER_LEVEL;
    }

    // Emit match result event
    emit!(MatchCompleted {
        player: player.key(),
        opponent: None,
        score,
        rewards_earned: gold_reward,
        experience_gained: xp_reward as u32,
        match_duration: duration,
        timestamp: clock.unix_timestamp,
        is_victory: true,
    });

    msg!(
        "Quest {} completed! +{} gold, +{} XP",
        quest_id, gold_reward, xp_reward
    );

    Ok(())
}

#[event]
pub struct MatchCompleted {
    pub player: Pubkey,
    pub opponent: Option<Pubkey>,
    pub score: u64,
    pub rewards_earned: u64,
    pub experience_gained: u32,
    pub match_duration: i64,
    pub timestamp: i64,
    pub is_victory: bool,
}
```

### PvP Match

```rust
pub fn complete_pvp_match(
    ctx: Context<CompletePvpMatch>,
    winner_score: u64,
    loser_score: u64,
    duration: i64,
) -> Result<()> {
    let winner = &mut ctx.accounts.winner;
    let loser = &mut ctx.accounts.loser;
    let clock = Clock::get()?;

    // Winner rewards
    let winner_gold = 100;
    let winner_xp = 200;
    winner.gold = winner.gold.saturating_add(winner_gold);
    winner.experience = winner.experience.saturating_add(winner_xp);

    // Loser consolation
    let loser_gold = 25;
    let loser_xp = 50;
    loser.gold = loser.gold.saturating_add(loser_gold);
    loser.experience = loser.experience.saturating_add(loser_xp);

    // Emit events for both players
    emit!(MatchCompleted {
        player: winner.key(),
        opponent: Some(loser.key()),
        score: winner_score,
        rewards_earned: winner_gold,
        experience_gained: winner_xp as u32,
        match_duration: duration,
        timestamp: clock.unix_timestamp,
        is_victory: true,
    });

    emit!(MatchCompleted {
        player: loser.key(),
        opponent: Some(winner.key()),
        score: loser_score,
        rewards_earned: loser_gold,
        experience_gained: loser_xp as u32,
        match_duration: duration,
        timestamp: clock.unix_timestamp,
        is_victory: false,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CompletePvpMatch<'info> {
    #[account(mut)]
    pub winner: Account<'info, PlayerAccount>,

    #[account(mut)]
    pub loser: Account<'info, PlayerAccount>,

    /// CHECK: Game server authority
    pub match_authority: Signer<'info>,
}
```

---

## State Management Patterns

### Atomic Multi-Account Updates

```rust
// Update player stats based on all equipped items
pub fn calculate_player_stats(ctx: Context<CalculateStats>) -> Result<()> {
    let player = &mut ctx.accounts.player;

    // Base stats
    let mut total_power: u16 = 0;
    let mut total_defense: u16 = 0;
    let mut total_speed: u16 = 0;

    // Sum equipped item stats
    // Note: In practice, you'd load remaining accounts
    for item in ctx.remaining_accounts.iter() {
        let item_data = Account::<GameItem>::try_from(item)?;
        if item_data.is_equipped && item_data.owner == player.key() {
            total_power = total_power.saturating_add(item_data.power);
            total_defense = total_defense.saturating_add(item_data.defense);
            total_speed = total_speed.saturating_add(item_data.speed);
        }
    }

    // Store calculated stats (you might add these fields)
    msg!(
        "Total stats - Power: {}, Defense: {}, Speed: {}",
        total_power, total_defense, total_speed
    );

    Ok(())
}
```

### Cross-Account Validation

```rust
// Ensure item belongs to player before any operation
fn validate_item_ownership(
    item: &Account<GameItem>,
    player: &Account<PlayerAccount>,
) -> Result<()> {
    require!(
        item.owner == player.key(),
        GameError::NotItemOwner
    );
    require!(
        player.inventory_items.contains(&item.key()) ||
        player.equipped_items.contains(&item.key()),
        GameError::ItemNotInInventory
    );
    Ok(())
}
```

### Event Patterns for Indexing

```rust
// Emit events for off-chain indexing
#[event]
pub struct PlayerCreated {
    pub player: Pubkey,
    pub wallet: Pubkey,
    pub username: String,
    pub timestamp: i64,
}

#[event]
pub struct ItemMinted {
    pub item: Pubkey,
    pub owner: Pubkey,
    pub name: String,
    pub rarity: u8,
    pub timestamp: i64,
}

#[event]
pub struct ItemTransferred {
    pub item: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct LevelUp {
    pub player: Pubkey,
    pub old_level: u16,
    pub new_level: u16,
    pub timestamp: i64,
}
```

---

## Performance Considerations

### Account Size & Rent

```rust
// Minimize account sizes to reduce rent

// Bad: Storing full strings
struct BadItem {
    description: String,  // Unbounded!
    lore: String,         // Unbounded!
}

// Good: Use max constraints and off-chain storage
#[solana]
#[account]
struct GoodItem {
    #[max(50)]
    name: String,         // Bounded
    metadata_uri: String, // Link to off-chain data
}

// Calculate rent
// ~0.00089 SOL per byte per year
// PlayerAccount (~2500 bytes) ≈ 0.017 SOL rent-exempt
```

### Compute Unit Budgeting

```rust
// Default CU limit: 200,000 per instruction
// Complex operations may need more

// Request additional compute units
use anchor_lang::solana_program::compute_budget::ComputeBudgetInstruction;

// In client code:
let increase_cu_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);
```

### Batch Operations

```rust
// Instead of multiple transactions, batch updates
pub fn batch_gain_experience(
    ctx: Context<BatchGainExperience>,
    amounts: Vec<u64>,
) -> Result<()> {
    require!(
        ctx.remaining_accounts.len() == amounts.len(),
        GameError::InvalidBatchSize
    );

    for (i, account) in ctx.remaining_accounts.iter().enumerate() {
        let mut player = Account::<PlayerAccount>::try_from(account)?;
        player.experience = player.experience.saturating_add(amounts[i]);
        // Note: Would need to serialize back
    }

    Ok(())
}
```

### Off-Chain Data Strategy

```markdown
**On-Chain (essential):**
- Player wallet, level, currency balances
- Item ownership, equipped status
- Leaderboard rankings

**Off-Chain (optional):**
- Item images, descriptions, lore
- Detailed match history
- Chat messages
- Achievement descriptions

**Indexing Solutions:**
- Helius webhooks
- Yellowstone gRPC
- Custom Geyser plugin
- The Graph (subgraphs)
```

### Transaction Size Limits

```rust
// Max transaction size: 1232 bytes
// Max accounts per transaction: ~35 (varies)

// For large inventory operations, paginate:
pub fn get_inventory_page(
    ctx: Context<GetInventory>,
    page: u32,
    page_size: u32,
) -> Result<Vec<Pubkey>> {
    let player = &ctx.accounts.player;
    let start = (page * page_size) as usize;
    let end = start + page_size as usize;

    Ok(player.inventory_items
        .get(start..end.min(player.inventory_items.len()))
        .unwrap_or(&[])
        .to_vec())
}
```

---

## Complete Implementation

### Full Program Structure

```rust
// programs/gaming/src/lib.rs

use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;

use instructions::*;

declare_id!("GameXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

#[program]
pub mod gaming {
    use super::*;

    // === Player Instructions ===

    pub fn create_player(
        ctx: Context<CreatePlayer>,
        username: String,
    ) -> Result<()> {
        instructions::player::create_player(ctx, username)
    }

    pub fn gain_experience(
        ctx: Context<GainExperience>,
        amount: u64,
    ) -> Result<()> {
        instructions::player::gain_experience(ctx, amount)
    }

    pub fn update_playtime(
        ctx: Context<UpdatePlaytime>,
        seconds: i64,
    ) -> Result<()> {
        instructions::player::update_playtime(ctx, seconds)
    }

    // === Item Instructions ===

    pub fn mint_item(
        ctx: Context<MintItem>,
        item_id: u64,
        name: String,
        item_type: u8,
        rarity: u8,
        base_power: u16,
        base_defense: u16,
        base_speed: u16,
    ) -> Result<()> {
        instructions::item::mint_item(
            ctx, item_id, name, item_type, rarity,
            base_power, base_defense, base_speed
        )
    }

    pub fn equip_item(ctx: Context<EquipItem>) -> Result<()> {
        instructions::item::equip_item(ctx)
    }

    pub fn unequip_item(ctx: Context<UnequipItem>) -> Result<()> {
        instructions::item::unequip_item(ctx)
    }

    pub fn transfer_item(ctx: Context<TransferItem>) -> Result<()> {
        instructions::item::transfer_item(ctx)
    }

    pub fn repair_item(ctx: Context<RepairItem>) -> Result<()> {
        instructions::item::repair_item(ctx)
    }

    // === Match Instructions ===

    pub fn complete_quest(
        ctx: Context<CompleteQuest>,
        quest_id: u32,
        score: u64,
        duration: i64,
    ) -> Result<()> {
        instructions::match_system::complete_quest(ctx, quest_id, score, duration)
    }

    pub fn complete_pvp_match(
        ctx: Context<CompletePvpMatch>,
        winner_score: u64,
        loser_score: u64,
        duration: i64,
    ) -> Result<()> {
        instructions::match_system::complete_pvp_match(
            ctx, winner_score, loser_score, duration
        )
    }

    // === Leaderboard Instructions ===

    pub fn initialize_season(
        ctx: Context<InitializeSeason>,
        season: u32,
    ) -> Result<()> {
        instructions::leaderboard::initialize_season(ctx, season)
    }

    pub fn submit_score(
        ctx: Context<SubmitScore>,
        score: u64,
    ) -> Result<()> {
        instructions::leaderboard::submit_score(ctx, score)
    }

    pub fn end_season(ctx: Context<EndSeason>) -> Result<()> {
        instructions::leaderboard::end_season(ctx)
    }

    pub fn claim_season_rewards(
        ctx: Context<ClaimRewards>,
    ) -> Result<()> {
        instructions::leaderboard::claim_season_rewards(ctx)
    }
}
```

### Error Definitions

```rust
// programs/gaming/src/errors.rs

use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("Username must be 3-20 characters")]
    UsernameTooShort,

    #[msg("Username must be 3-20 characters")]
    UsernameTooLong,

    #[msg("Item name must be 1-50 characters")]
    NameTooLong,

    #[msg("Invalid item type")]
    InvalidItemType,

    #[msg("Invalid rarity level")]
    InvalidRarity,

    #[msg("You don't own this item")]
    NotItemOwner,

    #[msg("Item is already equipped")]
    AlreadyEquipped,

    #[msg("Item is not equipped")]
    NotEquipped,

    #[msg("Item durability is zero")]
    ItemBroken,

    #[msg("All equipment slots are full")]
    EquipmentSlotsFull,

    #[msg("Cannot trade equipped items")]
    CannotTradeEquipped,

    #[msg("This item cannot be traded")]
    ItemNotTradeable,

    #[msg("Item not found in inventory")]
    ItemNotInInventory,

    #[msg("Insufficient gold")]
    InsufficientGold,

    #[msg("Insufficient gems")]
    InsufficientGems,

    #[msg("Season has ended")]
    SeasonEnded,

    #[msg("Season has not ended yet")]
    SeasonNotEnded,

    #[msg("Season is still active")]
    SeasonStillActive,

    #[msg("Player not on leaderboard")]
    NotOnLeaderboard,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Invalid batch size")]
    InvalidBatchSize,
}
```

---

## Frontend Integration

### React Hooks

```typescript
// src/hooks/usePlayer.ts
import { useCallback, useEffect, useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PlayerAccount, PlayerAccountSchema } from '../generated';
import { PROGRAM_ID } from '../constants';

export function usePlayer() {
  const { connection } = useConnection();
  const { publicKey } = useWallet();
  const [player, setPlayer] = useState<PlayerAccount | null>(null);
  const [loading, setLoading] = useState(true);

  const playerPda = publicKey
    ? PublicKey.findProgramAddressSync(
        [Buffer.from('player'), publicKey.toBuffer()],
        PROGRAM_ID
      )[0]
    : null;

  const fetchPlayer = useCallback(async () => {
    if (!playerPda) {
      setPlayer(null);
      setLoading(false);
      return;
    }

    try {
      const accountInfo = await connection.getAccountInfo(playerPda);
      if (accountInfo) {
        const data = accountInfo.data.slice(8);
        setPlayer(PlayerAccountSchema.decode(Buffer.from(data)));
      } else {
        setPlayer(null);
      }
    } catch (err) {
      console.error('Failed to fetch player:', err);
    } finally {
      setLoading(false);
    }
  }, [connection, playerPda]);

  useEffect(() => {
    fetchPlayer();
  }, [fetchPlayer]);

  // Subscribe to changes
  useEffect(() => {
    if (!playerPda) return;

    const subscriptionId = connection.onAccountChange(
      playerPda,
      (accountInfo) => {
        const data = accountInfo.data.slice(8);
        setPlayer(PlayerAccountSchema.decode(Buffer.from(data)));
      }
    );

    return () => {
      connection.removeAccountChangeListener(subscriptionId);
    };
  }, [connection, playerPda]);

  return { player, playerPda, loading, refetch: fetchPlayer };
}
```

### Inventory Hook

```typescript
// src/hooks/useInventory.ts
import { useEffect, useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { GameItem, GameItemSchema } from '../generated';

export function useInventory(itemPubkeys: PublicKey[]) {
  const { connection } = useConnection();
  const [items, setItems] = useState<GameItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchItems() {
      if (itemPubkeys.length === 0) {
        setItems([]);
        setLoading(false);
        return;
      }

      try {
        const accounts = await connection.getMultipleAccountsInfo(itemPubkeys);
        const decoded = accounts
          .filter((acc) => acc !== null)
          .map((acc) => {
            const data = acc!.data.slice(8);
            return GameItemSchema.decode(Buffer.from(data));
          });
        setItems(decoded);
      } catch (err) {
        console.error('Failed to fetch inventory:', err);
      } finally {
        setLoading(false);
      }
    }

    fetchItems();
  }, [connection, itemPubkeys]);

  return { items, loading };
}
```

### Game UI Components

```typescript
// src/components/PlayerStats.tsx
import { PlayerAccount } from '../generated';

interface PlayerStatsProps {
  player: PlayerAccount;
}

export function PlayerStats({ player }: PlayerStatsProps) {
  const xpToNextLevel = ((player.level) * 1000) - player.experience;
  const xpProgress = (player.experience % 1000) / 10;

  return (
    <div className="player-stats">
      <div className="stat-header">
        <h2>{player.username}</h2>
        <span className="level">Lv. {player.level}</span>
      </div>

      <div className="stat-bars">
        <div className="stat-bar health">
          <label>HP</label>
          <div className="bar">
            <div
              className="fill"
              style={{ width: `${(player.health / (100 + player.level * 10)) * 100}%` }}
            />
          </div>
          <span>{player.health}</span>
        </div>

        <div className="stat-bar mana">
          <label>MP</label>
          <div className="bar">
            <div
              className="fill"
              style={{ width: `${(player.mana / (50 + player.level * 5)) * 100}%` }}
            />
          </div>
          <span>{player.mana}</span>
        </div>

        <div className="stat-bar xp">
          <label>XP</label>
          <div className="bar">
            <div className="fill" style={{ width: `${xpProgress}%` }} />
          </div>
          <span>{xpToNextLevel} to next level</span>
        </div>
      </div>

      <div className="currency">
        <div className="gold">
          <span className="icon">🪙</span>
          <span>{player.gold.toLocaleString()}</span>
        </div>
        <div className="gems">
          <span className="icon">💎</span>
          <span>{player.gems.toLocaleString()}</span>
        </div>
      </div>

      <div className="progress">
        <div>Quests: {player.quests_completed}</div>
        <div>Achievements: {player.achievements.length}</div>
        <div>Playtime: {Math.floor(player.total_playtime / 3600)}h</div>
      </div>
    </div>
  );
}
```

### Item Card Component

```typescript
// src/components/ItemCard.tsx
import { GameItem } from '../generated';

const RARITY_COLORS = {
  0: '#9d9d9d', // Common - Gray
  1: '#1eff00', // Uncommon - Green
  2: '#0070dd', // Rare - Blue
  3: '#a335ee', // Epic - Purple
  4: '#ff8000', // Legendary - Orange
};

const RARITY_NAMES = ['Common', 'Uncommon', 'Rare', 'Epic', 'Legendary'];

interface ItemCardProps {
  item: GameItem;
  onEquip?: () => void;
  onUnequip?: () => void;
}

export function ItemCard({ item, onEquip, onUnequip }: ItemCardProps) {
  const rarityColor = RARITY_COLORS[item.rarity] || RARITY_COLORS[0];
  const durabilityPercent = (item.durability / 100) * 100;

  return (
    <div
      className="item-card"
      style={{ borderColor: rarityColor }}
    >
      <div className="item-header">
        <h3 style={{ color: rarityColor }}>{item.name}</h3>
        <span className="rarity">{RARITY_NAMES[item.rarity]}</span>
      </div>

      <div className="item-stats">
        {item.power > 0 && <div>⚔️ Power: {item.power}</div>}
        {item.defense > 0 && <div>🛡️ Defense: {item.defense}</div>}
        {item.speed > 0 && <div>⚡ Speed: {item.speed}</div>}
      </div>

      <div className="durability">
        <div
          className="durability-bar"
          style={{
            width: `${durabilityPercent}%`,
            backgroundColor: durabilityPercent > 50 ? 'green' :
                           durabilityPercent > 20 ? 'yellow' : 'red'
          }}
        />
        <span>{item.durability}%</span>
      </div>

      <div className="item-actions">
        {item.is_equipped ? (
          <button onClick={onUnequip}>Unequip</button>
        ) : (
          <button onClick={onEquip}>Equip</button>
        )}
      </div>

      {item.durability === 0 && (
        <div className="broken-overlay">BROKEN</div>
      )}
    </div>
  );
}
```

### Leaderboard Component

```typescript
// src/components/SeasonLeaderboard.tsx
import { useLeaderboard } from '../hooks/useLeaderboard';

interface LeaderboardProps {
  season: number;
}

export function SeasonLeaderboard({ season }: LeaderboardProps) {
  const { leaderboard, loading } = useLeaderboard(season);

  if (loading) return <div>Loading leaderboard...</div>;
  if (!leaderboard) return <div>No leaderboard found</div>;

  const entries = leaderboard.top_players.map((player, index) => ({
    rank: index + 1,
    player: player.toString(),
    score: leaderboard.top_scores[index] || 0,
  }));

  return (
    <div className="leaderboard">
      <div className="leaderboard-header">
        <h2>Season {season}</h2>
        {leaderboard.is_active ? (
          <span className="badge active">Active</span>
        ) : (
          <span className="badge ended">Ended</span>
        )}
      </div>

      <table>
        <thead>
          <tr>
            <th>Rank</th>
            <th>Player</th>
            <th>Score</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((entry) => (
            <tr
              key={entry.rank}
              className={entry.rank <= 3 ? `top-${entry.rank}` : ''}
            >
              <td>
                {entry.rank === 1 && '🥇'}
                {entry.rank === 2 && '🥈'}
                {entry.rank === 3 && '🥉'}
                {entry.rank > 3 && `#${entry.rank}`}
              </td>
              <td>{entry.player.slice(0, 8)}...</td>
              <td>{entry.score.toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>

      {leaderboard.is_active && (
        <div className="season-timer">
          Ends: {new Date(leaderboard.season_end * 1000).toLocaleDateString()}
        </div>
      )}
    </div>
  );
}
```

---

## Resources

### Example Code

- **LUMOS Gaming Schema:** [`examples/gaming/schema.lumos`](../../../examples/gaming/schema.lumos)
- **awesome-lumos Gaming:** [github.com/getlumos/awesome-lumos](https://github.com/getlumos/awesome-lumos)

### Related Guides

**Integration Guides:**
- [LUMOS + Anchor Integration](/guides/anchor-integration) - Build new game programs
- [Solana CLI Integration](/guides/solana-cli-integration) - Deploy your game
- [web3.js Integration](/guides/web3js-integration) - Frontend patterns

**Other Use Cases:**
- [NFT Marketplaces](/guides/use-cases/nft) - In-game item trading
- [DeFi Protocols](/guides/use-cases/defi) - Token economics

**Migration Guides:**
- [Migration: TypeScript → LUMOS](/guides/migration-typescript) - Migrate existing types
- [Migration: Anchor → LUMOS](/guides/migration-anchor) - Add LUMOS to existing games

### External Resources

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Cookbook - Gaming](https://solanacookbook.com/gaming/intro.html)
- [Metaplex (for NFT items)](https://docs.metaplex.com/)

### Community

- [LUMOS Discord](#) - Get help from the community
- [Solana Gaming Discord](https://discord.gg/solana) - #gaming channel

---

## Next Steps

1. **Generate your schema:** `lumos generate schemas/gaming.lumos`
2. **Build the program:** `anchor build`
3. **Test locally:** `solana-test-validator && anchor test`
4. **Deploy to devnet:** Follow [Solana CLI guide](../solana-cli-integration.md)
5. **Build your frontend:** Use patterns from [web3.js guide](../web3js-integration.md)

---

**Last Updated:** 2025-12-16
**Version:** 1.0.0
