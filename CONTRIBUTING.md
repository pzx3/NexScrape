# Contributing to NexScrape

Thank you for your interest in contributing to NexScrape! 🕷️

## Getting Started

1. **Fork** the repository
2. **Clone** your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/nexscrape.git
   cd nexscrape
   ```
3. **Build** the project:
   ```bash
   cargo build --workspace
   ```
4. **Run tests**:
   ```bash
   cargo test --workspace
   ```

## Development Setup

### Prerequisites
- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Python 3.10+ (for Python bindings)
- `maturin` (for building Python bindings): `pip install maturin`

### Building

```bash
# Build everything
cargo build --workspace

# Build and test
cargo test --workspace

# Build Python bindings
cd nexscrape-python
maturin develop
```

## Code Style

### Rust
- Follow standard Rust conventions (`rustfmt`)
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Add doc comments (`///`) to all public items

### Python
- Follow PEP 8
- Use type hints
- Add docstrings to all public functions/classes

## Pull Request Process

1. Create a feature branch: `git checkout -b feature/my-feature`
2. Make your changes
3. Add tests for new functionality
4. Run `cargo test --workspace` and ensure all tests pass
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## Commit Messages

Use clear, descriptive commit messages:
- `feat: add proxy health checking`
- `fix: handle timeout in rate limiter`
- `docs: update API reference`
- `test: add bloom filter edge cases`
- `refactor: simplify middleware pipeline`

## Reporting Issues

- Use GitHub Issues
- Include reproduction steps
- Include your Rust/Python version
- Include relevant error messages

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
