# LUMOS + web3.js Integration Guide

**Purpose:** Client-side integration guide for using LUMOS-generated TypeScript with @solana/web3.js

**Last Updated:** 2025-12-16

---

## Overview

This guide covers frontend integration patterns for Solana dApps using LUMOS-generated TypeScript:

```
schema.lumos → lumos generate → generated.ts → Your Frontend App
```

**What you'll learn:**
- Understanding generated TypeScript code
- Fetching and deserializing on-chain accounts
- Building and sending transactions
- React and Next.js integration patterns
- Complete dApp example

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Understanding Generated TypeScript](#understanding-generated-typescript)
3. [Fetching Account Data](#fetching-account-data)
4. [Deserializing with Borsh](#deserializing-with-borsh)
5. [Building Transactions](#building-transactions)
6. [React Integration](#react-integration)
7. [Next.js Integration](#nextjs-integration)
8. [Complete dApp Example](#complete-dapp-example)
9. [Error Handling](#error-handling)
10. [Advanced Patterns](#advanced-patterns)
11. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Packages

```bash
# Core Solana packages
npm install @solana/web3.js @coral-xyz/borsh

# For Anchor programs (recommended)
npm install @coral-xyz/anchor

# Wallet adapter (for React)
npm install @solana/wallet-adapter-react @solana/wallet-adapter-react-ui @solana/wallet-adapter-wallets
```

### Package Versions

| Package | Minimum Version | Purpose |
|---------|-----------------|---------|
| `@solana/web3.js` | 1.87+ | Solana RPC client |
| `@coral-xyz/borsh` | 0.3+ | Borsh serialization |
| `@coral-xyz/anchor` | 0.30+ | Anchor client (optional) |

### Project Setup

**Vite (Recommended):**
```bash
npm create vite@latest my-dapp -- --template react-ts
cd my-dapp
npm install @solana/web3.js @coral-xyz/borsh
```

**Next.js:**
```bash
npx create-next-app@latest my-dapp --typescript
cd my-dapp
npm install @solana/web3.js @coral-xyz/borsh
```

**Create React App:**
```bash
npx create-react-app my-dapp --template typescript
cd my-dapp
npm install @solana/web3.js @coral-xyz/borsh
```

### Generate TypeScript from LUMOS

```bash
# Generate to your src directory
lumos generate schema.lumos --output ./src/

# Files created:
# ./src/generated.ts
```

---

## Understanding Generated TypeScript

LUMOS generates three main components for each type:

### 1. TypeScript Interfaces

Type-safe interfaces matching your schema:

```typescript
// From schema.lumos:
// #[solana]
// #[account]
// struct PlayerAccount {
//     wallet: PublicKey,
//     username: String,
//     level: u16,
//     experience: u64,
// }

// Generated TypeScript:
export interface PlayerAccount {
  wallet: PublicKey;
  username: string;
  level: number;
  /**
   * WARNING: TypeScript 'number' has precision limit of 2^53-1.
   * For large values, ensure they stay within safe range.
   */
  experience: number;
}
```

### 2. Borsh Schemas

Serialization schemas for encoding/decoding:

```typescript
export const PlayerAccountSchema = borsh.struct([
  borsh.publicKey('wallet'),
  borsh.str('username'),
  borsh.u16('level'),
  borsh.u64('experience'),
]);
```

### 3. Enum Discriminated Unions

Type-safe enums with `kind` discriminator:

```typescript
// From schema.lumos:
// #[solana]
// enum GameState {
//     Active,
//     Paused,
//     Finished { winner: PublicKey },
// }

// Generated TypeScript:
export type GameState =
  | { kind: 'Active' }
  | { kind: 'Paused' }
  | { kind: 'Finished'; winner: PublicKey };

export const GameStateSchema = borsh.rustEnum([
  borsh.struct([], 'Active'),
  borsh.struct([], 'Paused'),
  borsh.struct([borsh.publicKey('winner')], 'Finished'),
]);
```

### Type Mappings Reference

| LUMOS Type | TypeScript Type | Notes |
|------------|-----------------|-------|
| `u8`, `u16`, `u32` | `number` | Safe integers |
| `u64`, `i64` | `number` | ⚠️ Precision warning for > 2^53 |
| `u128`, `i128` | `bigint` | Full precision |
| `String` | `string` | UTF-8 |
| `bool` | `boolean` | - |
| `PublicKey` | `PublicKey` | From @solana/web3.js |
| `[T]` | `T[]` | Dynamic array |
| `Option<T>` | `T \| undefined` | Optional |

### File Structure

```typescript
// generated.ts structure:

// 1. Imports (auto-detected)
import * as borsh from '@coral-xyz/borsh';
import { PublicKey } from '@solana/web3.js';

// 2. Interfaces
export interface PlayerAccount { ... }
export interface GameItem { ... }

// 3. Borsh Schemas
export const PlayerAccountSchema = borsh.struct([...]);
export const GameItemSchema = borsh.struct([...]);

// 4. Enum Types (if any)
export type GameState = ...;
export const GameStateSchema = borsh.rustEnum([...]);

// 5. Version Constants (if using #[version])
export const PLAYERACCOUNT_VERSION = "1.0.0";
```

---

## Fetching Account Data

### Basic Account Fetch

```typescript
import { Connection, PublicKey } from '@solana/web3.js';

// Create connection
const connection = new Connection('https://api.devnet.solana.com', 'confirmed');

// Fetch raw account data
async function fetchAccountInfo(pubkey: PublicKey) {
  const accountInfo = await connection.getAccountInfo(pubkey);

  if (!accountInfo) {
    throw new Error('Account not found');
  }

  return accountInfo;
}
```

### Fetch Multiple Accounts

```typescript
async function fetchMultipleAccounts(pubkeys: PublicKey[]) {
  const accounts = await connection.getMultipleAccountsInfo(pubkeys);

  return accounts.map((account, index) => ({
    pubkey: pubkeys[index],
    account,
  }));
}
```

### Program Derived Addresses (PDAs)

```typescript
const PROGRAM_ID = new PublicKey('YourProgramId...');

// Derive PDA
function getPlayerPDA(wallet: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('player'), wallet.toBuffer()],
    PROGRAM_ID
  );
}

// Usage
const [playerPda] = getPlayerPDA(walletPublicKey);
const accountInfo = await connection.getAccountInfo(playerPda);
```

---

## Deserializing with Borsh

### Basic Deserialization

```typescript
import { PlayerAccountSchema, PlayerAccount } from './generated';

async function fetchPlayer(pubkey: PublicKey): Promise<PlayerAccount> {
  const accountInfo = await connection.getAccountInfo(pubkey);

  if (!accountInfo) {
    throw new Error('Player account not found');
  }

  // Decode the account data
  return PlayerAccountSchema.decode(accountInfo.data);
}
```

### Anchor Account Discriminator

Anchor accounts have an 8-byte discriminator prefix. Skip it when deserializing:

```typescript
async function fetchAnchorAccount<T>(
  pubkey: PublicKey,
  schema: borsh.Schema<T>
): Promise<T> {
  const accountInfo = await connection.getAccountInfo(pubkey);

  if (!accountInfo) {
    throw new Error('Account not found');
  }

  // Skip 8-byte Anchor discriminator
  const data = accountInfo.data.slice(8);
  return schema.decode(Buffer.from(data));
}

// Usage
const player = await fetchAnchorAccount(playerPda, PlayerAccountSchema);
```

### Deserializing Enums

```typescript
import { GameStateSchema, GameState } from './generated';

function deserializeGameState(data: Buffer): GameState {
  return GameStateSchema.decode(data);
}

// Type narrowing with discriminated unions
function handleGameState(state: GameState) {
  switch (state.kind) {
    case 'Active':
      console.log('Game is active');
      break;
    case 'Paused':
      console.log('Game is paused');
      break;
    case 'Finished':
      console.log(`Winner: ${state.winner.toString()}`);
      break;
  }
}
```

### Handling Optional Fields

```typescript
interface GameItem {
  id: number;
  owner: PublicKey;
  name: string;
  description: string | undefined;  // Option<String>
}

// Usage
const item = await fetchGameItem(itemPda);
if (item.description) {
  console.log(`Description: ${item.description}`);
} else {
  console.log('No description set');
}
```

---

## Building Transactions

### With Anchor Client (Recommended)

```typescript
import { Program, AnchorProvider, web3 } from '@coral-xyz/anchor';
import { useAnchorWallet } from '@solana/wallet-adapter-react';

// Initialize program
const provider = new AnchorProvider(connection, wallet, {});
const program = new Program(idl, PROGRAM_ID, provider);

// Call instruction
async function initializePlayer(username: string) {
  const [playerPda] = getPlayerPDA(wallet.publicKey);

  const tx = await program.methods
    .initializePlayer(username)
    .accounts({
      player: playerPda,
      authority: wallet.publicKey,
      systemProgram: web3.SystemProgram.programId,
    })
    .rpc();

  console.log('Transaction signature:', tx);
  return tx;
}
```

### With Raw web3.js

```typescript
import {
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

// Build instruction manually
function buildInitializePlayerInstruction(
  playerPda: PublicKey,
  authority: PublicKey,
  username: string
): TransactionInstruction {
  // Instruction discriminator (first 8 bytes of sha256("global:initialize_player"))
  const discriminator = Buffer.from([/* your discriminator bytes */]);

  // Serialize instruction data
  const usernameBuffer = Buffer.from(username, 'utf-8');
  const data = Buffer.concat([
    discriminator,
    Buffer.from([usernameBuffer.length]),
    usernameBuffer,
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: playerPda, isSigner: false, isWritable: true },
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });
}

// Send transaction
async function sendTransaction(instruction: TransactionInstruction) {
  const transaction = new Transaction().add(instruction);

  const signature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet],  // signers
    { commitment: 'confirmed' }
  );

  return signature;
}
```

### Serializing Instruction Data with Borsh

```typescript
import * as borsh from '@coral-xyz/borsh';

// Define instruction schema
const InitializePlayerSchema = borsh.struct([
  borsh.str('username'),
  borsh.u16('startingLevel'),
]);

// Serialize instruction data
function serializeInitializePlayer(username: string, startingLevel: number): Buffer {
  const data = { username, startingLevel };
  return Buffer.from(InitializePlayerSchema.encode(data));
}
```

---

## React Integration

### Connection Provider

```typescript
// src/contexts/SolanaContext.tsx
import { Connection, clusterApiUrl } from '@solana/web3.js';
import { createContext, useContext, useMemo, ReactNode } from 'react';

interface SolanaContextType {
  connection: Connection;
  network: 'devnet' | 'testnet' | 'mainnet-beta';
}

const SolanaContext = createContext<SolanaContextType | null>(null);

export function SolanaProvider({
  children,
  network = 'devnet'
}: {
  children: ReactNode;
  network?: 'devnet' | 'testnet' | 'mainnet-beta';
}) {
  const connection = useMemo(
    () => new Connection(clusterApiUrl(network), 'confirmed'),
    [network]
  );

  return (
    <SolanaContext.Provider value={{ connection, network }}>
      {children}
    </SolanaContext.Provider>
  );
}

export function useSolana() {
  const context = useContext(SolanaContext);
  if (!context) {
    throw new Error('useSolana must be used within SolanaProvider');
  }
  return context;
}
```

### Wallet Adapter Setup

```typescript
// src/contexts/WalletContext.tsx
import { useMemo } from 'react';
import {
  ConnectionProvider,
  WalletProvider,
} from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import {
  PhantomWalletAdapter,
  SolflareWalletAdapter,
} from '@solana/wallet-adapter-wallets';
import { clusterApiUrl } from '@solana/web3.js';

import '@solana/wallet-adapter-react-ui/styles.css';

export function WalletContextProvider({ children }: { children: ReactNode }) {
  const network = 'devnet';
  const endpoint = useMemo(() => clusterApiUrl(network), [network]);

  const wallets = useMemo(
    () => [
      new PhantomWalletAdapter(),
      new SolflareWalletAdapter(),
    ],
    []
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          {children}
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
```

### Custom Hook: usePlayerAccount

```typescript
// src/hooks/usePlayerAccount.ts
import { useState, useEffect, useCallback } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { PlayerAccount, PlayerAccountSchema } from '../generated';

interface UsePlayerAccountResult {
  player: PlayerAccount | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

export function usePlayerAccount(pubkey: PublicKey | null): UsePlayerAccountResult {
  const { connection } = useConnection();
  const [player, setPlayer] = useState<PlayerAccount | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchPlayer = useCallback(async () => {
    if (!pubkey) {
      setPlayer(null);
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const accountInfo = await connection.getAccountInfo(pubkey);

      if (!accountInfo) {
        setPlayer(null);
        return;
      }

      // Skip 8-byte Anchor discriminator
      const data = accountInfo.data.slice(8);
      const decoded = PlayerAccountSchema.decode(Buffer.from(data));
      setPlayer(decoded);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to fetch player'));
    } finally {
      setLoading(false);
    }
  }, [connection, pubkey]);

  useEffect(() => {
    fetchPlayer();
  }, [fetchPlayer]);

  return { player, loading, error, refetch: fetchPlayer };
}
```

### Custom Hook: useProgram

```typescript
// src/hooks/useProgram.ts
import { useMemo } from 'react';
import { Program, AnchorProvider } from '@coral-xyz/anchor';
import { useConnection, useAnchorWallet } from '@solana/wallet-adapter-react';
import { IDL, PROGRAM_ID } from '../idl';

export function useProgram() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();

  const program = useMemo(() => {
    if (!wallet) return null;

    const provider = new AnchorProvider(connection, wallet, {
      commitment: 'confirmed',
    });

    return new Program(IDL, PROGRAM_ID, provider);
  }, [connection, wallet]);

  return program;
}
```

### Player Card Component

```typescript
// src/components/PlayerCard.tsx
import { PublicKey } from '@solana/web3.js';
import { usePlayerAccount } from '../hooks/usePlayerAccount';

interface PlayerCardProps {
  playerPda: PublicKey;
}

export function PlayerCard({ playerPda }: PlayerCardProps) {
  const { player, loading, error } = usePlayerAccount(playerPda);

  if (loading) {
    return <div className="player-card loading">Loading...</div>;
  }

  if (error) {
    return <div className="player-card error">Error: {error.message}</div>;
  }

  if (!player) {
    return <div className="player-card empty">No player found</div>;
  }

  return (
    <div className="player-card">
      <h2>{player.username}</h2>
      <div className="stats">
        <div className="stat">
          <span className="label">Level</span>
          <span className="value">{player.level}</span>
        </div>
        <div className="stat">
          <span className="label">Experience</span>
          <span className="value">{player.experience.toLocaleString()}</span>
        </div>
        <div className="stat">
          <span className="label">Gold</span>
          <span className="value">{player.gold.toLocaleString()}</span>
        </div>
      </div>
      <div className="wallet">
        {player.wallet.toString().slice(0, 8)}...
      </div>
    </div>
  );
}
```

### Transaction Hook

```typescript
// src/hooks/useInitializePlayer.ts
import { useState, useCallback } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { useProgram } from './useProgram';

interface UseInitializePlayerResult {
  initialize: (username: string) => Promise<string>;
  loading: boolean;
  error: Error | null;
}

export function useInitializePlayer(): UseInitializePlayerResult {
  const { publicKey } = useWallet();
  const program = useProgram();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const initialize = useCallback(async (username: string): Promise<string> => {
    if (!publicKey || !program) {
      throw new Error('Wallet not connected');
    }

    setLoading(true);
    setError(null);

    try {
      const [playerPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('player'), publicKey.toBuffer()],
        program.programId
      );

      const tx = await program.methods
        .initializePlayer(username)
        .accounts({
          player: playerPda,
          authority: publicKey,
        })
        .rpc();

      return tx;
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Transaction failed');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [publicKey, program]);

  return { initialize, loading, error };
}
```

---

## Next.js Integration

### App Router Setup (Next.js 13+)

```typescript
// app/providers.tsx
'use client';

import { WalletContextProvider } from '@/contexts/WalletContext';

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <WalletContextProvider>
      {children}
    </WalletContextProvider>
  );
}

// app/layout.tsx
import { Providers } from './providers';

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
```

### Server-Side Data Fetching

```typescript
// app/player/[pubkey]/page.tsx
import { Connection, PublicKey } from '@solana/web3.js';
import { PlayerAccountSchema } from '@/generated';

// Server component - runs on server
async function fetchPlayerServer(pubkey: string) {
  const connection = new Connection(process.env.RPC_URL!, 'confirmed');
  const accountInfo = await connection.getAccountInfo(new PublicKey(pubkey));

  if (!accountInfo) return null;

  const data = accountInfo.data.slice(8);
  return PlayerAccountSchema.decode(Buffer.from(data));
}

export default async function PlayerPage({
  params,
}: {
  params: { pubkey: string };
}) {
  const player = await fetchPlayerServer(params.pubkey);

  if (!player) {
    return <div>Player not found</div>;
  }

  return (
    <div>
      <h1>{player.username}</h1>
      <p>Level: {player.level}</p>
    </div>
  );
}
```

### API Routes

```typescript
// app/api/player/[pubkey]/route.ts
import { NextRequest, NextResponse } from 'next/server';
import { Connection, PublicKey } from '@solana/web3.js';
import { PlayerAccountSchema } from '@/generated';

export async function GET(
  request: NextRequest,
  { params }: { params: { pubkey: string } }
) {
  try {
    const connection = new Connection(process.env.RPC_URL!, 'confirmed');
    const pubkey = new PublicKey(params.pubkey);
    const accountInfo = await connection.getAccountInfo(pubkey);

    if (!accountInfo) {
      return NextResponse.json(
        { error: 'Player not found' },
        { status: 404 }
      );
    }

    const data = accountInfo.data.slice(8);
    const player = PlayerAccountSchema.decode(Buffer.from(data));

    // Convert PublicKey to string for JSON serialization
    return NextResponse.json({
      ...player,
      wallet: player.wallet.toString(),
    });
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to fetch player' },
      { status: 500 }
    );
  }
}
```

### Client Component with Hydration

```typescript
// components/PlayerDashboard.tsx
'use client';

import { useEffect, useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { usePlayerAccount } from '@/hooks/usePlayerAccount';

export function PlayerDashboard() {
  const { publicKey, connected } = useWallet();
  const [playerPda, setPlayerPda] = useState<PublicKey | null>(null);

  useEffect(() => {
    if (publicKey) {
      const [pda] = PublicKey.findProgramAddressSync(
        [Buffer.from('player'), publicKey.toBuffer()],
        PROGRAM_ID
      );
      setPlayerPda(pda);
    } else {
      setPlayerPda(null);
    }
  }, [publicKey]);

  const { player, loading } = usePlayerAccount(playerPda);

  return (
    <div>
      <WalletMultiButton />

      {connected && (
        <div>
          {loading ? (
            <p>Loading player data...</p>
          ) : player ? (
            <div>
              <h2>Welcome, {player.username}!</h2>
              <p>Level {player.level}</p>
            </div>
          ) : (
            <p>No player account found. Create one to get started!</p>
          )}
        </div>
      )}
    </div>
  );
}
```

### Environment Variables

```bash
# .env.local
RPC_URL=https://api.devnet.solana.com
NEXT_PUBLIC_RPC_URL=https://api.devnet.solana.com
NEXT_PUBLIC_PROGRAM_ID=YourProgramId...
```

---

## Complete dApp Example

### Project Structure

```
gaming-dashboard/
├── src/
│   ├── generated.ts           # LUMOS generated
│   ├── idl/
│   │   └── gaming_program.json
│   ├── contexts/
│   │   └── WalletContext.tsx
│   ├── hooks/
│   │   ├── usePlayerAccount.ts
│   │   ├── useLeaderboard.ts
│   │   ├── useProgram.ts
│   │   └── useInitializePlayer.ts
│   ├── components/
│   │   ├── PlayerCard.tsx
│   │   ├── Leaderboard.tsx
│   │   ├── CreatePlayer.tsx
│   │   └── GameActions.tsx
│   ├── utils/
│   │   └── pda.ts
│   ├── App.tsx
│   └── main.tsx
├── package.json
└── vite.config.ts
```

### PDA Utilities

```typescript
// src/utils/pda.ts
import { PublicKey } from '@solana/web3.js';

export const PROGRAM_ID = new PublicKey('YourProgramId...');

export function getPlayerPDA(wallet: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from('player'), wallet.toBuffer()],
    PROGRAM_ID
  );
}

export function getLeaderboardPDA(season: number): [PublicKey, number] {
  const seasonBuffer = Buffer.alloc(4);
  seasonBuffer.writeUInt32LE(season);

  return PublicKey.findProgramAddressSync(
    [Buffer.from('leaderboard'), seasonBuffer],
    PROGRAM_ID
  );
}

export function getGameItemPDA(itemId: bigint): [PublicKey, number] {
  const idBuffer = Buffer.alloc(8);
  idBuffer.writeBigUInt64LE(itemId);

  return PublicKey.findProgramAddressSync(
    [Buffer.from('item'), idBuffer],
    PROGRAM_ID
  );
}
```

### Leaderboard Hook

```typescript
// src/hooks/useLeaderboard.ts
import { useState, useEffect } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { Leaderboard, LeaderboardSchema } from '../generated';
import { getLeaderboardPDA } from '../utils/pda';

export function useLeaderboard(season: number) {
  const { connection } = useConnection();
  const [leaderboard, setLeaderboard] = useState<Leaderboard | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function fetch() {
      try {
        const [pda] = getLeaderboardPDA(season);
        const accountInfo = await connection.getAccountInfo(pda);

        if (accountInfo) {
          const data = accountInfo.data.slice(8);
          setLeaderboard(LeaderboardSchema.decode(Buffer.from(data)));
        }
      } catch (err) {
        console.error('Failed to fetch leaderboard:', err);
      } finally {
        setLoading(false);
      }
    }

    fetch();
  }, [connection, season]);

  return { leaderboard, loading };
}
```

### Leaderboard Component

```typescript
// src/components/Leaderboard.tsx
import { useLeaderboard } from '../hooks/useLeaderboard';

interface LeaderboardProps {
  season: number;
}

export function Leaderboard({ season }: LeaderboardProps) {
  const { leaderboard, loading } = useLeaderboard(season);

  if (loading) return <div>Loading leaderboard...</div>;
  if (!leaderboard) return <div>No leaderboard found</div>;

  // Combine players and scores into entries
  const entries = leaderboard.top_players.map((player, index) => ({
    rank: index + 1,
    player: player.toString(),
    score: leaderboard.top_scores[index] || 0,
  }));

  return (
    <div className="leaderboard">
      <h2>Season {season} Leaderboard</h2>
      {leaderboard.is_active && <span className="badge">Active</span>}

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
            <tr key={entry.rank}>
              <td>#{entry.rank}</td>
              <td>{entry.player.slice(0, 8)}...</td>
              <td>{entry.score.toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
```

### Create Player Form

```typescript
// src/components/CreatePlayer.tsx
import { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { useInitializePlayer } from '../hooks/useInitializePlayer';

interface CreatePlayerProps {
  onSuccess?: () => void;
}

export function CreatePlayer({ onSuccess }: CreatePlayerProps) {
  const { connected } = useWallet();
  const { initialize, loading, error } = useInitializePlayer();
  const [username, setUsername] = useState('');

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();

    if (!username.trim()) return;

    try {
      const tx = await initialize(username);
      console.log('Player created! TX:', tx);
      setUsername('');
      onSuccess?.();
    } catch (err) {
      console.error('Failed to create player:', err);
    }
  }

  if (!connected) {
    return <p>Connect your wallet to create a player</p>;
  }

  return (
    <form onSubmit={handleSubmit} className="create-player-form">
      <h3>Create Your Player</h3>

      <input
        type="text"
        value={username}
        onChange={(e) => setUsername(e.target.value)}
        placeholder="Enter username"
        maxLength={20}
        disabled={loading}
      />

      <button type="submit" disabled={loading || !username.trim()}>
        {loading ? 'Creating...' : 'Create Player'}
      </button>

      {error && <p className="error">{error.message}</p>}
    </form>
  );
}
```

### Main App

```typescript
// src/App.tsx
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { usePlayerAccount } from './hooks/usePlayerAccount';
import { PlayerCard } from './components/PlayerCard';
import { CreatePlayer } from './components/CreatePlayer';
import { Leaderboard } from './components/Leaderboard';
import { getPlayerPDA } from './utils/pda';

export function App() {
  const { publicKey, connected } = useWallet();

  const playerPda = publicKey ? getPlayerPDA(publicKey)[0] : null;
  const { player, loading, refetch } = usePlayerAccount(playerPda);

  return (
    <div className="app">
      <header>
        <h1>Gaming Dashboard</h1>
        <WalletMultiButton />
      </header>

      <main>
        {!connected ? (
          <div className="connect-prompt">
            <p>Connect your wallet to get started</p>
          </div>
        ) : loading ? (
          <div className="loading">Loading...</div>
        ) : player ? (
          <div className="dashboard">
            <PlayerCard playerPda={playerPda!} />
            <Leaderboard season={1} />
          </div>
        ) : (
          <CreatePlayer onSuccess={refetch} />
        )}
      </main>
    </div>
  );
}
```

---

## Error Handling

### Custom Error Types

```typescript
// src/errors.ts
export class AccountNotFoundError extends Error {
  constructor(pubkey: string) {
    super(`Account not found: ${pubkey}`);
    this.name = 'AccountNotFoundError';
  }
}

export class DeserializationError extends Error {
  constructor(message: string) {
    super(`Failed to deserialize: ${message}`);
    this.name = 'DeserializationError';
  }
}

export class TransactionError extends Error {
  signature?: string;
  logs?: string[];

  constructor(message: string, signature?: string, logs?: string[]) {
    super(message);
    this.name = 'TransactionError';
    this.signature = signature;
    this.logs = logs;
  }
}
```

### Safe Fetch Wrapper

```typescript
// src/utils/fetch.ts
import { Connection, PublicKey } from '@solana/web3.js';
import { AccountNotFoundError, DeserializationError } from '../errors';

export async function safeFetchAccount<T>(
  connection: Connection,
  pubkey: PublicKey,
  schema: { decode: (data: Buffer) => T },
  skipDiscriminator = true
): Promise<T> {
  const accountInfo = await connection.getAccountInfo(pubkey);

  if (!accountInfo) {
    throw new AccountNotFoundError(pubkey.toString());
  }

  try {
    const data = skipDiscriminator
      ? accountInfo.data.slice(8)
      : accountInfo.data;
    return schema.decode(Buffer.from(data));
  } catch (err) {
    throw new DeserializationError(
      err instanceof Error ? err.message : 'Unknown error'
    );
  }
}
```

### Retry Logic

```typescript
// src/utils/retry.ts
export async function withRetry<T>(
  fn: () => Promise<T>,
  options: {
    retries?: number;
    delay?: number;
    backoff?: number;
  } = {}
): Promise<T> {
  const { retries = 3, delay = 1000, backoff = 2 } = options;

  let lastError: Error | undefined;

  for (let attempt = 0; attempt < retries; attempt++) {
    try {
      return await fn();
    } catch (err) {
      lastError = err instanceof Error ? err : new Error(String(err));

      if (attempt < retries - 1) {
        const waitTime = delay * Math.pow(backoff, attempt);
        await new Promise((resolve) => setTimeout(resolve, waitTime));
      }
    }
  }

  throw lastError;
}

// Usage
const player = await withRetry(
  () => safeFetchAccount(connection, playerPda, PlayerAccountSchema),
  { retries: 3, delay: 500 }
);
```

### Transaction Error Handling

```typescript
// src/utils/transaction.ts
import { SendTransactionError } from '@solana/web3.js';
import { TransactionError } from '../errors';

export async function sendWithErrorHandling<T>(
  fn: () => Promise<T>
): Promise<T> {
  try {
    return await fn();
  } catch (err) {
    if (err instanceof SendTransactionError) {
      throw new TransactionError(
        parseTransactionError(err),
        err.signature,
        err.logs
      );
    }
    throw err;
  }
}

function parseTransactionError(err: SendTransactionError): string {
  const logs = err.logs || [];

  // Look for custom program errors
  const programError = logs.find((log) => log.includes('Program log: Error:'));
  if (programError) {
    return programError.replace('Program log: Error: ', '');
  }

  // Check for common errors
  if (logs.some((log) => log.includes('insufficient funds'))) {
    return 'Insufficient SOL balance';
  }

  if (logs.some((log) => log.includes('account already exists'))) {
    return 'Account already exists';
  }

  return err.message;
}
```

### User-Friendly Error Messages

```typescript
// src/utils/errorMessages.ts
export function getUserFriendlyMessage(error: unknown): string {
  if (error instanceof AccountNotFoundError) {
    return 'This account does not exist yet. You may need to create it first.';
  }

  if (error instanceof DeserializationError) {
    return 'Failed to read account data. The account may be corrupted or incompatible.';
  }

  if (error instanceof TransactionError) {
    return `Transaction failed: ${error.message}`;
  }

  if (error instanceof Error) {
    // Wallet errors
    if (error.message.includes('User rejected')) {
      return 'Transaction was cancelled.';
    }

    if (error.message.includes('Blockhash not found')) {
      return 'Transaction expired. Please try again.';
    }

    return error.message;
  }

  return 'An unexpected error occurred.';
}
```

---

## Advanced Patterns

### Account Subscriptions

```typescript
// src/hooks/useAccountSubscription.ts
import { useEffect, useRef, useCallback } from 'react';
import { PublicKey, AccountInfo } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';

export function useAccountSubscription<T>(
  pubkey: PublicKey | null,
  schema: { decode: (data: Buffer) => T },
  onUpdate: (data: T) => void
) {
  const { connection } = useConnection();
  const subscriptionRef = useRef<number | null>(null);

  const handleAccountChange = useCallback(
    (accountInfo: AccountInfo<Buffer>) => {
      try {
        const data = accountInfo.data.slice(8);
        const decoded = schema.decode(Buffer.from(data));
        onUpdate(decoded);
      } catch (err) {
        console.error('Failed to decode account update:', err);
      }
    },
    [schema, onUpdate]
  );

  useEffect(() => {
    if (!pubkey) return;

    subscriptionRef.current = connection.onAccountChange(
      pubkey,
      handleAccountChange,
      'confirmed'
    );

    return () => {
      if (subscriptionRef.current !== null) {
        connection.removeAccountChangeListener(subscriptionRef.current);
      }
    };
  }, [connection, pubkey, handleAccountChange]);
}

// Usage
function LivePlayerCard({ playerPda }: { playerPda: PublicKey }) {
  const [player, setPlayer] = useState<PlayerAccount | null>(null);

  useAccountSubscription(playerPda, PlayerAccountSchema, setPlayer);

  // player updates in real-time!
  return <div>{player?.username}</div>;
}
```

### Batch Operations

```typescript
// src/utils/batch.ts
import { Connection, PublicKey } from '@solana/web3.js';

export async function batchFetchAccounts<T>(
  connection: Connection,
  pubkeys: PublicKey[],
  schema: { decode: (data: Buffer) => T },
  batchSize = 100
): Promise<(T | null)[]> {
  const results: (T | null)[] = [];

  // Process in batches to avoid RPC limits
  for (let i = 0; i < pubkeys.length; i += batchSize) {
    const batch = pubkeys.slice(i, i + batchSize);
    const accounts = await connection.getMultipleAccountsInfo(batch);

    for (const account of accounts) {
      if (account) {
        try {
          const data = account.data.slice(8);
          results.push(schema.decode(Buffer.from(data)));
        } catch {
          results.push(null);
        }
      } else {
        results.push(null);
      }
    }
  }

  return results;
}

// Usage
const players = await batchFetchAccounts(
  connection,
  playerPubkeys,
  PlayerAccountSchema
);
```

### Caching with React Query

```typescript
// src/hooks/usePlayerQuery.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { PublicKey } from '@solana/web3.js';
import { useConnection } from '@solana/wallet-adapter-react';
import { safeFetchAccount } from '../utils/fetch';
import { PlayerAccountSchema, PlayerAccount } from '../generated';

export function usePlayerQuery(pubkey: PublicKey | null) {
  const { connection } = useConnection();

  return useQuery({
    queryKey: ['player', pubkey?.toString()],
    queryFn: () =>
      pubkey
        ? safeFetchAccount(connection, pubkey, PlayerAccountSchema)
        : null,
    enabled: !!pubkey,
    staleTime: 30_000, // 30 seconds
    refetchInterval: 60_000, // 1 minute
  });
}

export function useUpdatePlayerMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (params: { pubkey: PublicKey; username: string }) => {
      // ... send transaction
    },
    onSuccess: (_, variables) => {
      // Invalidate and refetch
      queryClient.invalidateQueries({
        queryKey: ['player', variables.pubkey.toString()],
      });
    },
  });
}
```

### Optimistic Updates

```typescript
// src/hooks/useOptimisticPlayer.ts
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { PublicKey } from '@solana/web3.js';
import { PlayerAccount } from '../generated';

export function useGainExperience(playerPda: PublicKey) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (amount: number) => {
      // Send transaction to gain experience
      return program.methods.gainExperience(new BN(amount)).rpc();
    },
    onMutate: async (amount) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({
        queryKey: ['player', playerPda.toString()],
      });

      // Snapshot previous value
      const previousPlayer = queryClient.getQueryData<PlayerAccount>([
        'player',
        playerPda.toString(),
      ]);

      // Optimistically update
      if (previousPlayer) {
        queryClient.setQueryData<PlayerAccount>(
          ['player', playerPda.toString()],
          {
            ...previousPlayer,
            experience: previousPlayer.experience + amount,
            level: Math.floor((previousPlayer.experience + amount) / 1000) + 1,
          }
        );
      }

      return { previousPlayer };
    },
    onError: (err, amount, context) => {
      // Rollback on error
      if (context?.previousPlayer) {
        queryClient.setQueryData(
          ['player', playerPda.toString()],
          context.previousPlayer
        );
      }
    },
    onSettled: () => {
      // Refetch after mutation
      queryClient.invalidateQueries({
        queryKey: ['player', playerPda.toString()],
      });
    },
  });
}
```

---

## Troubleshooting

### Common Issues and Solutions

#### "Buffer is not defined"

**Cause:** Browser doesn't have Node.js Buffer

**Solution (Vite):**
```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import { nodePolyfills } from 'vite-plugin-node-polyfills';

export default defineConfig({
  plugins: [nodePolyfills()],
});
```

**Solution (webpack/CRA):**
```bash
npm install buffer
```

```typescript
// src/polyfills.ts
import { Buffer } from 'buffer';
window.Buffer = Buffer;
```

#### "Account discriminator mismatch"

**Cause:** Trying to deserialize non-Anchor account as Anchor, or wrong schema

**Solution:**
```typescript
// Check if account is Anchor format
const ANCHOR_DISCRIMINATOR_LENGTH = 8;

function isAnchorAccount(data: Buffer): boolean {
  return data.length > ANCHOR_DISCRIMINATOR_LENGTH;
}

// Adjust slice accordingly
const offset = isAnchorAccount(data) ? 8 : 0;
const decoded = schema.decode(data.slice(offset));
```

#### BigInt serialization in JSON

**Cause:** JSON.stringify doesn't support BigInt

**Solution:**
```typescript
// Custom serializer
function serializePlayer(player: PlayerAccount) {
  return JSON.stringify(player, (key, value) =>
    typeof value === 'bigint' ? value.toString() : value
  );
}

// Or use superjson
import superjson from 'superjson';
const serialized = superjson.stringify(player);
```

#### "Transaction too large"

**Cause:** Too many instructions or accounts

**Solution:**
```typescript
// Split into multiple transactions
async function batchTransactions(
  instructions: TransactionInstruction[],
  maxPerTx = 5
) {
  const batches: Transaction[] = [];

  for (let i = 0; i < instructions.length; i += maxPerTx) {
    const tx = new Transaction();
    instructions.slice(i, i + maxPerTx).forEach((ix) => tx.add(ix));
    batches.push(tx);
  }

  return batches;
}
```

#### CORS errors with RPC

**Cause:** RPC endpoint blocks browser requests

**Solution:**
```typescript
// Use a CORS-friendly RPC or proxy
const RPC_ENDPOINTS = {
  devnet: 'https://api.devnet.solana.com',
  // Or use Helius, QuickNode, etc.
  devnetPremium: 'https://devnet.helius-rpc.com/?api-key=YOUR_KEY',
};
```

#### Wallet not connecting

**Cause:** Wallet adapter misconfiguration

**Solution:**
```typescript
// Ensure proper provider hierarchy
<ConnectionProvider endpoint={endpoint}>
  <WalletProvider wallets={wallets} autoConnect>
    <WalletModalProvider>
      {/* Your app */}
    </WalletModalProvider>
  </WalletProvider>
</ConnectionProvider>
```

---

## Quick Reference

### Essential Imports

```typescript
// Solana
import { Connection, PublicKey, Transaction } from '@solana/web3.js';

// Borsh
import * as borsh from '@coral-xyz/borsh';

// Anchor
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';

// Wallet Adapter
import { useConnection, useWallet } from '@solana/wallet-adapter-react';

// Generated Types
import { PlayerAccount, PlayerAccountSchema } from './generated';
```

### Common Patterns

```typescript
// Fetch and deserialize
const accountInfo = await connection.getAccountInfo(pubkey);
const data = accountInfo.data.slice(8); // Skip Anchor discriminator
const player = PlayerAccountSchema.decode(Buffer.from(data));

// PDA derivation
const [pda] = PublicKey.findProgramAddressSync(
  [Buffer.from('seed'), wallet.toBuffer()],
  PROGRAM_ID
);

// Send transaction with Anchor
await program.methods
  .instructionName(arg1, arg2)
  .accounts({ account1, account2 })
  .rpc();
```

---

## Related Guides

Continue your LUMOS journey with these related guides:

**Integration Guides:**
- [LUMOS + Solana CLI Integration](/guides/solana-cli-integration) - Backend deployment workflow
- [LUMOS + Anchor Integration](/guides/anchor-integration) - Schema-first Anchor development

**Migration Guides:**
- [Migration: TypeScript → LUMOS](/guides/migration-typescript) - Migrate existing TypeScript types
- [Migration: Anchor → LUMOS](/guides/migration-anchor) - Add LUMOS to existing Anchor projects

**Use Case Guides:**
- [Gaming Projects](/guides/use-cases/gaming) - Complete gaming dApp with React frontend
- [NFT Marketplaces](/guides/use-cases/nft) - Metaplex-compatible marketplace
- [DeFi Protocols](/guides/use-cases/defi) - Staking dashboard implementation

---

**Last Updated:** 2025-12-16
**Version:** 1.0.0
