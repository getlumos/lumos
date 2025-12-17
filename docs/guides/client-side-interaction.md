# Client-Side Interaction Examples

Advanced patterns for building production-ready frontends with LUMOS-generated TypeScript types.

> **Prerequisites:** Read [web3js-integration.md](./web3js-integration.md) first for basics.
> This guide covers **advanced patterns** not in the basics guide.

---

## Table of Contents

1. [State Management Integration](#1-state-management-integration)
2. [Real-Time Subscription Patterns](#2-real-time-subscription-patterns)
3. [Transaction UI Flows](#3-transaction-ui-flows)
4. [Form Integration Patterns](#4-form-integration-patterns)
5. [Display & Formatting Utilities](#5-display--formatting-utilities)
6. [Framework-Specific Patterns](#6-framework-specific-patterns)
7. [Testing Client Code](#7-testing-client-code)
8. [Performance Optimization](#8-performance-optimization)
9. [Complete Example: Token Dashboard](#9-complete-example-token-dashboard)

---

## 1. State Management Integration

### 1.1 Zustand Store Pattern

Zustand provides lightweight state management perfect for Solana dApps.

```typescript
// stores/usePlayerStore.ts
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { Connection, PublicKey } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';
import type { PlayerAccount } from '../generated/schema';
import { PlayerAccountSchema } from '../generated/schema';

interface PlayerState {
  // Data
  player: PlayerAccount | null;
  address: PublicKey | null;
  isLoading: boolean;
  error: Error | null;
  lastUpdated: number | null;

  // Actions
  fetchPlayer: (connection: Connection, address: PublicKey) => Promise<void>;
  subscribe: (connection: Connection, address: PublicKey) => () => void;
  updateOptimistic: (update: Partial<PlayerAccount>) => void;
  rollback: () => void;
  reset: () => void;
}

// Store previous state for rollback
let previousPlayer: PlayerAccount | null = null;

export const usePlayerStore = create<PlayerState>()(
  subscribeWithSelector((set, get) => ({
    player: null,
    address: null,
    isLoading: false,
    error: null,
    lastUpdated: null,

    fetchPlayer: async (connection, address) => {
      set({ isLoading: true, error: null, address });

      try {
        const accountInfo = await connection.getAccountInfo(address);

        if (!accountInfo) {
          throw new Error('Account not found');
        }

        // Skip 8-byte Anchor discriminator
        const data = accountInfo.data.slice(8);
        const player = PlayerAccountSchema.decode(data);

        set({
          player,
          isLoading: false,
          lastUpdated: Date.now()
        });
      } catch (error) {
        set({
          error: error as Error,
          isLoading: false
        });
      }
    },

    subscribe: (connection, address) => {
      const subscriptionId = connection.onAccountChange(
        address,
        (accountInfo) => {
          try {
            const data = accountInfo.data.slice(8);
            const player = PlayerAccountSchema.decode(data);
            set({ player, lastUpdated: Date.now() });
          } catch (error) {
            set({ error: error as Error });
          }
        },
        'confirmed'
      );

      // Return cleanup function
      return () => {
        connection.removeAccountChangeListener(subscriptionId);
      };
    },

    updateOptimistic: (update) => {
      const { player } = get();
      if (player) {
        previousPlayer = { ...player };
        set({ player: { ...player, ...update } });
      }
    },

    rollback: () => {
      if (previousPlayer) {
        set({ player: previousPlayer });
        previousPlayer = null;
      }
    },

    reset: () => {
      set({
        player: null,
        address: null,
        isLoading: false,
        error: null,
        lastUpdated: null,
      });
    },
  }))
);

// Selector hooks for granular subscriptions
export const usePlayer = () => usePlayerStore((s) => s.player);
export const usePlayerLoading = () => usePlayerStore((s) => s.isLoading);
export const usePlayerError = () => usePlayerStore((s) => s.error);
```

**Usage in Components:**

```tsx
// components/PlayerCard.tsx
import { useEffect } from 'react';
import { useConnection } from '@solana/wallet-adapter-react';
import { usePlayerStore, usePlayer, usePlayerLoading } from '../stores/usePlayerStore';

export function PlayerCard({ address }: { address: PublicKey }) {
  const { connection } = useConnection();
  const player = usePlayer();
  const isLoading = usePlayerLoading();
  const { fetchPlayer, subscribe } = usePlayerStore();

  useEffect(() => {
    fetchPlayer(connection, address);
    const unsubscribe = subscribe(connection, address);
    return unsubscribe;
  }, [connection, address]);

  if (isLoading) return <div>Loading...</div>;
  if (!player) return <div>No player found</div>;

  return (
    <div>
      <h2>Level {player.level}</h2>
      <p>XP: {player.experience.toString()}</p>
    </div>
  );
}
```

---

### 1.2 TanStack Query (React Query)

Best for caching, background refetching, and request deduplication.

```typescript
// hooks/usePlayerQuery.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useConnection } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import type { PlayerAccount } from '../generated/schema';
import { PlayerAccountSchema } from '../generated/schema';

// Query key factory
export const playerKeys = {
  all: ['players'] as const,
  detail: (address: string) => [...playerKeys.all, address] as const,
  inventory: (address: string) => [...playerKeys.detail(address), 'inventory'] as const,
};

// Fetch function
async function fetchPlayer(
  connection: Connection,
  address: PublicKey
): Promise<PlayerAccount> {
  const accountInfo = await connection.getAccountInfo(address);

  if (!accountInfo) {
    throw new Error('Player account not found');
  }

  const data = accountInfo.data.slice(8);
  return PlayerAccountSchema.decode(data);
}

// Query hook
export function usePlayerQuery(address: PublicKey | null) {
  const { connection } = useConnection();

  return useQuery({
    queryKey: playerKeys.detail(address?.toBase58() ?? ''),
    queryFn: () => fetchPlayer(connection, address!),
    enabled: !!address,
    staleTime: 30_000, // 30 seconds
    gcTime: 5 * 60 * 1000, // 5 minutes (formerly cacheTime)
    refetchOnWindowFocus: true,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

// Mutation with optimistic update
export function useLevelUpMutation() {
  const queryClient = useQueryClient();
  const { connection } = useConnection();

  return useMutation({
    mutationFn: async ({
      address,
      program
    }: {
      address: PublicKey;
      program: Program
    }) => {
      const tx = await program.methods
        .levelUp()
        .accounts({ player: address })
        .rpc();

      // Wait for confirmation
      await connection.confirmTransaction(tx, 'confirmed');
      return tx;
    },

    onMutate: async ({ address }) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({
        queryKey: playerKeys.detail(address.toBase58())
      });

      // Snapshot previous value
      const previousPlayer = queryClient.getQueryData<PlayerAccount>(
        playerKeys.detail(address.toBase58())
      );

      // Optimistically update
      if (previousPlayer) {
        queryClient.setQueryData<PlayerAccount>(
          playerKeys.detail(address.toBase58()),
          {
            ...previousPlayer,
            level: previousPlayer.level + 1,
          }
        );
      }

      return { previousPlayer };
    },

    onError: (err, { address }, context) => {
      // Rollback on error
      if (context?.previousPlayer) {
        queryClient.setQueryData(
          playerKeys.detail(address.toBase58()),
          context.previousPlayer
        );
      }
    },

    onSettled: (_, __, { address }) => {
      // Refetch to sync with chain
      queryClient.invalidateQueries({
        queryKey: playerKeys.detail(address.toBase58())
      });
    },
  });
}
```

**Usage:**

```tsx
function PlayerProfile({ address }: { address: PublicKey }) {
  const { data: player, isLoading, error } = usePlayerQuery(address);
  const levelUp = useLevelUpMutation();

  if (isLoading) return <Skeleton />;
  if (error) return <ErrorMessage error={error} />;
  if (!player) return null;

  return (
    <div>
      <h2>Level {player.level}</h2>
      <button
        onClick={() => levelUp.mutate({ address, program })}
        disabled={levelUp.isPending}
      >
        {levelUp.isPending ? 'Leveling up...' : 'Level Up'}
      </button>
    </div>
  );
}
```

---

### 1.3 Redux Toolkit Slice

For larger apps with complex state interactions.

```typescript
// store/playerSlice.ts
import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit';
import { Connection, PublicKey } from '@solana/web3.js';
import type { PlayerAccount } from '../generated/schema';
import { PlayerAccountSchema } from '../generated/schema';

interface PlayerState {
  data: PlayerAccount | null;
  address: string | null;
  status: 'idle' | 'loading' | 'succeeded' | 'failed';
  error: string | null;
}

const initialState: PlayerState = {
  data: null,
  address: null,
  status: 'idle',
  error: null,
};

export const fetchPlayer = createAsyncThunk(
  'player/fetch',
  async ({
    connection,
    address
  }: {
    connection: Connection;
    address: PublicKey
  }) => {
    const accountInfo = await connection.getAccountInfo(address);
    if (!accountInfo) throw new Error('Account not found');

    const data = accountInfo.data.slice(8);
    const player = PlayerAccountSchema.decode(data);

    // Serialize PublicKey for Redux (must be serializable)
    return {
      ...player,
      wallet: player.wallet.toBase58(),
      guild: player.guild?.toBase58() ?? null,
    };
  }
);

const playerSlice = createSlice({
  name: 'player',
  initialState,
  reducers: {
    setOptimisticLevel: (state, action: PayloadAction<number>) => {
      if (state.data) {
        state.data.level = action.payload;
      }
    },
    clearPlayer: () => initialState,
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchPlayer.pending, (state) => {
        state.status = 'loading';
      })
      .addCase(fetchPlayer.fulfilled, (state, action) => {
        state.status = 'succeeded';
        state.data = action.payload;
        state.error = null;
      })
      .addCase(fetchPlayer.rejected, (state, action) => {
        state.status = 'failed';
        state.error = action.error.message ?? 'Failed to fetch';
      });
  },
});

export const { setOptimisticLevel, clearPlayer } = playerSlice.actions;
export default playerSlice.reducer;

// Selectors
export const selectPlayer = (state: RootState) => state.player.data;
export const selectPlayerStatus = (state: RootState) => state.player.status;
```

---

### 1.4 Vue Pinia Store

```typescript
// stores/player.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { Connection, PublicKey } from '@solana/web3.js';
import type { PlayerAccount } from '../generated/schema';
import { PlayerAccountSchema } from '../generated/schema';

export const usePlayerStore = defineStore('player', () => {
  // State
  const player = ref<PlayerAccount | null>(null);
  const isLoading = ref(false);
  const error = ref<Error | null>(null);

  // Getters
  const level = computed(() => player.value?.level ?? 0);
  const hasGuild = computed(() => !!player.value?.guild);

  // Actions
  async function fetchPlayer(connection: Connection, address: PublicKey) {
    isLoading.value = true;
    error.value = null;

    try {
      const accountInfo = await connection.getAccountInfo(address);
      if (!accountInfo) throw new Error('Not found');

      const data = accountInfo.data.slice(8);
      player.value = PlayerAccountSchema.decode(data);
    } catch (e) {
      error.value = e as Error;
    } finally {
      isLoading.value = false;
    }
  }

  function subscribe(connection: Connection, address: PublicKey) {
    const id = connection.onAccountChange(address, (info) => {
      const data = info.data.slice(8);
      player.value = PlayerAccountSchema.decode(data);
    });

    return () => connection.removeAccountChangeListener(id);
  }

  function $reset() {
    player.value = null;
    isLoading.value = false;
    error.value = null;
  }

  return {
    player,
    isLoading,
    error,
    level,
    hasGuild,
    fetchPlayer,
    subscribe,
    $reset,
  };
});
```

**Vue Component Usage:**

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { usePlayerStore } from '../stores/player';
import { useConnection } from '../composables/useConnection';

const props = defineProps<{ address: PublicKey }>();

const playerStore = usePlayerStore();
const { connection } = useConnection();

let unsubscribe: (() => void) | null = null;

onMounted(async () => {
  await playerStore.fetchPlayer(connection, props.address);
  unsubscribe = playerStore.subscribe(connection, props.address);
});

onUnmounted(() => {
  unsubscribe?.();
});
</script>

<template>
  <div v-if="playerStore.isLoading">Loading...</div>
  <div v-else-if="playerStore.error">{{ playerStore.error.message }}</div>
  <div v-else-if="playerStore.player">
    <h2>Level {{ playerStore.level }}</h2>
  </div>
</template>
```

---

### 1.5 Svelte Stores

```typescript
// stores/player.ts
import { writable, derived } from 'svelte/store';
import { Connection, PublicKey } from '@solana/web3.js';
import type { PlayerAccount } from '../generated/schema';
import { PlayerAccountSchema } from '../generated/schema';

interface PlayerState {
  player: PlayerAccount | null;
  isLoading: boolean;
  error: Error | null;
}

function createPlayerStore() {
  const { subscribe, set, update } = writable<PlayerState>({
    player: null,
    isLoading: false,
    error: null,
  });

  let subscriptionId: number | null = null;

  return {
    subscribe,

    async fetch(connection: Connection, address: PublicKey) {
      update(s => ({ ...s, isLoading: true, error: null }));

      try {
        const info = await connection.getAccountInfo(address);
        if (!info) throw new Error('Not found');

        const data = info.data.slice(8);
        const player = PlayerAccountSchema.decode(data);

        update(s => ({ ...s, player, isLoading: false }));
      } catch (error) {
        update(s => ({ ...s, error: error as Error, isLoading: false }));
      }
    },

    subscribeToAccount(connection: Connection, address: PublicKey) {
      subscriptionId = connection.onAccountChange(address, (info) => {
        const data = info.data.slice(8);
        const player = PlayerAccountSchema.decode(data);
        update(s => ({ ...s, player }));
      });
    },

    unsubscribe(connection: Connection) {
      if (subscriptionId !== null) {
        connection.removeAccountChangeListener(subscriptionId);
        subscriptionId = null;
      }
    },

    reset() {
      set({ player: null, isLoading: false, error: null });
    },
  };
}

export const playerStore = createPlayerStore();

// Derived stores for convenience
export const player = derived(playerStore, $s => $s.player);
export const playerLevel = derived(playerStore, $s => $s.player?.level ?? 0);
```

**Svelte Component:**

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { playerStore, player, playerLevel } from '../stores/player';
  import { connection } from '../lib/connection';

  export let address: PublicKey;

  onMount(() => {
    playerStore.fetch(connection, address);
    playerStore.subscribeToAccount(connection, address);
  });

  onDestroy(() => {
    playerStore.unsubscribe(connection);
  });
</script>

{#if $playerStore.isLoading}
  <p>Loading...</p>
{:else if $playerStore.error}
  <p class="error">{$playerStore.error.message}</p>
{:else if $player}
  <div class="player-card">
    <h2>Level {$playerLevel}</h2>
    <p>XP: {$player.experience.toString()}</p>
  </div>
{/if}
```

---

## 2. Real-Time Subscription Patterns

### 2.1 Single Account Subscription with Cleanup

```typescript
// hooks/useAccountSubscription.ts
import { useEffect, useRef, useCallback } from 'react';
import { Connection, PublicKey, AccountInfo } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';

interface UseAccountSubscriptionOptions<T> {
  connection: Connection;
  address: PublicKey | null;
  schema: borsh.Layout<T>;
  discriminatorSize?: number;
  onUpdate: (data: T) => void;
  onError?: (error: Error) => void;
  commitment?: 'processed' | 'confirmed' | 'finalized';
}

export function useAccountSubscription<T>({
  connection,
  address,
  schema,
  discriminatorSize = 8,
  onUpdate,
  onError,
  commitment = 'confirmed',
}: UseAccountSubscriptionOptions<T>) {
  const subscriptionRef = useRef<number | null>(null);
  const isMountedRef = useRef(true);

  const handleAccountChange = useCallback(
    (accountInfo: AccountInfo<Buffer>) => {
      if (!isMountedRef.current) return;

      try {
        const data = accountInfo.data.slice(discriminatorSize);
        const decoded = schema.decode(data);
        onUpdate(decoded);
      } catch (error) {
        onError?.(error as Error);
      }
    },
    [schema, discriminatorSize, onUpdate, onError]
  );

  useEffect(() => {
    isMountedRef.current = true;

    if (!address) return;

    // Subscribe
    subscriptionRef.current = connection.onAccountChange(
      address,
      handleAccountChange,
      commitment
    );

    // Cleanup
    return () => {
      isMountedRef.current = false;
      if (subscriptionRef.current !== null) {
        connection.removeAccountChangeListener(subscriptionRef.current);
        subscriptionRef.current = null;
      }
    };
  }, [connection, address, commitment, handleAccountChange]);

  // Manual unsubscribe function
  const unsubscribe = useCallback(() => {
    if (subscriptionRef.current !== null) {
      connection.removeAccountChangeListener(subscriptionRef.current);
      subscriptionRef.current = null;
    }
  }, [connection]);

  return { unsubscribe };
}
```

**Usage:**

```tsx
function GameRoom({ playerAddress }: { playerAddress: PublicKey }) {
  const { connection } = useConnection();
  const [player, setPlayer] = useState<PlayerAccount | null>(null);

  useAccountSubscription({
    connection,
    address: playerAddress,
    schema: PlayerAccountSchema,
    onUpdate: setPlayer,
    onError: (err) => console.error('Subscription error:', err),
  });

  return player ? <PlayerDisplay player={player} /> : <Loading />;
}
```

---

### 2.2 Multi-Account Subscription Coordinator

```typescript
// hooks/useMultiAccountSubscription.ts
import { useEffect, useRef } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';

interface AccountSubscription<T> {
  address: PublicKey;
  schema: borsh.Layout<T>;
  onUpdate: (data: T) => void;
}

export function useMultiAccountSubscription<T>(
  connection: Connection,
  subscriptions: AccountSubscription<T>[],
  options?: { discriminatorSize?: number }
) {
  const subscriptionIds = useRef<Map<string, number>>(new Map());
  const discriminatorSize = options?.discriminatorSize ?? 8;

  useEffect(() => {
    // Subscribe to all accounts
    for (const sub of subscriptions) {
      const key = sub.address.toBase58();

      // Skip if already subscribed
      if (subscriptionIds.current.has(key)) continue;

      const id = connection.onAccountChange(
        sub.address,
        (accountInfo) => {
          try {
            const data = accountInfo.data.slice(discriminatorSize);
            const decoded = sub.schema.decode(data);
            sub.onUpdate(decoded);
          } catch (e) {
            console.error(`Error decoding ${key}:`, e);
          }
        },
        'confirmed'
      );

      subscriptionIds.current.set(key, id);
    }

    // Cleanup removed subscriptions
    const currentAddresses = new Set(
      subscriptions.map(s => s.address.toBase58())
    );

    for (const [key, id] of subscriptionIds.current) {
      if (!currentAddresses.has(key)) {
        connection.removeAccountChangeListener(id);
        subscriptionIds.current.delete(key);
      }
    }

    // Cleanup all on unmount
    return () => {
      for (const id of subscriptionIds.current.values()) {
        connection.removeAccountChangeListener(id);
      }
      subscriptionIds.current.clear();
    };
  }, [connection, subscriptions, discriminatorSize]);
}
```

**Usage - Watching Multiple Players:**

```tsx
function Leaderboard({ playerAddresses }: { playerAddresses: PublicKey[] }) {
  const { connection } = useConnection();
  const [players, setPlayers] = useState<Map<string, PlayerAccount>>(new Map());

  const subscriptions = useMemo(
    () =>
      playerAddresses.map((address) => ({
        address,
        schema: PlayerAccountSchema,
        onUpdate: (data: PlayerAccount) => {
          setPlayers((prev) => new Map(prev).set(address.toBase58(), data));
        },
      })),
    [playerAddresses]
  );

  useMultiAccountSubscription(connection, subscriptions);

  const sorted = [...players.values()].sort((a, b) => b.level - a.level);

  return (
    <ol>
      {sorted.map((p, i) => (
        <li key={i}>Level {p.level} - {p.experience.toString()} XP</li>
      ))}
    </ol>
  );
}
```

---

### 2.3 Polling with Exponential Backoff

For when WebSocket subscriptions are unreliable:

```typescript
// hooks/usePolledAccount.ts
import { useEffect, useRef, useCallback } from 'react';
import { Connection, PublicKey } from '@solana/web3.js';

interface UsePolledAccountOptions<T> {
  connection: Connection;
  address: PublicKey | null;
  schema: borsh.Layout<T>;
  onUpdate: (data: T) => void;
  onError?: (error: Error) => void;
  initialInterval?: number;
  maxInterval?: number;
  backoffMultiplier?: number;
}

export function usePolledAccount<T>({
  connection,
  address,
  schema,
  onUpdate,
  onError,
  initialInterval = 2000,
  maxInterval = 30000,
  backoffMultiplier = 1.5,
}: UsePolledAccountOptions<T>) {
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const currentInterval = useRef(initialInterval);
  const consecutiveErrors = useRef(0);

  const poll = useCallback(async () => {
    if (!address) return;

    try {
      const info = await connection.getAccountInfo(address);
      if (!info) throw new Error('Account not found');

      const data = info.data.slice(8);
      const decoded = schema.decode(data);
      onUpdate(decoded);

      // Reset on success
      consecutiveErrors.current = 0;
      currentInterval.current = initialInterval;
    } catch (error) {
      consecutiveErrors.current++;
      onError?.(error as Error);

      // Exponential backoff
      currentInterval.current = Math.min(
        currentInterval.current * backoffMultiplier,
        maxInterval
      );
    }

    // Schedule next poll
    intervalRef.current = setTimeout(poll, currentInterval.current);
  }, [
    connection,
    address,
    schema,
    onUpdate,
    onError,
    initialInterval,
    maxInterval,
    backoffMultiplier,
  ]);

  useEffect(() => {
    if (address) {
      poll(); // Initial fetch
    }

    return () => {
      if (intervalRef.current) {
        clearTimeout(intervalRef.current);
      }
    };
  }, [address, poll]);

  // Force immediate refresh
  const refresh = useCallback(() => {
    if (intervalRef.current) {
      clearTimeout(intervalRef.current);
    }
    currentInterval.current = initialInterval;
    poll();
  }, [poll, initialInterval]);

  return { refresh };
}
```

---

### 2.4 Optimistic Updates with Reconciliation

```typescript
// hooks/useOptimisticAccount.ts
import { useState, useCallback, useRef } from 'react';

interface OptimisticState<T> {
  confirmed: T | null;
  optimistic: T | null;
  pending: boolean;
}

export function useOptimisticAccount<T>(initial: T | null = null) {
  const [state, setState] = useState<OptimisticState<T>>({
    confirmed: initial,
    optimistic: null,
    pending: false,
  });

  const rollbackTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Apply optimistic update
  const applyOptimistic = useCallback((update: Partial<T>) => {
    setState((prev) => ({
      ...prev,
      optimistic: prev.confirmed
        ? { ...prev.confirmed, ...update }
        : null,
      pending: true,
    }));

    // Auto-rollback after timeout (e.g., 30s)
    rollbackTimeoutRef.current = setTimeout(() => {
      setState((prev) => ({
        ...prev,
        optimistic: null,
        pending: false,
      }));
    }, 30000);
  }, []);

  // Confirm with chain data
  const confirm = useCallback((chainData: T) => {
    if (rollbackTimeoutRef.current) {
      clearTimeout(rollbackTimeoutRef.current);
    }

    setState({
      confirmed: chainData,
      optimistic: null,
      pending: false,
    });
  }, []);

  // Manual rollback
  const rollback = useCallback(() => {
    if (rollbackTimeoutRef.current) {
      clearTimeout(rollbackTimeoutRef.current);
    }

    setState((prev) => ({
      ...prev,
      optimistic: null,
      pending: false,
    }));
  }, []);

  // Current display value (optimistic if pending, else confirmed)
  const current = state.optimistic ?? state.confirmed;

  return {
    current,
    confirmed: state.confirmed,
    pending: state.pending,
    applyOptimistic,
    confirm,
    rollback,
  };
}
```

**Usage:**

```tsx
function LevelUpButton({ player, address }: Props) {
  const { connection } = useConnection();
  const { current, pending, applyOptimistic, confirm, rollback } =
    useOptimisticAccount(player);

  const handleLevelUp = async () => {
    // Optimistic update
    applyOptimistic({ level: player.level + 1 });

    try {
      const tx = await program.methods.levelUp().accounts({ player: address }).rpc();
      await connection.confirmTransaction(tx, 'confirmed');

      // Fetch confirmed state
      const info = await connection.getAccountInfo(address);
      const data = info!.data.slice(8);
      const confirmed = PlayerAccountSchema.decode(data);
      confirm(confirmed);
    } catch (error) {
      rollback();
      toast.error('Level up failed');
    }
  };

  return (
    <div>
      <p>Level: {current?.level} {pending && '(pending...)'}</p>
      <button onClick={handleLevelUp} disabled={pending}>
        Level Up
      </button>
    </div>
  );
}
```

---

## 3. Transaction UI Flows

### 3.1 Multi-Step Transaction Wizard

```typescript
// hooks/useTransactionWizard.ts
import { useState, useCallback } from 'react';
import { Transaction, TransactionSignature } from '@solana/web3.js';

type WizardStep = 'idle' | 'building' | 'simulating' | 'signing' | 'sending' | 'confirming' | 'success' | 'error';

interface WizardState {
  step: WizardStep;
  transaction: Transaction | null;
  signature: TransactionSignature | null;
  simulationResult: SimulationResult | null;
  error: Error | null;
  estimatedFee: number | null;
}

interface SimulationResult {
  success: boolean;
  logs: string[];
  unitsConsumed: number;
}

export function useTransactionWizard() {
  const [state, setState] = useState<WizardState>({
    step: 'idle',
    transaction: null,
    signature: null,
    simulationResult: null,
    error: null,
    estimatedFee: null,
  });

  const build = useCallback(async (
    buildFn: () => Promise<Transaction>
  ) => {
    setState(s => ({ ...s, step: 'building', error: null }));

    try {
      const transaction = await buildFn();
      setState(s => ({ ...s, step: 'simulating', transaction }));
      return transaction;
    } catch (error) {
      setState(s => ({ ...s, step: 'error', error: error as Error }));
      throw error;
    }
  }, []);

  const simulate = useCallback(async (
    connection: Connection,
    transaction: Transaction
  ): Promise<SimulationResult> => {
    try {
      const result = await connection.simulateTransaction(transaction);

      const simulationResult: SimulationResult = {
        success: !result.value.err,
        logs: result.value.logs ?? [],
        unitsConsumed: result.value.unitsConsumed ?? 0,
      };

      // Estimate fee
      const fee = await transaction.getEstimatedFee(connection);

      setState(s => ({
        ...s,
        simulationResult,
        estimatedFee: fee,
        step: simulationResult.success ? 'signing' : 'error',
        error: simulationResult.success ? null : new Error('Simulation failed'),
      }));

      return simulationResult;
    } catch (error) {
      setState(s => ({ ...s, step: 'error', error: error as Error }));
      throw error;
    }
  }, []);

  const sign = useCallback(async (
    signTransaction: (tx: Transaction) => Promise<Transaction>,
    transaction: Transaction
  ) => {
    setState(s => ({ ...s, step: 'signing' }));

    try {
      const signed = await signTransaction(transaction);
      setState(s => ({ ...s, step: 'sending', transaction: signed }));
      return signed;
    } catch (error) {
      setState(s => ({ ...s, step: 'error', error: error as Error }));
      throw error;
    }
  }, []);

  const send = useCallback(async (
    connection: Connection,
    transaction: Transaction
  ) => {
    setState(s => ({ ...s, step: 'sending' }));

    try {
      const signature = await connection.sendRawTransaction(
        transaction.serialize()
      );
      setState(s => ({ ...s, step: 'confirming', signature }));
      return signature;
    } catch (error) {
      setState(s => ({ ...s, step: 'error', error: error as Error }));
      throw error;
    }
  }, []);

  const confirm = useCallback(async (
    connection: Connection,
    signature: TransactionSignature
  ) => {
    try {
      const result = await connection.confirmTransaction(signature, 'confirmed');

      if (result.value.err) {
        throw new Error(`Transaction failed: ${JSON.stringify(result.value.err)}`);
      }

      setState(s => ({ ...s, step: 'success' }));
    } catch (error) {
      setState(s => ({ ...s, step: 'error', error: error as Error }));
      throw error;
    }
  }, []);

  const reset = useCallback(() => {
    setState({
      step: 'idle',
      transaction: null,
      signature: null,
      simulationResult: null,
      error: null,
      estimatedFee: null,
    });
  }, []);

  return {
    ...state,
    build,
    simulate,
    sign,
    send,
    confirm,
    reset,
  };
}
```

**Wizard Component:**

```tsx
function TransactionWizard({ onBuild }: { onBuild: () => Promise<Transaction> }) {
  const { connection } = useConnection();
  const { signTransaction } = useWallet();
  const wizard = useTransactionWizard();

  const steps = [
    { key: 'building', label: 'Building' },
    { key: 'simulating', label: 'Simulating' },
    { key: 'signing', label: 'Sign' },
    { key: 'sending', label: 'Sending' },
    { key: 'confirming', label: 'Confirming' },
  ];

  const handleStart = async () => {
    try {
      const tx = await wizard.build(onBuild);
      const sim = await wizard.simulate(connection, tx);

      if (!sim.success) return;

      const signed = await wizard.sign(signTransaction, tx);
      const sig = await wizard.send(connection, signed);
      await wizard.confirm(connection, sig);
    } catch (e) {
      // Error already captured in state
    }
  };

  return (
    <div className="wizard">
      {/* Progress Steps */}
      <div className="steps">
        {steps.map((step, i) => (
          <div
            key={step.key}
            className={`step ${wizard.step === step.key ? 'active' : ''}`}
          >
            {step.label}
          </div>
        ))}
      </div>

      {/* Content by Step */}
      {wizard.step === 'idle' && (
        <button onClick={handleStart}>Start Transaction</button>
      )}

      {wizard.step === 'simulating' && wizard.simulationResult && (
        <div>
          <p>Estimated compute: {wizard.simulationResult.unitsConsumed} units</p>
          <p>Estimated fee: {wizard.estimatedFee} lamports</p>
        </div>
      )}

      {wizard.step === 'signing' && (
        <p>Please approve the transaction in your wallet...</p>
      )}

      {wizard.step === 'confirming' && (
        <div>
          <Spinner />
          <p>Waiting for confirmation...</p>
          <a href={`https://solscan.io/tx/${wizard.signature}`} target="_blank">
            View on Solscan
          </a>
        </div>
      )}

      {wizard.step === 'success' && (
        <div className="success">
          <CheckIcon />
          <p>Transaction confirmed!</p>
          <button onClick={wizard.reset}>Done</button>
        </div>
      )}

      {wizard.step === 'error' && (
        <div className="error">
          <p>Error: {wizard.error?.message}</p>
          <button onClick={wizard.reset}>Try Again</button>
        </div>
      )}
    </div>
  );
}
```

---

### 3.2 Transaction Simulation Preview

```typescript
// components/SimulationPreview.tsx
import { useState } from 'react';
import { Transaction, Connection } from '@solana/web3.js';

interface SimulationPreviewProps {
  connection: Connection;
  transaction: Transaction;
  onConfirm: () => void;
  onCancel: () => void;
}

export function SimulationPreview({
  connection,
  transaction,
  onConfirm,
  onCancel,
}: SimulationPreviewProps) {
  const [result, setResult] = useState<{
    success: boolean;
    logs: string[];
    unitsConsumed: number;
    fee: number;
  } | null>(null);
  const [isSimulating, setIsSimulating] = useState(false);

  const runSimulation = async () => {
    setIsSimulating(true);

    try {
      const [simResult, fee] = await Promise.all([
        connection.simulateTransaction(transaction),
        transaction.getEstimatedFee(connection),
      ]);

      setResult({
        success: !simResult.value.err,
        logs: simResult.value.logs ?? [],
        unitsConsumed: simResult.value.unitsConsumed ?? 0,
        fee: fee ?? 5000,
      });
    } finally {
      setIsSimulating(false);
    }
  };

  return (
    <div className="simulation-preview">
      <h3>Transaction Preview</h3>

      {!result && (
        <button onClick={runSimulation} disabled={isSimulating}>
          {isSimulating ? 'Simulating...' : 'Simulate Transaction'}
        </button>
      )}

      {result && (
        <>
          <div className={`status ${result.success ? 'success' : 'error'}`}>
            {result.success ? '✓ Simulation Passed' : '✗ Simulation Failed'}
          </div>

          <div className="details">
            <div className="row">
              <span>Compute Units:</span>
              <span>{result.unitsConsumed.toLocaleString()}</span>
            </div>
            <div className="row">
              <span>Estimated Fee:</span>
              <span>{(result.fee / 1e9).toFixed(6)} SOL</span>
            </div>
          </div>

          {result.logs.length > 0 && (
            <details>
              <summary>Transaction Logs ({result.logs.length})</summary>
              <pre className="logs">
                {result.logs.map((log, i) => (
                  <div key={i}>{log}</div>
                ))}
              </pre>
            </details>
          )}

          <div className="actions">
            <button onClick={onCancel} className="secondary">
              Cancel
            </button>
            <button
              onClick={onConfirm}
              disabled={!result.success}
              className="primary"
            >
              Confirm & Sign
            </button>
          </div>
        </>
      )}
    </div>
  );
}
```

---

### 3.3 Transaction State Machine

```typescript
// machines/transactionMachine.ts
// Using XState for complex transaction flows

import { createMachine, assign } from 'xstate';
import { Transaction, TransactionSignature } from '@solana/web3.js';

interface TransactionContext {
  transaction: Transaction | null;
  signature: TransactionSignature | null;
  error: Error | null;
  retryCount: number;
}

type TransactionEvent =
  | { type: 'BUILD'; buildFn: () => Promise<Transaction> }
  | { type: 'SIMULATE' }
  | { type: 'SIGN' }
  | { type: 'SEND' }
  | { type: 'RETRY' }
  | { type: 'RESET' }
  | { type: 'SUCCESS'; signature: string }
  | { type: 'ERROR'; error: Error };

export const transactionMachine = createMachine<TransactionContext, TransactionEvent>({
  id: 'transaction',
  initial: 'idle',
  context: {
    transaction: null,
    signature: null,
    error: null,
    retryCount: 0,
  },
  states: {
    idle: {
      on: {
        BUILD: 'building',
      },
    },
    building: {
      invoke: {
        src: 'buildTransaction',
        onDone: {
          target: 'simulating',
          actions: assign({ transaction: (_, event) => event.data }),
        },
        onError: {
          target: 'error',
          actions: assign({ error: (_, event) => event.data }),
        },
      },
    },
    simulating: {
      invoke: {
        src: 'simulateTransaction',
        onDone: 'awaitingSignature',
        onError: {
          target: 'error',
          actions: assign({ error: (_, event) => event.data }),
        },
      },
    },
    awaitingSignature: {
      on: {
        SIGN: 'signing',
        RESET: 'idle',
      },
    },
    signing: {
      invoke: {
        src: 'signTransaction',
        onDone: 'sending',
        onError: {
          target: 'error',
          actions: assign({ error: (_, event) => event.data }),
        },
      },
    },
    sending: {
      invoke: {
        src: 'sendTransaction',
        onDone: {
          target: 'confirming',
          actions: assign({ signature: (_, event) => event.data }),
        },
        onError: {
          target: 'error',
          actions: assign({ error: (_, event) => event.data }),
        },
      },
    },
    confirming: {
      invoke: {
        src: 'confirmTransaction',
        onDone: 'success',
        onError: {
          target: 'error',
          actions: assign({ error: (_, event) => event.data }),
        },
      },
    },
    success: {
      on: {
        RESET: {
          target: 'idle',
          actions: assign({
            transaction: null,
            signature: null,
            error: null,
            retryCount: 0,
          }),
        },
      },
    },
    error: {
      on: {
        RETRY: {
          target: 'building',
          cond: (ctx) => ctx.retryCount < 3,
          actions: assign({ retryCount: (ctx) => ctx.retryCount + 1 }),
        },
        RESET: {
          target: 'idle',
          actions: assign({
            transaction: null,
            signature: null,
            error: null,
            retryCount: 0,
          }),
        },
      },
    },
  },
});
```

---

### 3.4 Transaction History Component

```typescript
// hooks/useTransactionHistory.ts
import { useState, useEffect } from 'react';
import { Connection, PublicKey, ParsedTransactionWithMeta } from '@solana/web3.js';

interface TransactionHistoryItem {
  signature: string;
  timestamp: number | null;
  status: 'success' | 'failed';
  type: string;
  fee: number;
}

export function useTransactionHistory(
  connection: Connection,
  address: PublicKey | null,
  limit: number = 10
) {
  const [transactions, setTransactions] = useState<TransactionHistoryItem[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!address) return;

    const fetchHistory = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const signatures = await connection.getSignaturesForAddress(
          address,
          { limit }
        );

        const items: TransactionHistoryItem[] = signatures.map((sig) => ({
          signature: sig.signature,
          timestamp: sig.blockTime,
          status: sig.err ? 'failed' : 'success',
          type: 'unknown', // Would need to parse logs for type
          fee: 0, // Would need to fetch full tx for fee
        }));

        setTransactions(items);
      } catch (err) {
        setError(err as Error);
      } finally {
        setIsLoading(false);
      }
    };

    fetchHistory();
  }, [connection, address, limit]);

  return { transactions, isLoading, error };
}
```

**History Display:**

```tsx
function TransactionHistory({ address }: { address: PublicKey }) {
  const { connection } = useConnection();
  const { transactions, isLoading } = useTransactionHistory(connection, address);

  if (isLoading) return <Skeleton count={5} />;

  return (
    <div className="tx-history">
      <h3>Recent Transactions</h3>
      {transactions.map((tx) => (
        <div key={tx.signature} className="tx-item">
          <div className={`status ${tx.status}`}>
            {tx.status === 'success' ? '✓' : '✗'}
          </div>
          <div className="details">
            <code>{tx.signature.slice(0, 8)}...{tx.signature.slice(-8)}</code>
            <span className="time">
              {tx.timestamp
                ? new Date(tx.timestamp * 1000).toLocaleString()
                : 'Pending'}
            </span>
          </div>
          <a
            href={`https://solscan.io/tx/${tx.signature}`}
            target="_blank"
            rel="noopener"
          >
            View
          </a>
        </div>
      ))}
    </div>
  );
}
```

---

## 4. Form Integration Patterns

### 4.1 React Hook Form + LUMOS Types

```typescript
// components/PlayerForm.tsx
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { PublicKey } from '@solana/web3.js';
import type { PlayerAccount } from '../generated/schema';

// Zod schema matching LUMOS type
const playerFormSchema = z.object({
  wallet: z.string().refine(
    (val) => {
      try {
        new PublicKey(val);
        return true;
      } catch {
        return false;
      }
    },
    { message: 'Invalid public key' }
  ),
  level: z.number().min(1).max(100),
  experience: z.string().refine(
    (val) => {
      try {
        const n = BigInt(val);
        return n >= 0n && n <= 18446744073709551615n; // u64 max
      } catch {
        return false;
      }
    },
    { message: 'Must be valid u64' }
  ),
  nickname: z.string().min(1).max(32),
});

type PlayerFormData = z.infer<typeof playerFormSchema>;

interface PlayerFormProps {
  defaultValues?: Partial<PlayerAccount>;
  onSubmit: (data: PlayerFormData) => Promise<void>;
}

export function PlayerForm({ defaultValues, onSubmit }: PlayerFormProps) {
  const {
    register,
    handleSubmit,
    control,
    formState: { errors, isSubmitting },
  } = useForm<PlayerFormData>({
    resolver: zodResolver(playerFormSchema),
    defaultValues: {
      wallet: defaultValues?.wallet?.toBase58() ?? '',
      level: defaultValues?.level ?? 1,
      experience: defaultValues?.experience?.toString() ?? '0',
      nickname: defaultValues?.nickname ?? '',
    },
  });

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <div className="field">
        <label>Wallet Address</label>
        <input {...register('wallet')} placeholder="PublicKey..." />
        {errors.wallet && <span className="error">{errors.wallet.message}</span>}
      </div>

      <div className="field">
        <label>Level (1-100)</label>
        <input
          type="number"
          {...register('level', { valueAsNumber: true })}
        />
        {errors.level && <span className="error">{errors.level.message}</span>}
      </div>

      <div className="field">
        <label>Experience (u64)</label>
        <input {...register('experience')} placeholder="0" />
        {errors.experience && (
          <span className="error">{errors.experience.message}</span>
        )}
        <span className="hint">Max: 18,446,744,073,709,551,615</span>
      </div>

      <div className="field">
        <label>Nickname</label>
        <input {...register('nickname')} maxLength={32} />
        {errors.nickname && (
          <span className="error">{errors.nickname.message}</span>
        )}
      </div>

      <button type="submit" disabled={isSubmitting}>
        {isSubmitting ? 'Saving...' : 'Save Player'}
      </button>
    </form>
  );
}
```

---

### 4.2 Array Fields (Inventory Management)

```typescript
// components/InventoryForm.tsx
import { useForm, useFieldArray } from 'react-hook-form';
import { z } from 'zod';
import { zodResolver } from '@hookform/resolvers/zod';

const inventorySchema = z.object({
  items: z.array(
    z.object({
      itemId: z.number().min(1).max(65535), // u16
      quantity: z.number().min(1).max(255), // u8
    })
  ).max(20, 'Maximum 20 items allowed'),
});

type InventoryFormData = z.infer<typeof inventorySchema>;

export function InventoryForm({
  defaultItems,
  onSubmit
}: {
  defaultItems?: { itemId: number; quantity: number }[];
  onSubmit: (data: InventoryFormData) => Promise<void>;
}) {
  const { control, register, handleSubmit, formState: { errors } } = useForm<InventoryFormData>({
    resolver: zodResolver(inventorySchema),
    defaultValues: {
      items: defaultItems ?? [{ itemId: 1, quantity: 1 }],
    },
  });

  const { fields, append, remove } = useFieldArray({
    control,
    name: 'items',
  });

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <h3>Inventory ({fields.length}/20 slots)</h3>

      {fields.map((field, index) => (
        <div key={field.id} className="inventory-row">
          <input
            type="number"
            placeholder="Item ID"
            {...register(`items.${index}.itemId`, { valueAsNumber: true })}
          />
          <input
            type="number"
            placeholder="Qty"
            {...register(`items.${index}.quantity`, { valueAsNumber: true })}
          />
          <button type="button" onClick={() => remove(index)}>
            Remove
          </button>
        </div>
      ))}

      {errors.items && (
        <span className="error">{errors.items.message}</span>
      )}

      <button
        type="button"
        onClick={() => append({ itemId: 1, quantity: 1 })}
        disabled={fields.length >= 20}
      >
        Add Item
      </button>

      <button type="submit">Save Inventory</button>
    </form>
  );
}
```

---

### 4.3 Conditional Fields for Enums

```typescript
// components/GameStateForm.tsx
// For LUMOS enum:
// enum GameState {
//   Active,
//   Paused { reason: String },
//   Finished { winner: PublicKey, score: u64 },
// }

import { useForm, useWatch } from 'react-hook-form';

type GameStateKind = 'Active' | 'Paused' | 'Finished';

interface GameStateFormData {
  kind: GameStateKind;
  pauseReason?: string;
  winner?: string;
  score?: string;
}

export function GameStateForm({ onSubmit }: { onSubmit: (data: GameStateFormData) => void }) {
  const { register, control, handleSubmit, formState: { errors } } = useForm<GameStateFormData>({
    defaultValues: { kind: 'Active' },
  });

  const kind = useWatch({ control, name: 'kind' });

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <div className="field">
        <label>Game State</label>
        <select {...register('kind')}>
          <option value="Active">Active</option>
          <option value="Paused">Paused</option>
          <option value="Finished">Finished</option>
        </select>
      </div>

      {/* Conditional: Paused reason */}
      {kind === 'Paused' && (
        <div className="field">
          <label>Pause Reason</label>
          <input
            {...register('pauseReason', { required: 'Reason required' })}
            placeholder="Why is the game paused?"
          />
          {errors.pauseReason && (
            <span className="error">{errors.pauseReason.message}</span>
          )}
        </div>
      )}

      {/* Conditional: Finished fields */}
      {kind === 'Finished' && (
        <>
          <div className="field">
            <label>Winner Address</label>
            <input
              {...register('winner', { required: 'Winner required' })}
              placeholder="PublicKey..."
            />
          </div>
          <div className="field">
            <label>Final Score</label>
            <input
              {...register('score', { required: 'Score required' })}
              placeholder="0"
            />
          </div>
        </>
      )}

      <button type="submit">Update State</button>
    </form>
  );
}
```

---

### 4.4 PublicKey Input Component

```typescript
// components/PublicKeyInput.tsx
import { useState, useCallback } from 'react';
import { PublicKey } from '@solana/web3.js';

interface PublicKeyInputProps {
  value: string;
  onChange: (value: string) => void;
  onValidKey?: (key: PublicKey) => void;
  placeholder?: string;
  required?: boolean;
}

export function PublicKeyInput({
  value,
  onChange,
  onValidKey,
  placeholder = 'Enter Solana address...',
  required = false,
}: PublicKeyInputProps) {
  const [error, setError] = useState<string | null>(null);
  const [isValid, setIsValid] = useState(false);

  const validate = useCallback((input: string) => {
    if (!input && !required) {
      setError(null);
      setIsValid(false);
      return;
    }

    try {
      const key = new PublicKey(input);
      setError(null);
      setIsValid(true);
      onValidKey?.(key);
    } catch {
      setError('Invalid public key format');
      setIsValid(false);
    }
  }, [required, onValidKey]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    onChange(newValue);
    validate(newValue);
  };

  const handlePaste = async () => {
    try {
      const text = await navigator.clipboard.readText();
      onChange(text.trim());
      validate(text.trim());
    } catch {
      // Clipboard access denied
    }
  };

  return (
    <div className="pubkey-input">
      <div className="input-wrapper">
        <input
          type="text"
          value={value}
          onChange={handleChange}
          placeholder={placeholder}
          className={error ? 'error' : isValid ? 'valid' : ''}
        />
        <button type="button" onClick={handlePaste} className="paste-btn">
          Paste
        </button>
        {isValid && <span className="check">✓</span>}
      </div>
      {error && <span className="error-message">{error}</span>}
      {isValid && value && (
        <span className="truncated">
          {value.slice(0, 4)}...{value.slice(-4)}
        </span>
      )}
    </div>
  );
}
```

---

## 5. Display & Formatting Utilities

### 5.1 BigInt/u64 Formatting

```typescript
// utils/format.ts

/**
 * Format large numbers (u64) for display
 * Handles BigInt and number types safely
 */
export function formatU64(
  value: bigint | number | string,
  options?: {
    decimals?: number;
    locale?: string;
    compact?: boolean;
  }
): string {
  const { decimals = 0, locale = 'en-US', compact = false } = options ?? {};

  let num: number;

  if (typeof value === 'bigint') {
    // Check if safe to convert
    if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
      // For very large numbers, format as string with separators
      const str = value.toString();
      if (compact) {
        return formatCompact(str);
      }
      return str.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    }
    num = Number(value);
  } else if (typeof value === 'string') {
    num = parseFloat(value);
  } else {
    num = value;
  }

  if (compact) {
    return new Intl.NumberFormat(locale, {
      notation: 'compact',
      maximumFractionDigits: 1,
    }).format(num);
  }

  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  }).format(num);
}

function formatCompact(numStr: string): string {
  const len = numStr.length;
  if (len > 15) return `${numStr.slice(0, len - 15)}.${numStr.slice(len - 15, len - 14)}Q`;
  if (len > 12) return `${numStr.slice(0, len - 12)}.${numStr.slice(len - 12, len - 11)}T`;
  if (len > 9) return `${numStr.slice(0, len - 9)}.${numStr.slice(len - 9, len - 8)}B`;
  if (len > 6) return `${numStr.slice(0, len - 6)}.${numStr.slice(len - 6, len - 5)}M`;
  if (len > 3) return `${numStr.slice(0, len - 3)}.${numStr.slice(len - 3, len - 2)}K`;
  return numStr;
}

/**
 * Format lamports to SOL
 */
export function formatLamports(
  lamports: bigint | number,
  options?: { decimals?: number; symbol?: boolean }
): string {
  const { decimals = 4, symbol = true } = options ?? {};

  const sol = Number(lamports) / 1e9;
  const formatted = sol.toLocaleString('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });

  return symbol ? `${formatted} SOL` : formatted;
}

/**
 * Parse SOL input to lamports
 */
export function parseToLamports(solAmount: string): bigint {
  const sol = parseFloat(solAmount);
  if (isNaN(sol)) throw new Error('Invalid SOL amount');
  return BigInt(Math.floor(sol * 1e9));
}
```

**Usage:**

```tsx
function TokenBalance({ balance }: { balance: bigint }) {
  return (
    <div className="balance">
      <span className="full">{formatU64(balance)}</span>
      <span className="compact">{formatU64(balance, { compact: true })}</span>
    </div>
  );
}

// Output for 1234567890123n:
// full: "1,234,567,890,123"
// compact: "1.2T"
```

---

### 5.2 PublicKey Display Component

```typescript
// components/AddressDisplay.tsx
import { useState } from 'react';
import { PublicKey } from '@solana/web3.js';

interface AddressDisplayProps {
  address: PublicKey | string;
  truncate?: boolean;
  chars?: number;
  copyable?: boolean;
  explorerLink?: boolean;
  cluster?: 'mainnet-beta' | 'devnet' | 'testnet';
}

export function AddressDisplay({
  address,
  truncate = true,
  chars = 4,
  copyable = true,
  explorerLink = true,
  cluster = 'mainnet-beta',
}: AddressDisplayProps) {
  const [copied, setCopied] = useState(false);

  const addressStr = typeof address === 'string'
    ? address
    : address.toBase58();

  const displayAddress = truncate
    ? `${addressStr.slice(0, chars)}...${addressStr.slice(-chars)}`
    : addressStr;

  const explorerUrl = `https://solscan.io/account/${addressStr}${
    cluster !== 'mainnet-beta' ? `?cluster=${cluster}` : ''
  }`;

  const handleCopy = async () => {
    await navigator.clipboard.writeText(addressStr);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <span className="address-display">
      <code className="address" title={addressStr}>
        {displayAddress}
      </code>

      {copyable && (
        <button
          onClick={handleCopy}
          className="copy-btn"
          title="Copy address"
        >
          {copied ? '✓' : '📋'}
        </button>
      )}

      {explorerLink && (
        <a
          href={explorerUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="explorer-link"
          title="View on Solscan"
        >
          ↗
        </a>
      )}
    </span>
  );
}
```

---

### 5.3 Timestamp/Date Formatting

```typescript
// utils/time.ts

/**
 * Convert Solana timestamp (i64 seconds) to Date
 */
export function timestampToDate(timestamp: bigint | number): Date {
  const seconds = typeof timestamp === 'bigint'
    ? Number(timestamp)
    : timestamp;
  return new Date(seconds * 1000);
}

/**
 * Format timestamp for display
 */
export function formatTimestamp(
  timestamp: bigint | number,
  options?: {
    format?: 'full' | 'date' | 'time' | 'relative';
    locale?: string;
  }
): string {
  const { format = 'full', locale = 'en-US' } = options ?? {};
  const date = timestampToDate(timestamp);

  switch (format) {
    case 'date':
      return date.toLocaleDateString(locale);
    case 'time':
      return date.toLocaleTimeString(locale);
    case 'relative':
      return formatRelative(date);
    case 'full':
    default:
      return date.toLocaleString(locale);
  }
}

function formatRelative(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return 'just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

/**
 * Component for displaying timestamps
 */
export function Timestamp({
  value,
  format = 'relative'
}: {
  value: bigint | number;
  format?: 'full' | 'date' | 'time' | 'relative';
}) {
  const date = timestampToDate(value);
  const display = formatTimestamp(value, { format });

  return (
    <time dateTime={date.toISOString()} title={date.toLocaleString()}>
      {display}
    </time>
  );
}
```

---

### 5.4 Enum Variant Display

```typescript
// utils/enum-display.ts
import type { GameState } from '../generated/schema';

/**
 * Type-safe enum display mapping
 */
const gameStateLabels: Record<GameState['kind'], string> = {
  Active: 'In Progress',
  Paused: 'Paused',
  Finished: 'Completed',
};

const gameStateColors: Record<GameState['kind'], string> = {
  Active: 'green',
  Paused: 'yellow',
  Finished: 'gray',
};

const gameStateIcons: Record<GameState['kind'], string> = {
  Active: '▶️',
  Paused: '⏸️',
  Finished: '✅',
};

export function GameStateBadge({ state }: { state: GameState }) {
  const label = gameStateLabels[state.kind];
  const color = gameStateColors[state.kind];
  const icon = gameStateIcons[state.kind];

  return (
    <span className={`badge badge-${color}`}>
      <span className="icon">{icon}</span>
      <span className="label">{label}</span>

      {/* Show variant-specific data */}
      {state.kind === 'Paused' && (
        <span className="detail">({state.reason})</span>
      )}
      {state.kind === 'Finished' && (
        <span className="detail">
          Winner: <AddressDisplay address={state.winner} chars={4} />
        </span>
      )}
    </span>
  );
}

/**
 * Generic enum display helper
 */
export function getEnumLabel<T extends { kind: string }>(
  value: T,
  labels: Record<T['kind'], string>
): string {
  return labels[value.kind] ?? value.kind;
}
```

---

## 6. Framework-Specific Patterns

### 6.1 Next.js App Router (RSC Boundaries)

```typescript
// app/player/[address]/page.tsx
import { Suspense } from 'react';
import { PublicKey } from '@solana/web3.js';
import { PlayerDetails } from './PlayerDetails';
import { PlayerSkeleton } from './PlayerSkeleton';

// Server Component - can fetch initial data
export default async function PlayerPage({
  params,
}: {
  params: { address: string };
}) {
  // Validate address on server
  let publicKey: PublicKey;
  try {
    publicKey = new PublicKey(params.address);
  } catch {
    return <div>Invalid address</div>;
  }

  return (
    <div>
      <h1>Player Profile</h1>
      <Suspense fallback={<PlayerSkeleton />}>
        {/* Client Component for wallet interaction */}
        <PlayerDetails address={params.address} />
      </Suspense>
    </div>
  );
}
```

```typescript
// app/player/[address]/PlayerDetails.tsx
'use client';

import { useEffect, useState } from 'react';
import { PublicKey } from '@solana/web3.js';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import type { PlayerAccount } from '@/generated/schema';
import { PlayerAccountSchema } from '@/generated/schema';

export function PlayerDetails({ address }: { address: string }) {
  const { connection } = useConnection();
  const { publicKey: walletKey } = useWallet();
  const [player, setPlayer] = useState<PlayerAccount | null>(null);

  const playerAddress = new PublicKey(address);
  const isOwner = walletKey?.equals(playerAddress);

  useEffect(() => {
    const fetchPlayer = async () => {
      const info = await connection.getAccountInfo(playerAddress);
      if (info) {
        const data = info.data.slice(8);
        setPlayer(PlayerAccountSchema.decode(data));
      }
    };
    fetchPlayer();
  }, [connection, address]);

  if (!player) return <div>Loading...</div>;

  return (
    <div>
      <p>Level: {player.level}</p>
      <p>XP: {player.experience.toString()}</p>
      {isOwner && <button>Edit Profile</button>}
    </div>
  );
}
```

---

### 6.2 SvelteKit Load Functions

```typescript
// src/routes/player/[address]/+page.ts
import type { PageLoad } from './$types';
import { PublicKey } from '@solana/web3.js';
import { error } from '@sveltejs/kit';

export const load: PageLoad = ({ params }) => {
  // Validate address
  try {
    new PublicKey(params.address);
  } catch {
    throw error(400, 'Invalid address');
  }

  return {
    address: params.address,
  };
};
```

```svelte
<!-- src/routes/player/[address]/+page.svelte -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { PublicKey } from '@solana/web3.js';
  import { connection } from '$lib/connection';
  import { PlayerAccountSchema } from '$lib/generated/schema';
  import type { PlayerAccount } from '$lib/generated/schema';
  import type { PageData } from './$types';

  export let data: PageData;

  let player: PlayerAccount | null = null;
  let subscriptionId: number | null = null;

  onMount(async () => {
    const address = new PublicKey(data.address);

    // Initial fetch
    const info = await connection.getAccountInfo(address);
    if (info) {
      player = PlayerAccountSchema.decode(info.data.slice(8));
    }

    // Subscribe
    subscriptionId = connection.onAccountChange(address, (info) => {
      player = PlayerAccountSchema.decode(info.data.slice(8));
    });
  });

  onDestroy(() => {
    if (subscriptionId !== null) {
      connection.removeAccountChangeListener(subscriptionId);
    }
  });
</script>

<h1>Player Profile</h1>

{#if player}
  <div class="player-card">
    <p>Level: {player.level}</p>
    <p>XP: {player.experience.toString()}</p>
  </div>
{:else}
  <p>Loading...</p>
{/if}
```

---

### 6.3 Remix Loaders/Actions

```typescript
// app/routes/player.$address.tsx
import { json, type LoaderFunctionArgs, type ActionFunctionArgs } from '@remix-run/node';
import { useLoaderData, useFetcher } from '@remix-run/react';
import { PublicKey } from '@solana/web3.js';

// Loader runs on server
export async function loader({ params }: LoaderFunctionArgs) {
  const { address } = params;

  // Validate
  try {
    new PublicKey(address!);
  } catch {
    throw new Response('Invalid address', { status: 400 });
  }

  return json({ address });
}

// Action handles form submissions
export async function action({ request }: ActionFunctionArgs) {
  const formData = await request.formData();
  const intent = formData.get('intent');

  // Handle different actions
  if (intent === 'refresh') {
    // Trigger refresh logic
    return json({ refreshed: true });
  }

  return json({ error: 'Unknown action' }, { status: 400 });
}

// Component
export default function PlayerRoute() {
  const { address } = useLoaderData<typeof loader>();
  const fetcher = useFetcher();

  return (
    <div>
      <h1>Player: {address}</h1>

      {/* Client-side Solana interaction component */}
      <ClientOnly>
        <PlayerDetails address={address} />
      </ClientOnly>

      {/* Server action */}
      <fetcher.Form method="post">
        <input type="hidden" name="intent" value="refresh" />
        <button type="submit">Refresh</button>
      </fetcher.Form>
    </div>
  );
}
```

---

### 6.4 Nuxt 3 Composables

```typescript
// composables/usePlayer.ts
import { ref, watch, onUnmounted } from 'vue';
import { Connection, PublicKey } from '@solana/web3.js';
import type { PlayerAccount } from '~/generated/schema';
import { PlayerAccountSchema } from '~/generated/schema';

export function usePlayer(address: Ref<string | null>) {
  const player = ref<PlayerAccount | null>(null);
  const isLoading = ref(false);
  const error = ref<Error | null>(null);

  const connection = useConnection(); // Another composable
  let subscriptionId: number | null = null;

  const fetchPlayer = async (addr: PublicKey) => {
    isLoading.value = true;
    error.value = null;

    try {
      const info = await connection.getAccountInfo(addr);
      if (info) {
        player.value = PlayerAccountSchema.decode(info.data.slice(8));
      }
    } catch (e) {
      error.value = e as Error;
    } finally {
      isLoading.value = false;
    }
  };

  const subscribe = (addr: PublicKey) => {
    subscriptionId = connection.onAccountChange(addr, (info) => {
      try {
        player.value = PlayerAccountSchema.decode(info.data.slice(8));
      } catch (e) {
        error.value = e as Error;
      }
    });
  };

  const unsubscribe = () => {
    if (subscriptionId !== null) {
      connection.removeAccountChangeListener(subscriptionId);
      subscriptionId = null;
    }
  };

  // Watch address changes
  watch(address, async (newAddr, oldAddr) => {
    unsubscribe();

    if (newAddr) {
      const pubkey = new PublicKey(newAddr);
      await fetchPlayer(pubkey);
      subscribe(pubkey);
    } else {
      player.value = null;
    }
  }, { immediate: true });

  onUnmounted(unsubscribe);

  return {
    player: readonly(player),
    isLoading: readonly(isLoading),
    error: readonly(error),
    refresh: () => address.value && fetchPlayer(new PublicKey(address.value)),
  };
}
```

**Nuxt Page:**

```vue
<!-- pages/player/[address].vue -->
<script setup lang="ts">
const route = useRoute();
const address = computed(() => route.params.address as string);

const { player, isLoading, error } = usePlayer(address);
</script>

<template>
  <div>
    <h1>Player Profile</h1>

    <div v-if="isLoading">Loading...</div>
    <div v-else-if="error">{{ error.message }}</div>
    <div v-else-if="player">
      <p>Level: {{ player.level }}</p>
      <p>XP: {{ player.experience.toString() }}</p>
    </div>
  </div>
</template>
```

---

## 7. Testing Client Code

### 7.1 Mocking Account Deserialization

```typescript
// __tests__/player.test.ts
import { describe, it, expect, vi } from 'vitest';
import { PublicKey } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';
import type { PlayerAccount } from '../generated/schema';
import { PlayerAccountSchema } from '../generated/schema';

// Mock player data
const mockPlayer: PlayerAccount = {
  wallet: new PublicKey('11111111111111111111111111111111'),
  level: 10,
  experience: BigInt(5000),
  inventory: [1, 2, 3],
  guild: undefined,
};

// Create mock buffer
function createMockAccountBuffer(player: PlayerAccount): Buffer {
  // 8-byte discriminator + serialized data
  const discriminator = Buffer.alloc(8);
  const data = Buffer.alloc(1024);
  PlayerAccountSchema.encode(player, data);
  return Buffer.concat([discriminator, data]);
}

describe('PlayerAccount deserialization', () => {
  it('should decode player from buffer', () => {
    const buffer = createMockAccountBuffer(mockPlayer);
    const data = buffer.slice(8);
    const decoded = PlayerAccountSchema.decode(data);

    expect(decoded.level).toBe(10);
    expect(decoded.experience).toBe(BigInt(5000));
    expect(decoded.inventory).toEqual([1, 2, 3]);
  });

  it('should handle missing optional fields', () => {
    const playerWithoutGuild = { ...mockPlayer, guild: undefined };
    const buffer = createMockAccountBuffer(playerWithoutGuild);
    const data = buffer.slice(8);
    const decoded = PlayerAccountSchema.decode(data);

    expect(decoded.guild).toBeUndefined();
  });
});
```

---

### 7.2 Testing Custom Hooks

```typescript
// __tests__/usePlayerQuery.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { usePlayerQuery } from '../hooks/usePlayerQuery';
import { PublicKey, Connection } from '@solana/web3.js';

// Mock connection
vi.mock('@solana/wallet-adapter-react', () => ({
  useConnection: () => ({
    connection: {
      getAccountInfo: vi.fn().mockResolvedValue({
        data: Buffer.alloc(100), // Mock buffer
      }),
    },
  }),
}));

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
};

describe('usePlayerQuery', () => {
  it('should fetch player data', async () => {
    const address = new PublicKey('11111111111111111111111111111111');

    const { result } = renderHook(
      () => usePlayerQuery(address),
      { wrapper: createWrapper() }
    );

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });
  });

  it('should not fetch when address is null', () => {
    const { result } = renderHook(
      () => usePlayerQuery(null),
      { wrapper: createWrapper() }
    );

    expect(result.current.isLoading).toBe(false);
    expect(result.current.data).toBeUndefined();
  });
});
```

---

### 7.3 E2E Testing with Localnet

```typescript
// e2e/player.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Player Profile', () => {
  test.beforeEach(async ({ page }) => {
    // Start local validator (assumes running)
    // Connect to localnet
    await page.goto('http://localhost:3000?cluster=localnet');
  });

  test('should display player level', async ({ page }) => {
    // Navigate to a known test player
    await page.goto('/player/TestPlayer111111111111111111111111');

    // Wait for data to load
    await expect(page.getByText('Level')).toBeVisible();

    // Check level is displayed
    const levelText = await page.getByTestId('player-level').textContent();
    expect(levelText).toMatch(/Level \d+/);
  });

  test('should update on level up', async ({ page }) => {
    await page.goto('/player/TestPlayer111111111111111111111111');

    // Get initial level
    const initialLevel = await page.getByTestId('player-level').textContent();

    // Click level up (assumes wallet connected)
    await page.getByRole('button', { name: 'Level Up' }).click();

    // Wait for confirmation
    await expect(page.getByText('Transaction confirmed')).toBeVisible();

    // Check level increased
    const newLevel = await page.getByTestId('player-level').textContent();
    expect(parseInt(newLevel!)).toBeGreaterThan(parseInt(initialLevel!));
  });
});
```

---

## 8. Performance Optimization

### 8.1 Memoization Strategies

```typescript
// hooks/useMemoizedPlayer.ts
import { useMemo } from 'react';
import type { PlayerAccount } from '../generated/schema';

/**
 * Memoize derived data from player account
 */
export function usePlayerStats(player: PlayerAccount | null) {
  // Expensive calculations memoized
  const stats = useMemo(() => {
    if (!player) return null;

    return {
      levelProgress: calculateLevelProgress(player.experience, player.level),
      inventoryValue: calculateInventoryValue(player.inventory),
      rank: calculateRank(player.level, player.experience),
    };
  }, [player?.level, player?.experience, player?.inventory]);

  return stats;
}

function calculateLevelProgress(xp: bigint, level: number): number {
  const xpForLevel = BigInt(level * 1000);
  const xpForNextLevel = BigInt((level + 1) * 1000);
  const progress = Number(xp - xpForLevel) / Number(xpForNextLevel - xpForLevel);
  return Math.min(Math.max(progress, 0), 1);
}

function calculateInventoryValue(inventory: number[]): number {
  // Simulate expensive calculation
  return inventory.reduce((sum, itemId) => sum + getItemValue(itemId), 0);
}

function calculateRank(level: number, xp: bigint): string {
  if (level >= 50) return 'Master';
  if (level >= 30) return 'Expert';
  if (level >= 15) return 'Intermediate';
  return 'Beginner';
}
```

---

### 8.2 Request Deduplication

```typescript
// utils/deduplicatedFetch.ts
const pendingRequests = new Map<string, Promise<any>>();

export async function deduplicatedAccountFetch<T>(
  connection: Connection,
  address: PublicKey,
  schema: borsh.Layout<T>,
  discriminatorSize = 8
): Promise<T | null> {
  const key = address.toBase58();

  // Return pending request if exists
  if (pendingRequests.has(key)) {
    return pendingRequests.get(key);
  }

  // Create new request
  const request = (async () => {
    try {
      const info = await connection.getAccountInfo(address);
      if (!info) return null;

      const data = info.data.slice(discriminatorSize);
      return schema.decode(data);
    } finally {
      // Clean up after completion
      pendingRequests.delete(key);
    }
  })();

  pendingRequests.set(key, request);
  return request;
}
```

---

### 8.3 RPC Rate Limiting

```typescript
// utils/rateLimiter.ts
export class RateLimiter {
  private queue: Array<() => Promise<any>> = [];
  private processing = false;
  private lastRequestTime = 0;

  constructor(
    private requestsPerSecond: number = 10,
    private burstLimit: number = 5
  ) {}

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      this.queue.push(async () => {
        try {
          const result = await fn();
          resolve(result);
        } catch (error) {
          reject(error);
        }
      });

      this.processQueue();
    });
  }

  private async processQueue() {
    if (this.processing || this.queue.length === 0) return;

    this.processing = true;

    while (this.queue.length > 0) {
      const now = Date.now();
      const timeSinceLastRequest = now - this.lastRequestTime;
      const minDelay = 1000 / this.requestsPerSecond;

      if (timeSinceLastRequest < minDelay) {
        await new Promise(r => setTimeout(r, minDelay - timeSinceLastRequest));
      }

      const fn = this.queue.shift();
      if (fn) {
        this.lastRequestTime = Date.now();
        await fn();
      }
    }

    this.processing = false;
  }
}

// Usage
const rateLimiter = new RateLimiter(10); // 10 requests per second

export async function rateLimitedFetch(connection: Connection, address: PublicKey) {
  return rateLimiter.execute(() => connection.getAccountInfo(address));
}
```

---

### 8.4 Prefetching Patterns

```typescript
// hooks/usePrefetch.ts
import { useQueryClient } from '@tanstack/react-query';
import { useConnection } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { playerKeys } from './usePlayerQuery';
import { PlayerAccountSchema } from '../generated/schema';

export function usePrefetchPlayer() {
  const queryClient = useQueryClient();
  const { connection } = useConnection();

  const prefetch = async (address: PublicKey) => {
    await queryClient.prefetchQuery({
      queryKey: playerKeys.detail(address.toBase58()),
      queryFn: async () => {
        const info = await connection.getAccountInfo(address);
        if (!info) throw new Error('Not found');
        return PlayerAccountSchema.decode(info.data.slice(8));
      },
      staleTime: 30_000,
    });
  };

  return { prefetch };
}

// Usage: Prefetch on hover
function PlayerListItem({ address }: { address: PublicKey }) {
  const { prefetch } = usePrefetchPlayer();

  return (
    <Link
      href={`/player/${address.toBase58()}`}
      onMouseEnter={() => prefetch(address)}
      onFocus={() => prefetch(address)}
    >
      {address.toBase58().slice(0, 8)}...
    </Link>
  );
}
```

---

## 9. Complete Example: Token Dashboard

A full working example combining all patterns.

### Schema (token-dashboard.lumos)

```rust
#[solana]
#[account]
struct TokenAccount {
    owner: PublicKey,
    mint: PublicKey,
    balance: u64,
    delegated_amount: u64,
    delegate: Option<PublicKey>,
    is_frozen: bool,
    last_activity: i64,
}

#[solana]
enum TransferStatus {
    Pending,
    Completed { timestamp: i64 },
    Failed { error_code: u32 },
}

#[solana]
#[account]
struct TransferRecord {
    from: PublicKey,
    to: PublicKey,
    amount: u64,
    status: TransferStatus,
    created_at: i64,
}
```

### Zustand Store (stores/tokenStore.ts)

```typescript
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { Connection, PublicKey } from '@solana/web3.js';
import type { TokenAccount, TransferRecord } from '../generated/schema';
import { TokenAccountSchema, TransferRecordSchema } from '../generated/schema';

interface TokenState {
  // Data
  account: TokenAccount | null;
  transfers: TransferRecord[];
  isLoading: boolean;
  error: Error | null;

  // Actions
  fetchAccount: (connection: Connection, address: PublicKey) => Promise<void>;
  fetchTransfers: (connection: Connection, addresses: PublicKey[]) => Promise<void>;
  subscribe: (connection: Connection, address: PublicKey) => () => void;
  reset: () => void;
}

export const useTokenStore = create<TokenState>()(
  subscribeWithSelector((set, get) => ({
    account: null,
    transfers: [],
    isLoading: false,
    error: null,

    fetchAccount: async (connection, address) => {
      set({ isLoading: true, error: null });
      try {
        const info = await connection.getAccountInfo(address);
        if (!info) throw new Error('Account not found');
        const account = TokenAccountSchema.decode(info.data.slice(8));
        set({ account, isLoading: false });
      } catch (error) {
        set({ error: error as Error, isLoading: false });
      }
    },

    fetchTransfers: async (connection, addresses) => {
      try {
        const infos = await connection.getMultipleAccountsInfo(addresses);
        const transfers = infos
          .filter((info): info is NonNullable<typeof info> => info !== null)
          .map((info) => TransferRecordSchema.decode(info.data.slice(8)));
        set({ transfers });
      } catch (error) {
        set({ error: error as Error });
      }
    },

    subscribe: (connection, address) => {
      const id = connection.onAccountChange(address, (info) => {
        try {
          const account = TokenAccountSchema.decode(info.data.slice(8));
          set({ account });
        } catch (error) {
          set({ error: error as Error });
        }
      });
      return () => connection.removeAccountChangeListener(id);
    },

    reset: () => set({ account: null, transfers: [], isLoading: false, error: null }),
  }))
);
```

### Dashboard Component (components/TokenDashboard.tsx)

```tsx
import { useEffect, useMemo } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey } from '@solana/web3.js';
import { useTokenStore } from '../stores/tokenStore';
import { formatU64, formatLamports, formatTimestamp } from '../utils/format';
import { AddressDisplay } from './AddressDisplay';
import { TransferForm } from './TransferForm';
import { TransactionHistory } from './TransactionHistory';

export function TokenDashboard({ tokenAddress }: { tokenAddress: string }) {
  const { connection } = useConnection();
  const { publicKey: wallet } = useWallet();
  const { account, transfers, isLoading, error, fetchAccount, subscribe, reset } = useTokenStore();

  const address = useMemo(() => new PublicKey(tokenAddress), [tokenAddress]);
  const isOwner = wallet && account?.owner.equals(wallet);

  // Fetch and subscribe on mount
  useEffect(() => {
    fetchAccount(connection, address);
    const unsubscribe = subscribe(connection, address);
    return () => {
      unsubscribe();
      reset();
    };
  }, [connection, address]);

  if (isLoading) {
    return <DashboardSkeleton />;
  }

  if (error) {
    return (
      <div className="error-state">
        <p>Error: {error.message}</p>
        <button onClick={() => fetchAccount(connection, address)}>Retry</button>
      </div>
    );
  }

  if (!account) {
    return <div>Account not found</div>;
  }

  return (
    <div className="token-dashboard">
      {/* Header */}
      <header className="dashboard-header">
        <h1>Token Dashboard</h1>
        <AddressDisplay address={address} copyable explorerLink />
      </header>

      {/* Balance Card */}
      <div className="balance-card">
        <div className="balance-main">
          <span className="label">Balance</span>
          <span className="value">{formatU64(account.balance)}</span>
        </div>
        {account.delegated_amount > 0n && (
          <div className="balance-delegated">
            <span className="label">Delegated</span>
            <span className="value">{formatU64(account.delegated_amount)}</span>
          </div>
        )}
      </div>

      {/* Account Details */}
      <div className="details-grid">
        <div className="detail-item">
          <span className="label">Owner</span>
          <AddressDisplay address={account.owner} />
        </div>
        <div className="detail-item">
          <span className="label">Mint</span>
          <AddressDisplay address={account.mint} />
        </div>
        <div className="detail-item">
          <span className="label">Status</span>
          <span className={`status ${account.is_frozen ? 'frozen' : 'active'}`}>
            {account.is_frozen ? '🔒 Frozen' : '✓ Active'}
          </span>
        </div>
        <div className="detail-item">
          <span className="label">Last Activity</span>
          <span>{formatTimestamp(account.last_activity, { format: 'relative' })}</span>
        </div>
        {account.delegate && (
          <div className="detail-item">
            <span className="label">Delegate</span>
            <AddressDisplay address={account.delegate} />
          </div>
        )}
      </div>

      {/* Transfer Form (only for owner) */}
      {isOwner && !account.is_frozen && (
        <section className="transfer-section">
          <h2>Send Tokens</h2>
          <TransferForm
            fromAccount={address}
            maxAmount={account.balance}
            onSuccess={() => fetchAccount(connection, address)}
          />
        </section>
      )}

      {/* Recent Transfers */}
      <section className="history-section">
        <h2>Recent Transfers</h2>
        <TransferList transfers={transfers} />
      </section>
    </div>
  );
}

function TransferList({ transfers }: { transfers: TransferRecord[] }) {
  if (transfers.length === 0) {
    return <p className="empty">No transfers yet</p>;
  }

  return (
    <ul className="transfer-list">
      {transfers.map((transfer, i) => (
        <li key={i} className="transfer-item">
          <div className="transfer-addresses">
            <AddressDisplay address={transfer.from} chars={4} />
            <span className="arrow">→</span>
            <AddressDisplay address={transfer.to} chars={4} />
          </div>
          <div className="transfer-amount">{formatU64(transfer.amount)}</div>
          <div className="transfer-status">
            <TransferStatusBadge status={transfer.status} />
          </div>
          <div className="transfer-time">
            {formatTimestamp(transfer.created_at, { format: 'relative' })}
          </div>
        </li>
      ))}
    </ul>
  );
}

function TransferStatusBadge({ status }: { status: TransferStatus }) {
  switch (status.kind) {
    case 'Pending':
      return <span className="badge pending">⏳ Pending</span>;
    case 'Completed':
      return <span className="badge completed">✓ Completed</span>;
    case 'Failed':
      return <span className="badge failed">✗ Failed ({status.error_code})</span>;
  }
}

function DashboardSkeleton() {
  return (
    <div className="token-dashboard skeleton">
      <div className="skeleton-header" />
      <div className="skeleton-balance" />
      <div className="skeleton-details" />
    </div>
  );
}
```

### Transfer Form Component (components/TransferForm.tsx)

```tsx
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { PublicKey } from '@solana/web3.js';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKeyInput } from './PublicKeyInput';
import { useTransactionWizard } from '../hooks/useTransactionWizard';

const transferSchema = z.object({
  recipient: z.string().refine((val) => {
    try {
      new PublicKey(val);
      return true;
    } catch {
      return false;
    }
  }, 'Invalid address'),
  amount: z.string().refine((val) => {
    const n = BigInt(val);
    return n > 0n;
  }, 'Amount must be greater than 0'),
});

type TransferFormData = z.infer<typeof transferSchema>;

interface TransferFormProps {
  fromAccount: PublicKey;
  maxAmount: bigint;
  onSuccess: () => void;
}

export function TransferForm({ fromAccount, maxAmount, onSuccess }: TransferFormProps) {
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const wizard = useTransactionWizard();

  const {
    register,
    handleSubmit,
    setValue,
    formState: { errors },
  } = useForm<TransferFormData>({
    resolver: zodResolver(transferSchema),
  });

  const onSubmit = async (data: TransferFormData) => {
    if (!publicKey) return;

    try {
      // Build transaction
      const tx = await wizard.build(async () => {
        const recipient = new PublicKey(data.recipient);
        const amount = BigInt(data.amount);

        // Build your transfer instruction here
        return buildTransferTransaction(
          fromAccount,
          recipient,
          amount,
          publicKey
        );
      });

      // Simulate
      await wizard.simulate(connection, tx);

      // Sign and send
      const signed = await wizard.sign(
        (tx) => sendTransaction(tx, connection),
        tx
      );

      // Confirm
      await wizard.confirm(connection, wizard.signature!);

      onSuccess();
    } catch (e) {
      // Error captured in wizard state
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="transfer-form">
      <div className="field">
        <label>Recipient</label>
        <PublicKeyInput
          value=""
          onChange={(val) => setValue('recipient', val)}
          placeholder="Enter recipient address..."
        />
        {errors.recipient && (
          <span className="error">{errors.recipient.message}</span>
        )}
      </div>

      <div className="field">
        <label>Amount</label>
        <div className="amount-input">
          <input
            {...register('amount')}
            placeholder="0"
          />
          <button
            type="button"
            onClick={() => setValue('amount', maxAmount.toString())}
            className="max-btn"
          >
            MAX
          </button>
        </div>
        {errors.amount && (
          <span className="error">{errors.amount.message}</span>
        )}
        <span className="hint">Available: {maxAmount.toString()}</span>
      </div>

      {/* Transaction Status */}
      {wizard.step !== 'idle' && (
        <div className="tx-status">
          {wizard.step === 'building' && <p>Building transaction...</p>}
          {wizard.step === 'simulating' && <p>Simulating...</p>}
          {wizard.step === 'signing' && <p>Please sign in your wallet...</p>}
          {wizard.step === 'sending' && <p>Sending...</p>}
          {wizard.step === 'confirming' && <p>Confirming...</p>}
          {wizard.step === 'success' && <p className="success">✓ Transfer complete!</p>}
          {wizard.step === 'error' && (
            <p className="error">Error: {wizard.error?.message}</p>
          )}
        </div>
      )}

      <button
        type="submit"
        disabled={wizard.step !== 'idle' && wizard.step !== 'error'}
        className="submit-btn"
      >
        {wizard.step === 'idle' ? 'Send Tokens' : 'Processing...'}
      </button>
    </form>
  );
}
```

---

## Related Guides

- [web3.js Integration](./web3js-integration.md) - Basic patterns
- [Usage Examples](./usage-examples.md) - Generated code patterns
- [Error Handling & Validation](./error-handling-validation.md) - Error patterns
- [Anchor Integration](./anchor-integration.md) - Program interaction

---

## Summary

This guide covered advanced client-side patterns for LUMOS-generated types:

| Pattern | Use Case |
|---------|----------|
| **State Management** | Zustand, TanStack Query, Redux, Pinia, Svelte stores |
| **Real-Time** | Subscriptions, polling, optimistic updates |
| **Transaction UI** | Wizards, simulation, state machines |
| **Forms** | React Hook Form, validation, arrays, enums |
| **Display** | BigInt formatting, PublicKey display, timestamps |
| **Frameworks** | Next.js, SvelteKit, Remix, Nuxt patterns |
| **Testing** | Mocking, hook testing, E2E |
| **Performance** | Memoization, deduplication, rate limiting |

Use these patterns to build production-ready Solana frontends with type-safe LUMOS schemas.
