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

5. **Environment variables**

   Copy the example environment file and customize optional values for local development:

   ```bash
   cp .env.example .env
   ```

   The repository includes a `.env.example` template documenting the optional environment variables you can configure:

   - `LUMOS_WATCH_DEBOUNCE`: Watch mode debounce duration in milliseconds (default `100`, max `5000`). Controls how quickly file changes trigger rebuilds when running in watch mode.
   - `RUST_LOG`: Logging level for tracing and debugging (common values: `trace`, `debug`, `info`, `warn`, `error`). Default is `info`.
   - `SENTRY_DSN`: Error monitoring DSN (optional). If you use Sentry for error reporting, set this value; otherwise leave it empty.

   Example (from `.env.example`):

   ```text
   # LUMOS_WATCH_DEBOUNCE=100
   # RUST_LOG=info
   # SENTRY_DSN=
   ```

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
