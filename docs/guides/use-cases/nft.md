# NFT Projects with LUMOS

**Purpose:** Complete guide for building NFT marketplaces and collections with LUMOS on Solana

**Last Updated:** 2025-12-16

---

## Overview

This guide walks through building a complete NFT marketplace using LUMOS, covering:

- Marketplace configuration with fees and royalties
- Metaplex-compatible metadata structures
- NFT minting workflows
- Listing and trading patterns
- Creator royalties and fee distribution

**What we'll build:**

```
┌─────────────────────────────────────────────────────────────┐
│                    NFT Marketplace                          │
├─────────────────────────────────────────────────────────────┤
│  • Marketplace Config (fees, authority, volume tracking)   │
│  • NFT Metadata (Metaplex-compatible, creators, royalties) │
│  • Listing System (price, expiration, cancellation)        │
│  • Trading Engine (buy, sell, offers)                      │
│  • Receipt Tracking (full transaction audit trail)         │
└─────────────────────────────────────────────────────────────┘
```

---

## Table of Contents

1. [Why LUMOS for NFTs](#why-lumos-for-nfts)
2. [NFT Architecture](#nft-architecture)
3. [Marketplace Configuration](#marketplace-configuration)
4. [NFT Metadata & Metaplex](#nft-metadata--metaplex)
5. [Listing System](#listing-system)
6. [Minting Workflow](#minting-workflow)
7. [Trading & Purchases](#trading--purchases)
8. [Royalties & Creator Fees](#royalties--creator-fees)
9. [Best Practices](#best-practices)
10. [Complete Implementation](#complete-implementation)
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

**Additional NFT tools:**
```bash
# Metaplex CLI (optional, for testing)
npm install -g @metaplex-foundation/cli
```

---

## Why LUMOS for NFTs

### The NFT Challenge

NFT projects require:
- **Metaplex compatibility** - Industry standard metadata
- **Precise fee calculations** - Royalties, marketplace fees
- **Complex relationships** - Mints, metadata, listings, receipts
- **Audit trails** - Transaction signatures for provenance

### The LUMOS Solution

| Benefit | Impact on NFTs |
|---------|----------------|
| `#[max()]` constraints | Enforce Metaplex limits at schema level |
| `Signature` type | Native transaction signature handling |
| Basis points support | Precise royalty calculations |
| Generated validators | Auto-validate metadata before mint |

### Metaplex Integration

LUMOS includes built-in Metaplex support:

```bash
# Validate schema against Metaplex standards
lumos metaplex validate schema.lumos

# Generate Metaplex-compatible code
lumos metaplex generate schema.lumos

# Show standard Metaplex type definitions
lumos metaplex types --format lumos
```

---

## NFT Architecture

### Account Model

```
┌─────────────────┐         ┌─────────────────┐
│   Marketplace   │         │   NftMetadata   │
│     (PDA)       │         │     (PDA)       │
│                 │         │                 │
│ • authority     │         │ • mint (key)    │
│ • fee_recipient │         │ • name, symbol  │
│ • fee_bps       │         │ • uri           │
│ • total_volume  │         │ • creators[]    │
│ • is_active     │         │ • royalties     │
└────────┬────────┘         └────────┬────────┘
         │                           │
         │ manages                   │ describes
         ▼                           ▼
┌─────────────────┐         ┌─────────────────┐
│   NftListing    │◀────────│   NFT Mint      │
│     (PDA)       │  lists  │  (SPL Token)    │
│                 │         │                 │
│ • seller        │         │ • supply = 1    │
│ • nft_mint (key)│         │ • decimals = 0  │
│ • price         │         └─────────────────┘
│ • expires_at    │
└────────┬────────┘
         │
         │ generates
         ▼
┌─────────────────┐
│ PurchaseReceipt │
│   (event data)  │
│                 │
│ • buyer/seller  │
│ • price         │
│ • fees          │
│ • tx_signature  │
└─────────────────┘
```

### PDA Derivation Strategy

```rust
// Marketplace PDA - one per authority
seeds = [b"marketplace", authority.as_ref()]

// NFT Metadata PDA - one per mint
seeds = [b"metadata", mint.as_ref()]

// Listing PDA - one per listed NFT
seeds = [b"listing", nft_mint.as_ref()]

// Escrow PDA - holds NFT during listing
seeds = [b"escrow", nft_mint.as_ref()]
```

### Program Structure

```
programs/nft-marketplace/
├── src/
│   ├── lib.rs              # Program entry point
│   ├── state/
│   │   ├── mod.rs
│   │   ├── marketplace.rs  # Marketplace (from LUMOS)
│   │   ├── metadata.rs     # NftMetadata (from LUMOS)
│   │   ├── listing.rs      # NftListing (from LUMOS)
│   │   └── receipt.rs      # PurchaseReceipt (from LUMOS)
│   ├── instructions/
│   │   ├── mod.rs
│   │   ├── marketplace.rs  # initialize, update, pause
│   │   ├── mint.rs         # mint_nft, update_metadata
│   │   ├── listing.rs      # list, update, cancel
│   │   └── trade.rs        # buy, make_offer, accept_offer
│   ├── utils/
│   │   └── fees.rs         # Fee calculation helpers
│   └── errors.rs           # Custom errors
└── Cargo.toml
```

---

## Marketplace Configuration

### Complete Schema

```rust
// schemas/nft-marketplace.lumos

#[solana]
#[account]
struct Marketplace {
    // === Authority ===
    #[key]
    authority: PublicKey,       // Owner who can update config
    fee_recipient: PublicKey,   // Address receiving marketplace fees

    // === Fee Structure ===
    fee_basis_points: u16,      // Marketplace fee (250 = 2.5%)

    // === Statistics ===
    total_volume: u64,          // Cumulative SOL traded (lamports)
    total_sales: u64,           // Total successful transactions

    // === Status ===
    is_active: bool,            // Can pause marketplace
}
```

### Generate Code

```bash
lumos generate schemas/nft-marketplace.lumos --output programs/nft-marketplace/src/state/
```

### Understanding Basis Points

```rust
// Basis points provide precise percentage calculations
// 1 basis point = 0.01%
// 100 basis points = 1%
// 10000 basis points = 100%

pub const MAX_FEE_BASIS_POINTS: u16 = 1000;  // Max 10% fee

// Examples:
// 250 basis points = 2.5% marketplace fee
// 500 basis points = 5% royalty fee

fn calculate_fee(price: u64, basis_points: u16) -> u64 {
    ((price as u128) * (basis_points as u128) / 10000) as u64
}
```

### Initialize Marketplace

```rust
pub fn initialize_marketplace(
    ctx: Context<InitializeMarketplace>,
    fee_basis_points: u16,
) -> Result<()> {
    require!(
        fee_basis_points <= MAX_FEE_BASIS_POINTS,
        MarketplaceError::FeeTooHigh
    );

    let marketplace = &mut ctx.accounts.marketplace;

    marketplace.authority = ctx.accounts.authority.key();
    marketplace.fee_recipient = ctx.accounts.fee_recipient.key();
    marketplace.fee_basis_points = fee_basis_points;
    marketplace.total_volume = 0;
    marketplace.total_sales = 0;
    marketplace.is_active = true;

    msg!("Marketplace initialized with {}bp fee", fee_basis_points);
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Marketplace::INIT_SPACE,
        seeds = [b"marketplace", authority.key().as_ref()],
        bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Fee recipient can be any address
    pub fee_recipient: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
```

### Update & Pause Marketplace

```rust
pub fn update_marketplace(
    ctx: Context<UpdateMarketplace>,
    new_fee_basis_points: Option<u16>,
    new_fee_recipient: Option<Pubkey>,
) -> Result<()> {
    let marketplace = &mut ctx.accounts.marketplace;

    if let Some(fee_bps) = new_fee_basis_points {
        require!(fee_bps <= MAX_FEE_BASIS_POINTS, MarketplaceError::FeeTooHigh);
        marketplace.fee_basis_points = fee_bps;
    }

    if let Some(recipient) = new_fee_recipient {
        marketplace.fee_recipient = recipient;
    }

    Ok(())
}

pub fn pause_marketplace(ctx: Context<UpdateMarketplace>) -> Result<()> {
    ctx.accounts.marketplace.is_active = false;
    msg!("Marketplace paused");
    Ok(())
}

pub fn resume_marketplace(ctx: Context<UpdateMarketplace>) -> Result<()> {
    ctx.accounts.marketplace.is_active = true;
    msg!("Marketplace resumed");
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateMarketplace<'info> {
    #[account(
        mut,
        seeds = [b"marketplace", authority.key().as_ref()],
        bump,
        has_one = authority @ MarketplaceError::Unauthorized
    )]
    pub marketplace: Account<'info, Marketplace>,

    pub authority: Signer<'info>,
}
```

---

## NFT Metadata & Metaplex

### Metaplex-Compatible Schema

```rust
#[solana]
#[account]
struct NftMetadata {
    // === Identity ===
    #[key]
    mint: PublicKey,            // NFT mint address

    // === Display Info (Metaplex Constraints) ===
    #[max(32)]                  // Metaplex: MAX_NAME_LENGTH
    name: String,

    #[max(10)]                  // Metaplex: MAX_SYMBOL_LENGTH
    symbol: String,

    #[max(200)]                 // Metaplex: MAX_URI_LENGTH
    uri: String,                // Points to JSON metadata

    // === Royalties ===
    seller_fee_basis_points: u16,  // 0-10000 (0-100%)

    // === Creators ===
    creators: [PublicKey],      // Creator addresses (max 5)

    // === Mutability ===
    is_mutable: bool,           // Can metadata be updated?
}
```

### Metaplex Constraints

```rust
// packages/core/src/metaplex/types.rs

pub mod constraints {
    pub const MAX_NAME_LENGTH: usize = 32;
    pub const MAX_SYMBOL_LENGTH: usize = 10;
    pub const MAX_URI_LENGTH: usize = 200;
    pub const MAX_CREATORS: usize = 5;
    pub const MAX_SELLER_FEE_BASIS_POINTS: u16 = 10000;
    pub const CREATOR_SHARES_TOTAL: u8 = 100;
}
```

### Validate with CLI

```bash
# Validate your schema against Metaplex standards
lumos metaplex validate schemas/nft-marketplace.lumos

# Output:
# ✓ NftMetadata.name: max length 32 (Metaplex compliant)
# ✓ NftMetadata.symbol: max length 10 (Metaplex compliant)
# ✓ NftMetadata.uri: max length 200 (Metaplex compliant)
# ✓ NftMetadata.seller_fee_basis_points: u16 (valid range)
# ✓ All constraints validated successfully
```

### Extended Creator Structure

For full Metaplex compatibility with verified creators and shares:

```rust
// Extended schema with creator details
#[solana]
struct Creator {
    address: PublicKey,
    verified: bool,             // Has creator signed?
    share: u8,                  // Percentage of royalties (0-100)
}

#[solana]
#[account]
struct NftMetadataExtended {
    #[key]
    mint: PublicKey,

    #[max(32)]
    name: String,

    #[max(10)]
    symbol: String,

    #[max(200)]
    uri: String,

    seller_fee_basis_points: u16,

    // Full creator info with shares
    creators: [Creator],        // Shares must sum to 100

    is_mutable: bool,

    // Optional collection
    collection: Option<PublicKey>,
    collection_verified: bool,

    // Token standard
    token_standard: u8,         // 0=NFT, 1=FungibleAsset, etc.
}
```

### Metadata JSON Structure

The `uri` field points to off-chain JSON:

```json
{
  "name": "Cool NFT #1",
  "symbol": "COOL",
  "description": "A very cool NFT from our collection",
  "image": "https://arweave.net/abc123",
  "animation_url": "https://arweave.net/xyz789",
  "external_url": "https://mycollection.com/nft/1",
  "attributes": [
    {
      "trait_type": "Background",
      "value": "Blue"
    },
    {
      "trait_type": "Rarity",
      "value": "Legendary"
    }
  ],
  "properties": {
    "files": [
      {
        "uri": "https://arweave.net/abc123",
        "type": "image/png"
      }
    ],
    "category": "image",
    "creators": [
      {
        "address": "Creator1PublicKey...",
        "share": 100
      }
    ]
  }
}
```

---

## Listing System

### Listing Schema

```rust
#[solana]
#[account]
struct NftListing {
    // === Seller Info ===
    seller: PublicKey,          // Who listed the NFT

    // === NFT Info ===
    #[key]
    nft_mint: PublicKey,        // The listed NFT's mint

    // === Pricing ===
    price: u64,                 // Price in lamports

    // === Timing ===
    created_at: i64,            // When listed
    expires_at: Option<i64>,    // Optional auto-expiration
}
```

### Create Listing

```rust
pub fn list_nft(
    ctx: Context<ListNft>,
    price: u64,
    expires_at: Option<i64>,
) -> Result<()> {
    let marketplace = &ctx.accounts.marketplace;
    let listing = &mut ctx.accounts.listing;
    let clock = Clock::get()?;

    // Validate marketplace is active
    require!(marketplace.is_active, MarketplaceError::MarketplacePaused);

    // Validate price
    require!(price > 0, MarketplaceError::InvalidPrice);

    // Validate expiration if provided
    if let Some(expiry) = expires_at {
        require!(expiry > clock.unix_timestamp, MarketplaceError::InvalidExpiration);
    }

    // Initialize listing
    listing.seller = ctx.accounts.seller.key();
    listing.nft_mint = ctx.accounts.nft_mint.key();
    listing.price = price;
    listing.created_at = clock.unix_timestamp;
    listing.expires_at = expires_at;

    // Transfer NFT to escrow
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.seller_token_account.to_account_info(),
            to: ctx.accounts.escrow_token_account.to_account_info(),
            authority: ctx.accounts.seller.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, 1)?;

    msg!("NFT listed for {} lamports", price);
    Ok(())
}

#[derive(Accounts)]
pub struct ListNft<'info> {
    #[account(
        seeds = [b"marketplace", marketplace.authority.as_ref()],
        bump
    )]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        init,
        payer = seller,
        space = 8 + NftListing::INIT_SPACE,
        seeds = [b"listing", nft_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, NftListing>,

    pub nft_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = seller
    )]
    pub seller_token_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = seller,
        associated_token::mint = nft_mint,
        associated_token::authority = escrow_authority
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for escrow
    #[account(seeds = [b"escrow", nft_mint.key().as_ref()], bump)]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
```

### Update Listing Price

```rust
pub fn update_listing(
    ctx: Context<UpdateListing>,
    new_price: u64,
) -> Result<()> {
    require!(new_price > 0, MarketplaceError::InvalidPrice);

    let listing = &mut ctx.accounts.listing;
    listing.price = new_price;

    msg!("Listing price updated to {} lamports", new_price);
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateListing<'info> {
    #[account(
        mut,
        seeds = [b"listing", listing.nft_mint.as_ref()],
        bump,
        has_one = seller @ MarketplaceError::Unauthorized
    )]
    pub listing: Account<'info, NftListing>,

    pub seller: Signer<'info>,
}
```

### Cancel Listing

```rust
pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
    // Transfer NFT back to seller
    let nft_mint = ctx.accounts.listing.nft_mint;
    let seeds = &[b"escrow", nft_mint.as_ref(), &[ctx.bumps.escrow_authority]];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.seller_token_account.to_account_info(),
            authority: ctx.accounts.escrow_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_ctx, 1)?;

    msg!("Listing cancelled");
    Ok(())
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(
        mut,
        close = seller,
        seeds = [b"listing", listing.nft_mint.as_ref()],
        bump,
        has_one = seller @ MarketplaceError::Unauthorized
    )]
    pub listing: Account<'info, NftListing>,

    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub seller_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(seeds = [b"escrow", listing.nft_mint.as_ref()], bump)]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub seller: Signer<'info>,

    pub token_program: Program<'info, Token>,
}
```

### Clean Expired Listings

```rust
pub fn delist_expired(ctx: Context<DelistExpired>) -> Result<()> {
    let listing = &ctx.accounts.listing;
    let clock = Clock::get()?;

    // Check if listing has expired
    if let Some(expires_at) = listing.expires_at {
        require!(
            clock.unix_timestamp >= expires_at,
            MarketplaceError::ListingNotExpired
        );
    } else {
        return err!(MarketplaceError::ListingHasNoExpiration);
    }

    // Transfer NFT back to seller (same as cancel)
    // ... transfer logic ...

    msg!("Expired listing removed");
    Ok(())
}
```

---

## Minting Workflow

### Complete Mint Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    NFT Minting Flow                         │
├─────────────────────────────────────────────────────────────┤
│  1. Create Mint Account (SPL Token, supply=0, decimals=0)  │
│  2. Create Associated Token Account for creator            │
│  3. Create NftMetadata account with LUMOS schema           │
│  4. Mint 1 token to creator's ATA                          │
│  5. (Optional) Create Master Edition for prints            │
│  6. (Optional) Verify creators                             │
└─────────────────────────────────────────────────────────────┘
```

### Mint NFT Instruction

```rust
pub fn mint_nft(
    ctx: Context<MintNft>,
    name: String,
    symbol: String,
    uri: String,
    seller_fee_basis_points: u16,
) -> Result<()> {
    // Validate Metaplex constraints
    require!(name.len() <= 32, MarketplaceError::NameTooLong);
    require!(symbol.len() <= 10, MarketplaceError::SymbolTooLong);
    require!(uri.len() <= 200, MarketplaceError::UriTooLong);
    require!(
        seller_fee_basis_points <= 10000,
        MarketplaceError::RoyaltyTooHigh
    );

    let clock = Clock::get()?;

    // Initialize metadata
    let metadata = &mut ctx.accounts.metadata;
    metadata.mint = ctx.accounts.mint.key();
    metadata.name = name.clone();
    metadata.symbol = symbol.clone();
    metadata.uri = uri;
    metadata.seller_fee_basis_points = seller_fee_basis_points;
    metadata.creators = vec![ctx.accounts.creator.key()];
    metadata.is_mutable = true;

    // Mint 1 token to creator
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.creator_token_account.to_account_info(),
            authority: ctx.accounts.creator.to_account_info(),
        },
    );
    token::mint_to(cpi_ctx, 1)?;

    // Disable further minting (NFT = supply of 1)
    let freeze_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        SetAuthority {
            current_authority: ctx.accounts.creator.to_account_info(),
            account_or_mint: ctx.accounts.mint.to_account_info(),
        },
    );
    token::set_authority(
        freeze_ctx,
        AuthorityType::MintTokens,
        None,  // Remove mint authority
    )?;

    emit!(NftMinted {
        mint: ctx.accounts.mint.key(),
        creator: ctx.accounts.creator.key(),
        name,
        symbol,
        timestamp: clock.unix_timestamp,
    });

    msg!("NFT minted: {}", metadata.name);
    Ok(())
}

#[event]
pub struct NftMinted {
    pub mint: Pubkey,
    pub creator: Pubkey,
    pub name: String,
    pub symbol: String,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(
        init,
        payer = creator,
        mint::decimals = 0,
        mint::authority = creator,
        mint::freeze_authority = creator
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = creator,
        space = 8 + NftMetadata::INIT_SPACE,
        seeds = [b"metadata", mint.key().as_ref()],
        bump
    )]
    pub metadata: Account<'info, NftMetadata>,

    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
```

### Update Metadata

```rust
pub fn update_metadata(
    ctx: Context<UpdateMetadata>,
    name: Option<String>,
    symbol: Option<String>,
    uri: Option<String>,
) -> Result<()> {
    let metadata = &mut ctx.accounts.metadata;

    require!(metadata.is_mutable, MarketplaceError::MetadataImmutable);

    if let Some(new_name) = name {
        require!(new_name.len() <= 32, MarketplaceError::NameTooLong);
        metadata.name = new_name;
    }

    if let Some(new_symbol) = symbol {
        require!(new_symbol.len() <= 10, MarketplaceError::SymbolTooLong);
        metadata.symbol = new_symbol;
    }

    if let Some(new_uri) = uri {
        require!(new_uri.len() <= 200, MarketplaceError::UriTooLong);
        metadata.uri = new_uri;
    }

    msg!("Metadata updated");
    Ok(())
}

pub fn make_immutable(ctx: Context<UpdateMetadata>) -> Result<()> {
    ctx.accounts.metadata.is_mutable = false;
    msg!("Metadata locked - now immutable");
    Ok(())
}
```

---

## Trading & Purchases

### Purchase Receipt Schema

```rust
// Non-account struct for event emission
#[solana]
struct PurchaseReceipt {
    buyer: PublicKey,
    seller: PublicKey,
    nft_mint: PublicKey,

    // Pricing breakdown
    price: u64,                     // Total price paid
    marketplace_fee: u64,           // Fee to marketplace
    royalty_fee: u64,               // Fee to creators

    // Proof
    timestamp: i64,
    transaction_signature: Signature,  // On-chain tx ID
}
```

### Buy NFT

```rust
pub fn buy_nft(ctx: Context<BuyNft>) -> Result<()> {
    let marketplace = &mut ctx.accounts.marketplace;
    let listing = &ctx.accounts.listing;
    let metadata = &ctx.accounts.metadata;
    let clock = Clock::get()?;

    // Validate marketplace is active
    require!(marketplace.is_active, MarketplaceError::MarketplacePaused);

    // Check listing hasn't expired
    if let Some(expires_at) = listing.expires_at {
        require!(
            clock.unix_timestamp < expires_at,
            MarketplaceError::ListingExpired
        );
    }

    let price = listing.price;

    // Calculate fees
    let marketplace_fee = calculate_fee(price, marketplace.fee_basis_points);
    let royalty_fee = calculate_fee(price, metadata.seller_fee_basis_points);
    let seller_proceeds = price
        .checked_sub(marketplace_fee)
        .unwrap()
        .checked_sub(royalty_fee)
        .unwrap();

    // Transfer SOL: buyer -> seller
    let transfer_to_seller = system_instruction::transfer(
        &ctx.accounts.buyer.key(),
        &listing.seller,
        seller_proceeds,
    );
    invoke(
        &transfer_to_seller,
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.seller.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Transfer SOL: buyer -> marketplace fee recipient
    if marketplace_fee > 0 {
        let transfer_marketplace_fee = system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &marketplace.fee_recipient,
            marketplace_fee,
        );
        invoke(
            &transfer_marketplace_fee,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.fee_recipient.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    // Transfer SOL: buyer -> creator (royalties)
    if royalty_fee > 0 && !metadata.creators.is_empty() {
        let creator = metadata.creators[0];  // Simplified: single creator
        let transfer_royalty = system_instruction::transfer(
            &ctx.accounts.buyer.key(),
            &creator,
            royalty_fee,
        );
        invoke(
            &transfer_royalty,
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.creator_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    // Transfer NFT: escrow -> buyer
    let nft_mint = listing.nft_mint;
    let seeds = &[b"escrow", nft_mint.as_ref(), &[ctx.bumps.escrow_authority]];
    let signer_seeds = &[&seeds[..]];

    let transfer_nft = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.escrow_token_account.to_account_info(),
            to: ctx.accounts.buyer_token_account.to_account_info(),
            authority: ctx.accounts.escrow_authority.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_nft, 1)?;

    // Update marketplace stats
    marketplace.total_volume = marketplace.total_volume.saturating_add(price);
    marketplace.total_sales = marketplace.total_sales.saturating_add(1);

    // Emit purchase receipt
    emit!(PurchaseCompleted {
        buyer: ctx.accounts.buyer.key(),
        seller: listing.seller,
        nft_mint: listing.nft_mint,
        price,
        marketplace_fee,
        royalty_fee,
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "NFT sold for {} lamports (fees: {} marketplace, {} royalty)",
        price, marketplace_fee, royalty_fee
    );

    Ok(())
}

#[event]
pub struct PurchaseCompleted {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub price: u64,
    pub marketplace_fee: u64,
    pub royalty_fee: u64,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct BuyNft<'info> {
    #[account(mut)]
    pub marketplace: Account<'info, Marketplace>,

    #[account(
        mut,
        close = seller,
        seeds = [b"listing", listing.nft_mint.as_ref()],
        bump
    )]
    pub listing: Account<'info, NftListing>,

    #[account(
        seeds = [b"metadata", listing.nft_mint.as_ref()],
        bump
    )]
    pub metadata: Account<'info, NftMetadata>,

    #[account(mut)]
    pub escrow_token_account: Account<'info, TokenAccount>,

    /// CHECK: PDA authority
    #[account(seeds = [b"escrow", listing.nft_mint.as_ref()], bump)]
    pub escrow_authority: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = nft_mint,
        associated_token::authority = buyer
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    pub nft_mint: Account<'info, Mint>,

    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Seller receives SOL
    #[account(mut)]
    pub seller: UncheckedAccount<'info>,

    /// CHECK: Fee recipient
    #[account(mut)]
    pub fee_recipient: UncheckedAccount<'info>,

    /// CHECK: Creator for royalties
    #[account(mut)]
    pub creator_account: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
```

### Offer System

```rust
#[solana]
#[account]
struct Offer {
    buyer: PublicKey,
    nft_mint: PublicKey,
    amount: u64,
    expires_at: i64,
}

pub fn make_offer(
    ctx: Context<MakeOffer>,
    amount: u64,
    expires_at: i64,
) -> Result<()> {
    let offer = &mut ctx.accounts.offer;
    let clock = Clock::get()?;

    require!(amount > 0, MarketplaceError::InvalidPrice);
    require!(expires_at > clock.unix_timestamp, MarketplaceError::InvalidExpiration);

    offer.buyer = ctx.accounts.buyer.key();
    offer.nft_mint = ctx.accounts.nft_mint.key();
    offer.amount = amount;
    offer.expires_at = expires_at;

    // Escrow the SOL
    let transfer_ix = system_instruction::transfer(
        &ctx.accounts.buyer.key(),
        &ctx.accounts.offer_escrow.key(),
        amount,
    );
    invoke(
        &transfer_ix,
        &[
            ctx.accounts.buyer.to_account_info(),
            ctx.accounts.offer_escrow.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    msg!("Offer made: {} lamports", amount);
    Ok(())
}

pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
    // Similar to buy_nft but uses offer.amount
    // Transfer NFT to buyer, SOL to seller (minus fees)
    Ok(())
}

pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
    // Return escrowed SOL to buyer
    Ok(())
}
```

---

## Royalties & Creator Fees

### Royalty Distribution

```rust
// Distribute royalties among multiple creators
fn distribute_royalties(
    total_royalty: u64,
    creators: &[Creator],
    accounts: &[AccountInfo],
    buyer: &Signer,
    system_program: &Program<System>,
) -> Result<()> {
    for (i, creator) in creators.iter().enumerate() {
        // Calculate this creator's share
        let creator_amount = (total_royalty as u128)
            .checked_mul(creator.share as u128)
            .unwrap()
            .checked_div(100)
            .unwrap() as u64;

        if creator_amount > 0 {
            let transfer_ix = system_instruction::transfer(
                &buyer.key(),
                &creator.address,
                creator_amount,
            );
            invoke(
                &transfer_ix,
                &[
                    buyer.to_account_info(),
                    accounts[i].clone(),
                    system_program.to_account_info(),
                ],
            )?;

            msg!(
                "Royalty to creator {}: {} lamports ({}%)",
                creator.address,
                creator_amount,
                creator.share
            );
        }
    }

    Ok(())
}
```

### Royalty Verification

```rust
// Validate creator shares sum to 100
fn validate_creator_shares(creators: &[Creator]) -> Result<()> {
    let total_shares: u8 = creators.iter().map(|c| c.share).sum();

    require!(
        total_shares == 100,
        MarketplaceError::InvalidCreatorShares
    );

    require!(
        creators.len() <= 5,
        MarketplaceError::TooManyCreators
    );

    Ok(())
}
```

### Enforced vs Honor-Based Royalties

```rust
// Enforced royalties (in program)
pub fn buy_with_enforced_royalties(ctx: Context<BuyNft>) -> Result<()> {
    // Royalties are automatically deducted - cannot be bypassed
    let royalty_fee = calculate_fee(price, metadata.seller_fee_basis_points);
    // ... distribute to creators ...
    Ok(())
}

// For Metaplex pNFTs (Programmable NFTs), royalties are enforced at protocol level
// Token2022 also supports enforced royalties via transfer hooks
```

---

## Best Practices

### On-Chain vs Off-Chain Data

```markdown
**Store On-Chain:**
- Mint address
- Owner (derived from token account)
- Creator addresses
- Royalty percentage
- Collection reference
- Mutability flag

**Store Off-Chain (URI → JSON):**
- Name, description
- Image/animation URLs
- Attributes/traits
- Full creator details
- External links
```

### Metadata Storage Options

| Storage | Pros | Cons | Cost |
|---------|------|------|------|
| **Arweave** | Permanent, decentralized | Slower, one-time cost | ~$0.01/KB |
| **IPFS** | Fast, decentralized | Needs pinning | Free + pinning |
| **AWS S3** | Fast, reliable | Centralized | ~$0.02/GB/mo |
| **Shadow Drive** | Solana-native, fast | Newer | ~0.05 SHDW/GB |

### Account Size Optimization

```rust
impl NftMetadata {
    // Calculate exact space needed
    pub const INIT_SPACE: usize =
        32 +          // mint: PublicKey
        4 + 32 +      // name: String (len + max chars)
        4 + 10 +      // symbol: String
        4 + 200 +     // uri: String
        2 +           // seller_fee_basis_points: u16
        4 + (32 * 5) +// creators: Vec<Pubkey> (max 5)
        1;            // is_mutable: bool
    // Total: ~430 bytes
    // Rent: ~0.003 SOL
}
```

### Collection Verification

```rust
// Link NFT to verified collection
#[solana]
#[account]
struct CollectionMembership {
    nft_mint: PublicKey,
    collection_mint: PublicKey,
    verified: bool,
}

pub fn verify_collection(ctx: Context<VerifyCollection>) -> Result<()> {
    // Only collection authority can verify
    require!(
        ctx.accounts.authority.key() == ctx.accounts.collection.update_authority,
        MarketplaceError::Unauthorized
    );

    ctx.accounts.membership.verified = true;
    Ok(())
}
```

### Indexing for Discovery

```rust
// Emit events for indexers (Helius, Yellowstone, etc.)
#[event]
pub struct NftListed {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub nft_mint: Pubkey,
    pub price: u64,
    pub collection: Option<Pubkey>,
    pub timestamp: i64,
}

#[event]
pub struct NftSold {
    pub nft_mint: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub price: u64,
    pub timestamp: i64,
}

// Indexer subscribes to these events for real-time updates
```

---

## Complete Implementation

### Full Program Structure

```rust
// programs/nft-marketplace/src/lib.rs

use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;
pub mod utils;

use instructions::*;

declare_id!("NFTmktXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

#[program]
pub mod nft_marketplace {
    use super::*;

    // === Marketplace Management ===

    pub fn initialize_marketplace(
        ctx: Context<InitializeMarketplace>,
        fee_basis_points: u16,
    ) -> Result<()> {
        instructions::marketplace::initialize(ctx, fee_basis_points)
    }

    pub fn update_marketplace(
        ctx: Context<UpdateMarketplace>,
        new_fee_basis_points: Option<u16>,
        new_fee_recipient: Option<Pubkey>,
    ) -> Result<()> {
        instructions::marketplace::update(ctx, new_fee_basis_points, new_fee_recipient)
    }

    pub fn pause_marketplace(ctx: Context<UpdateMarketplace>) -> Result<()> {
        instructions::marketplace::pause(ctx)
    }

    pub fn resume_marketplace(ctx: Context<UpdateMarketplace>) -> Result<()> {
        instructions::marketplace::resume(ctx)
    }

    // === NFT Minting ===

    pub fn mint_nft(
        ctx: Context<MintNft>,
        name: String,
        symbol: String,
        uri: String,
        seller_fee_basis_points: u16,
    ) -> Result<()> {
        instructions::mint::mint_nft(ctx, name, symbol, uri, seller_fee_basis_points)
    }

    pub fn update_metadata(
        ctx: Context<UpdateMetadata>,
        name: Option<String>,
        symbol: Option<String>,
        uri: Option<String>,
    ) -> Result<()> {
        instructions::mint::update_metadata(ctx, name, symbol, uri)
    }

    pub fn make_immutable(ctx: Context<UpdateMetadata>) -> Result<()> {
        instructions::mint::make_immutable(ctx)
    }

    // === Listing Management ===

    pub fn list_nft(
        ctx: Context<ListNft>,
        price: u64,
        expires_at: Option<i64>,
    ) -> Result<()> {
        instructions::listing::list(ctx, price, expires_at)
    }

    pub fn update_listing(
        ctx: Context<UpdateListing>,
        new_price: u64,
    ) -> Result<()> {
        instructions::listing::update(ctx, new_price)
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        instructions::listing::cancel(ctx)
    }

    pub fn delist_expired(ctx: Context<DelistExpired>) -> Result<()> {
        instructions::listing::delist_expired(ctx)
    }

    // === Trading ===

    pub fn buy_nft(ctx: Context<BuyNft>) -> Result<()> {
        instructions::trade::buy(ctx)
    }

    pub fn make_offer(
        ctx: Context<MakeOffer>,
        amount: u64,
        expires_at: i64,
    ) -> Result<()> {
        instructions::trade::make_offer(ctx, amount, expires_at)
    }

    pub fn accept_offer(ctx: Context<AcceptOffer>) -> Result<()> {
        instructions::trade::accept_offer(ctx)
    }

    pub fn cancel_offer(ctx: Context<CancelOffer>) -> Result<()> {
        instructions::trade::cancel_offer(ctx)
    }
}
```

### Error Definitions

```rust
// programs/nft-marketplace/src/errors.rs

use anchor_lang::prelude::*;

#[error_code]
pub enum MarketplaceError {
    #[msg("Marketplace fee cannot exceed 10%")]
    FeeTooHigh,

    #[msg("Marketplace is paused")]
    MarketplacePaused,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Name exceeds 32 characters")]
    NameTooLong,

    #[msg("Symbol exceeds 10 characters")]
    SymbolTooLong,

    #[msg("URI exceeds 200 characters")]
    UriTooLong,

    #[msg("Royalty fee cannot exceed 100%")]
    RoyaltyTooHigh,

    #[msg("Metadata is immutable")]
    MetadataImmutable,

    #[msg("Invalid price")]
    InvalidPrice,

    #[msg("Expiration must be in the future")]
    InvalidExpiration,

    #[msg("Listing has expired")]
    ListingExpired,

    #[msg("Listing has not expired")]
    ListingNotExpired,

    #[msg("Listing has no expiration")]
    ListingHasNoExpiration,

    #[msg("Creator shares must sum to 100")]
    InvalidCreatorShares,

    #[msg("Maximum 5 creators allowed")]
    TooManyCreators,

    #[msg("Offer has expired")]
    OfferExpired,

    #[msg("Insufficient funds")]
    InsufficientFunds,
}
```

---

## Frontend Integration

### React Hooks

```typescript
// src/hooks/useNftMetadata.ts
import { useEffect, useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { NftMetadata, NftMetadataSchema } from '../generated';
import { PROGRAM_ID } from '../constants';

export function useNftMetadata(mint: PublicKey | null) {
  const { connection } = useConnection();
  const [metadata, setMetadata] = useState<NftMetadata | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetch() {
      if (!mint) {
        setMetadata(null);
        setLoading(false);
        return;
      }

      try {
        const [metadataPda] = PublicKey.findProgramAddressSync(
          [Buffer.from('metadata'), mint.toBuffer()],
          PROGRAM_ID
        );

        const accountInfo = await connection.getAccountInfo(metadataPda);
        if (accountInfo) {
          const data = accountInfo.data.slice(8);
          setMetadata(NftMetadataSchema.decode(Buffer.from(data)));
        }
      } catch (err) {
        console.error('Failed to fetch metadata:', err);
      } finally {
        setLoading(false);
      }
    }

    fetch();
  }, [connection, mint]);

  return { metadata, loading };
}
```

### Listings Hook

```typescript
// src/hooks/useListings.ts
import { useEffect, useState } from 'react';
import { PublicKey, GetProgramAccountsFilter } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { NftListing, NftListingSchema } from '../generated';
import { PROGRAM_ID } from '../constants';

export function useListings(filters?: { seller?: PublicKey; maxPrice?: number }) {
  const { connection } = useConnection();
  const [listings, setListings] = useState<NftListing[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetchListings() {
      try {
        const programFilters: GetProgramAccountsFilter[] = [
          { dataSize: 8 + 32 + 32 + 8 + 8 + 9 }, // NftListing size
        ];

        if (filters?.seller) {
          programFilters.push({
            memcmp: {
              offset: 8, // After discriminator
              bytes: filters.seller.toBase58(),
            },
          });
        }

        const accounts = await connection.getProgramAccounts(PROGRAM_ID, {
          filters: programFilters,
        });

        const decoded = accounts
          .map(({ account }) => {
            const data = account.data.slice(8);
            return NftListingSchema.decode(Buffer.from(data));
          })
          .filter((listing) => {
            if (filters?.maxPrice && listing.price > filters.maxPrice) {
              return false;
            }
            return true;
          });

        setListings(decoded);
      } catch (err) {
        console.error('Failed to fetch listings:', err);
      } finally {
        setLoading(false);
      }
    }

    fetchListings();
  }, [connection, filters?.seller, filters?.maxPrice]);

  return { listings, loading };
}
```

### NFT Card Component

```typescript
// src/components/NftCard.tsx
import { PublicKey } from '@solana/web3.js';
import { useNftMetadata } from '../hooks/useNftMetadata';
import { NftListing } from '../generated';

interface NftCardProps {
  mint: PublicKey;
  listing?: NftListing;
  onBuy?: () => void;
  onList?: () => void;
}

export function NftCard({ mint, listing, onBuy, onList }: NftCardProps) {
  const { metadata, loading } = useNftMetadata(mint);
  const [jsonMetadata, setJsonMetadata] = useState<any>(null);

  // Fetch off-chain metadata
  useEffect(() => {
    if (metadata?.uri) {
      fetch(metadata.uri)
        .then((res) => res.json())
        .then(setJsonMetadata)
        .catch(console.error);
    }
  }, [metadata?.uri]);

  if (loading) return <div className="nft-card loading">Loading...</div>;
  if (!metadata) return <div className="nft-card error">NFT not found</div>;

  const royaltyPercent = metadata.seller_fee_basis_points / 100;

  return (
    <div className="nft-card">
      {jsonMetadata?.image && (
        <img
          src={jsonMetadata.image}
          alt={metadata.name}
          className="nft-image"
        />
      )}

      <div className="nft-info">
        <h3>{metadata.name}</h3>
        <span className="symbol">{metadata.symbol}</span>

        {jsonMetadata?.description && (
          <p className="description">{jsonMetadata.description}</p>
        )}

        <div className="royalty">
          Royalty: {royaltyPercent}%
        </div>

        {jsonMetadata?.attributes && (
          <div className="attributes">
            {jsonMetadata.attributes.map((attr: any, i: number) => (
              <span key={i} className="attribute">
                {attr.trait_type}: {attr.value}
              </span>
            ))}
          </div>
        )}
      </div>

      {listing ? (
        <div className="listing-info">
          <div className="price">
            {(listing.price / 1e9).toFixed(2)} SOL
          </div>
          <button onClick={onBuy} className="buy-button">
            Buy Now
          </button>
        </div>
      ) : (
        <button onClick={onList} className="list-button">
          List for Sale
        </button>
      )}
    </div>
  );
}
```

### Listing Form Component

```typescript
// src/components/ListingForm.tsx
import { useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { useProgram } from '../hooks/useProgram';

interface ListingFormProps {
  nftMint: PublicKey;
  onSuccess?: () => void;
}

export function ListingForm({ nftMint, onSuccess }: ListingFormProps) {
  const program = useProgram();
  const [price, setPrice] = useState('');
  const [duration, setDuration] = useState('7'); // days
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!program) return;

    setLoading(true);
    try {
      const priceInLamports = new BN(parseFloat(price) * 1e9);
      const expiresAt = duration
        ? new BN(Date.now() / 1000 + parseInt(duration) * 86400)
        : null;

      await program.methods
        .listNft(priceInLamports, expiresAt)
        .accounts({
          // ... account setup
        })
        .rpc();

      onSuccess?.();
    } catch (err) {
      console.error('Failed to list:', err);
    } finally {
      setLoading(false);
    }
  }

  return (
    <form onSubmit={handleSubmit} className="listing-form">
      <h3>List NFT for Sale</h3>

      <div className="form-group">
        <label>Price (SOL)</label>
        <input
          type="number"
          step="0.01"
          min="0"
          value={price}
          onChange={(e) => setPrice(e.target.value)}
          placeholder="0.00"
          required
        />
      </div>

      <div className="form-group">
        <label>Duration (days)</label>
        <select value={duration} onChange={(e) => setDuration(e.target.value)}>
          <option value="">No expiration</option>
          <option value="1">1 day</option>
          <option value="7">7 days</option>
          <option value="30">30 days</option>
        </select>
      </div>

      <button type="submit" disabled={loading || !price}>
        {loading ? 'Listing...' : 'List NFT'}
      </button>
    </form>
  );
}
```

### Marketplace Stats Component

```typescript
// src/components/MarketplaceStats.tsx
import { useMarketplace } from '../hooks/useMarketplace';

export function MarketplaceStats() {
  const { marketplace, loading } = useMarketplace();

  if (loading) return <div>Loading stats...</div>;
  if (!marketplace) return null;

  const volumeInSol = marketplace.total_volume / 1e9;
  const feePercent = marketplace.fee_basis_points / 100;

  return (
    <div className="marketplace-stats">
      <div className="stat">
        <span className="label">Total Volume</span>
        <span className="value">{volumeInSol.toLocaleString()} SOL</span>
      </div>

      <div className="stat">
        <span className="label">Total Sales</span>
        <span className="value">{marketplace.total_sales.toLocaleString()}</span>
      </div>

      <div className="stat">
        <span className="label">Marketplace Fee</span>
        <span className="value">{feePercent}%</span>
      </div>

      <div className="stat">
        <span className="label">Status</span>
        <span className={`status ${marketplace.is_active ? 'active' : 'paused'}`}>
          {marketplace.is_active ? 'Active' : 'Paused'}
        </span>
      </div>
    </div>
  );
}
```

---

## Resources

### Example Code

- **LUMOS NFT Schema:** [`examples/nft-marketplace/schema.lumos`](../../../examples/nft-marketplace/schema.lumos)
- **awesome-lumos NFT:** [github.com/getlumos/awesome-lumos](https://github.com/getlumos/awesome-lumos)

### Related Guides

- [Solana CLI Integration](../solana-cli-integration.md) - Deploy your marketplace
- [web3.js Integration](../web3js-integration.md) - Frontend patterns
- [Gaming Use Case](./gaming.md) - Another LUMOS use case

### External Resources

- [Metaplex Documentation](https://docs.metaplex.com/)
- [Metaplex Token Metadata](https://developers.metaplex.com/token-metadata)
- [Anchor Documentation](https://www.anchor-lang.com/)
- [Magic Eden API](https://docs.magiceden.io/)

### Metadata Standards

- [Metaplex Token Standard](https://docs.metaplex.com/programs/token-metadata/token-standard)
- [OpenSea Metadata Standards](https://docs.opensea.io/docs/metadata-standards)

---

## Next Steps

1. **Generate your schema:** `lumos generate schemas/nft-marketplace.lumos`
2. **Validate Metaplex compliance:** `lumos metaplex validate schemas/nft-marketplace.lumos`
3. **Build the program:** `anchor build`
4. **Test locally:** `solana-test-validator && anchor test`
5. **Deploy to devnet:** Follow [Solana CLI guide](../solana-cli-integration.md)
6. **Build your frontend:** Use patterns from [web3.js guide](../web3js-integration.md)

---

**Last Updated:** 2025-12-16
**Version:** 1.0.0
