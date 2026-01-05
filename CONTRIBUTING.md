# Contributing to Reputest

First off, thank you for considering contributing to Reputest! It's people like you that make open source great.

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

- **Use a clear and descriptive title**
- **Describe the exact steps to reproduce the problem**
- **Provide specific examples** (code snippets, config files)
- **Describe the behavior you observed and what you expected**
- **Include logs** with `RUST_LOG=debug` if applicable
- **Specify your environment** (OS, Rust version, Docker version)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion:

- **Use a clear and descriptive title**
- **Provide a detailed description** of the suggested enhancement
- **Explain why this enhancement would be useful**
- **List any alternatives you've considered**

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Follow the coding style** of the project
3. **Write tests** for any new functionality
4. **Ensure all tests pass** locally before submitting
5. **Update documentation** as needed
6. **Write a clear PR description** explaining your changes

## Development Setup

### Prerequisites

- Rust (latest stable) - install via [rustup](https://rustup.rs/)
- Docker (for containerization)
- PostgreSQL (or use Docker)

### Getting Started

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/reputest.git
cd reputest

# Create a branch for your changes
git checkout -b feature/your-feature-name

# Copy environment template
cp .env.example .env
# Edit .env with your configuration

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Code Quality

Before submitting a PR, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test
```

Or use the Makefile:

```bash
make checkall
```

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests when relevant

### Documentation

- Update the README.md if you change functionality
- Add rustdoc comments for public functions
- Include examples in documentation where helpful

## Project Structure

```
reputest/
├── src/
│   ├── main.rs          # Application entry point
│   ├── config.rs        # Configuration handling
│   ├── handlers.rs      # HTTP route handlers
│   ├── twitter/         # Twitter API integration
│   ├── oauth.rs         # OAuth 2.0 implementation
│   ├── crypto.rs        # Token encryption
│   ├── db.rs            # Database operations
│   ├── cronjob.rs       # Scheduled tasks
│   └── tests.rs         # Test suite
├── sql/                 # Database schema
├── scripts/             # Utility scripts
└── docs/                # Additional documentation
```

## Questions?

Feel free to open an issue for any questions about contributing!

