# @getlumos/cli

> LUMOS schema language CLI for JavaScript/TypeScript - Generate type-safe Rust and TypeScript code for Solana

[![npm version](https://img.shields.io/npm/v/@getlumos/cli.svg)](https://www.npmjs.com/package/@getlumos/cli)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/getlumos/lumos)

Write data structures once in `.lumos` syntax → Generate production-ready Rust + TypeScript with guaranteed Borsh serialization compatibility.

## Features

- ✅ **No Rust toolchain required** - Powered by WebAssembly
- ✅ **Type-safe code generation** - Rust + TypeScript from one schema
- ✅ **Borsh serialization** - Guaranteed compatibility between languages
- ✅ **Anchor integration** - First-class support for Solana Anchor programs
- ✅ **Fast installation** - Pre-compiled WASM binary (~750 KB)
- ✅ **CLI + Programmatic API** - Use in scripts or build tools

## Installation

```bash
npm install @getlumos/cli
# or
yarn add @getlumos/cli
# or
pnpm add @getlumos/cli
```

## Quick Start

### 1. Create a schema

```lumos
// schema.lumos
#[solana]
#[account]
struct PlayerAccount {
    wallet: PublicKey,
    username: String,
    level: u16,
    experience: u64,
}
```

### 2. Generate code

**CLI:**
```bash
npx lumos generate schema.lumos \
  --output-rust src/generated.rs \
  --output-typescript src/generated.ts
```

**Programmatic:**
```typescript
import { generate } from '@getlumos/cli';

await generate('schema.lumos', {
  outputRust: 'src/generated.rs',
  outputTypeScript: 'src/generated.ts',
});
```

### 3. Use generated code

**Rust (Anchor program):**
```rust
use generated::PlayerAccount;

#[program]
pub mod my_program {
    pub fn create_player(ctx: Context<CreatePlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.wallet = ctx.accounts.authority.key();
        player.level = 1;
        Ok(())
    }
}
```

**TypeScript (Frontend):**
```typescript
import { PlayerAccount, PlayerAccountSchema } from './generated';

const player = borsh.deserialize(
  PlayerAccountSchema,
  accountData
);
console.log(player.username, player.level);
```

## CLI Commands

### `generate`

Generate Rust and TypeScript code from schema.

```bash
lumos generate <schema> [options]

Options:
  --output-rust <path>       Output path for Rust code
  --output-typescript <path> Output path for TypeScript code
```

**Examples:**

```bash
# Generate both Rust and TypeScript
npx lumos generate schema.lumos \
  --output-rust programs/src/state.rs \
  --output-typescript app/src/types.ts

# Generate only Rust
npx lumos generate schema.lumos --output-rust src/state.rs

# Generate only TypeScript
npx lumos generate schema.lumos --output-typescript src/types.ts
```

### `validate`

Validate schema syntax without generating code.

```bash
lumos validate <schema>
```

**Example:**
```bash
npx lumos validate schema.lumos
# ✅ Schema is valid
```

## package.json Integration

Add to your `package.json` scripts:

```json
{
  "scripts": {
    "generate": "lumos generate schema.lumos --output-rust programs/src/state.rs --output-typescript app/src/types.ts",
    "build": "npm run generate && anchor build",
    "dev": "npm run generate && npm run start"
  }
}
```

Then:
```bash
npm run generate  # Generate code
npm run build     # Generate + build Anchor program
```

## Programmatic API

### `generate(schemaPath, options)`

Generate code from a schema file.

**Parameters:**
- `schemaPath` (string): Path to `.lumos` schema file
- `options` (object):
  - `outputRust` (string, optional): Output path for Rust code
  - `outputTypeScript` (string, optional): Output path for TypeScript code

**Returns:** `Promise<GeneratedCode>`
- `rust` (string): Generated Rust code
- `typescript` (string): Generated TypeScript code

**Example:**
```typescript
import { generate } from '@getlumos/cli';

const result = await generate('schema.lumos', {
  outputRust: 'src/generated.rs',
  outputTypeScript: 'src/generated.ts',
});

console.log('Rust code length:', result.rust.length);
console.log('TypeScript code length:', result.typescript.length);
```

### `validate(schemaPath)`

Validate a schema file.

**Parameters:**
- `schemaPath` (string): Path to `.lumos` schema file

**Returns:** `Promise<ValidationResult>`
- `valid` (boolean): Whether schema is valid
- `error` (string, optional): Error message if invalid

**Example:**
```typescript
import { validate } from '@getlumos/cli';

const result = await validate('schema.lumos');
if (!result.valid) {
  console.error('Validation failed:', result.error);
  process.exit(1);
}
```

## Build Tool Integration

### Vite Plugin (Example)

```javascript
// vite.config.js
import { generate } from '@getlumos/cli';

export default {
  plugins: [
    {
      name: 'lumos',
      async buildStart() {
        await generate('schema.lumos', {
          outputTypeScript: 'src/generated.ts',
        });
      },
    },
  ],
};
```

### Webpack Plugin (Example)

```javascript
// webpack.config.js
const { generate } = require('@getlumos/cli');

class LumosPlugin {
  apply(compiler) {
    compiler.hooks.beforeCompile.tapPromise('LumosPlugin', async () => {
      await generate('schema.lumos', {
        outputTypeScript: 'src/generated.ts',
      });
    });
  }
}

module.exports = {
  plugins: [new LumosPlugin()],
};
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'

      - run: npm install
      - run: npm run generate  # Uses @getlumos/cli
      - run: npm run build
```

### Vercel / Netlify

Add to build command:
```json
{
  "scripts": {
    "build": "lumos generate schema.lumos --output-typescript src/generated.ts && vite build"
  }
}
```

## Comparison with Rust CLI

| Feature | @getlumos/cli (npm) | lumos-cli (cargo) |
|---------|---------------------|-------------------|
| **Installation** | `npm install` (~5s) | `cargo install` (~5 min) |
| **Dependencies** | Node.js only | Rust toolchain (1-2 GB) |
| **Package size** | ~750 KB WASM | N/A (compiles from source) |
| **Speed** | ~10-20% slower (WASM) | Native (100%) |
| **Use case** | JS/TS projects | Rust projects |
| **Browser support** | ✅ Yes (future) | ❌ No |

**When to use npm package:**
- ✅ JavaScript/TypeScript projects
- ✅ Frontend dApps
- ✅ CI/CD without Rust toolchain
- ✅ Fast installation required

**When to use Rust crate:**
- ✅ Rust-first projects
- ✅ Maximum performance needed
- ✅ Already have Rust toolchain

## Examples

See [awesome-lumos](https://github.com/getlumos/awesome-lumos) for full-stack examples:
- NFT Marketplace
- DeFi Staking
- DAO Governance
- Gaming Inventory
- Token Vesting

## Troubleshooting

### "Cannot find module 'lumos_core.js'"

**Solution:** Run `npm run build` to compile WASM bindings.

### "Permission denied" when running CLI

**Solution:** Make CLI executable:
```bash
chmod +x node_modules/@getlumos/cli/dist/cli.js
```

### WASM module initialization fails

**Solution:** Ensure Node.js >= 16.0.0:
```bash
node --version  # Should be >= v16.0.0
```

## Development

```bash
# Install dependencies
npm install

# Build WASM + TypeScript
npm run build

# Run tests
npm test

# Watch mode
npm run test:watch
```

## Related Packages

- [`lumos-core`](https://crates.io/crates/lumos-core) - Rust core library
- [`lumos-cli`](https://crates.io/crates/lumos-cli) - Rust CLI binary
- [`lumos-lsp`](https://crates.io/crates/lumos-lsp) - Language Server Protocol
- [VS Code Extension](https://marketplace.visualstudio.com/items?itemName=getlumos.vscode-lumos)

## License

MIT OR Apache-2.0

## Links

- **Website:** https://lumos-lang.org
- **Documentation:** https://docs.lumos-lang.org
- **GitHub:** https://github.com/getlumos/lumos
- **Issues:** https://github.com/getlumos/lumos/issues
