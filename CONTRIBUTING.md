# Contributing to LUMOS

Thank you for your interest in contributing to LUMOS! This document provides guidelines and instructions for contributing.

## 🚀 Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- A GitHub account

### Development Setup

1. **Fork and clone the repository**
   ```bash

### Environment Variables

Create a `.env` file based on `.env.example` at the repo root. Useful variables:

- `LUMOS_WATCH_DEBOUNCE`: Debounce delay in ms for `--watch`
- `RUST_LOG`: Set logging level (e.g., `info`)

### Anchor Program Generation

The `anchor generate` command now requires an explicit program address via `--address` and validates it as a base58-encoded 32-byte Solana program ID.

Example:

```
lumos anchor generate examples/basic/schema.lumos --address Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS --typescript
```
   git clone https://github.com/RECTOR-LABS/lumos.git
   cd lumos
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

4. **Run the CLI locally**
   ```bash
   cargo run --package lumos-cli -- --help
   ```

5. **Install git hooks (recommended)**
   ```bash
   bash .github/scripts/install-hooks.sh
   ```
   This installs a pre-commit hook that automatically validates `.lumos` schema files before each commit.

## 📋 How to Contribute

### Reporting Bugs

If you find a bug, please create an issue with:
- A clear, descriptive title
- Steps to reproduce the issue
- Expected vs actual behavior
- Your environment (OS, Rust version, LUMOS version)
- Code samples if applicable

### Suggesting Features

Feature requests are welcome! Please create an issue describing:
- The problem you're trying to solve
- Your proposed solution
- Any alternatives you've considered
- Why this would be useful to other users

### Pull Requests

1. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Follow the existing code style
   - Add tests for new functionality
   - Update documentation as needed

3. **Run checks locally**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features
   cargo test --all
   ```

4. **Commit your changes**
   - Write clear, concise commit messages
   - Reference any related issues
   - If you've installed git hooks, `.lumos` files are automatically validated

5. **Push and create a pull request**
   ```bash
   git push origin feature/your-feature-name
   ```

## 📝 Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Address all Clippy warnings (`cargo clippy`)
- Write tests for new functionality
- Document public APIs with doc comments

## 🧪 Testing

- Unit tests: `cargo test --lib`
- Integration tests: `cargo test --test '*'`
- All tests: `cargo test --all`

## 📚 Documentation

- Update README.md for user-facing changes
- Add doc comments for public APIs
- Update examples if behavior changes

## 🏗️ Project Structure

```
lumos/
├── packages/
│   ├── core/          # Core schema parsing and IR
│   └── cli/           # Command-line interface
├── examples/          # Usage examples
└── docs/              # Documentation
```

## 🤝 Code of Conduct

Be respectful and inclusive. We're building for the Solana community together.

## ❓ Questions?

- Open an issue for questions about contributing
- Tag @rz1989s for urgent matters

## 📄 License

By contributing, you agree that your contributions will be dual-licensed under MIT OR Apache-2.0, matching the project's license.

---

**Thank you for helping make LUMOS better!**
