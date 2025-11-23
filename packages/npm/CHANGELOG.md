# Changelog

All notable changes to `@getlumos/cli` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-11-23

### Added
- Initial release of `@getlumos/cli` npm package
- WebAssembly-based code generation (no Rust toolchain required)
- CLI commands: `generate` and `validate`
- Programmatic API for Node.js/TypeScript projects
- TypeScript definitions and source maps
- Comprehensive README with examples
- Integration examples for Vite, Webpack, CI/CD
- ~750 KB optimized WASM binary

### Features
- Generate Rust code with Anchor integration
- Generate TypeScript code with Borsh schemas
- Schema validation without code generation
- Support for Solana types (PublicKey, Signature)
- Support for complex types (Vec, Option)
- Enum support (unit, tuple, struct variants)

### Documentation
- CLI usage examples
- Programmatic API reference
- Build tool integration guide
- CI/CD integration examples
- Troubleshooting guide

[unreleased]: https://github.com/getlumos/lumos/compare/@getlumos/cli-v0.1.0...HEAD
[0.1.0]: https://github.com/getlumos/lumos/releases/tag/@getlumos/cli-v0.1.0
