# Contributing to Netabase

Thank you for your interest in contributing to Netabase! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the Rust Code of Conduct

## Getting Started

### Prerequisites

- Rust 1.70+ (latest stable recommended)
- Git
- Familiarity with:
  - Rust async programming (tokio)
  - libp2p networking concepts
  - Database fundamentals

### Quick Start

```bash
# Clone the repository
git clone https://github.com/newsnet-africa/netabase.git
cd netabase

# Build the project
cargo build --features native

# Run tests
cargo test --features native

# Run examples
cargo run --example simple_mdns_chat --features native alice
```

## Development Setup

### Install Development Tools

```bash
# Install cargo-nextest (faster test runner)
cargo install cargo-nextest

# Install wasm-pack (for WASM development)
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Install cargo-watch (optional, for auto-rebuild)
cargo install cargo-watch
```

### Recommended VS Code Extensions

- rust-analyzer
- CodeLLDB (debugging)
- Even Better TOML
- crates (dependency management)

### Environment Variables

```bash
# Enable debug logging
export RUST_LOG=debug

# Or for specific modules
export RUST_LOG=netabase=debug,libp2p=info

# Enable backtrace
export RUST_BACKTRACE=1
```

## Project Structure

```
netabase/
├── src/
│   ├── lib.rs              # Main crate entry, Netabase struct
│   ├── errors.rs           # Error types
│   └── network/
│       ├── mod.rs          # Network module root
│       ├── config.rs       # Configuration structs
│       ├── store.rs        # NetabaseStore implementation
│       └── swarm/          # Swarm and behavior implementations
├── examples/               # Example applications
├── tests/                  # Integration tests
│   ├── p2p_integration_tests.rs
│   ├── dht_advanced_tests.rs
│   └── chat_integration_tests.rs
├── benches/                # Performance benchmarks
└── docs/                   # Additional documentation
```

### Key Components

#### 1. Netabase Struct (`src/lib.rs`)
- Main public API
- Manages swarm lifecycle
- Provides typed record operations
- Event subscription system

#### 2. Network Layer (`src/network/`)
- **config.rs**: Storage backend selection, network configuration
- **store.rs**: Bridge between netabase_store and libp2p RecordStore
- **swarm/**: libp2p swarm handlers, behaviors, commands

#### 3. Storage Layer (`netabase_store`)
- Type-safe key-value operations
- Multi-backend support (Sled, Redb)
- Secondary key indexing

## Making Changes

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring
- `test/description` - Test additions/improvements

### Commit Messages

Follow conventional commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Test changes
- `chore`: Maintenance

Examples:
```
feat(dht): add batch record insertion support

Implement bulk record insertion to improve performance when
adding multiple records to the DHT.

Closes #123
```

```
fix(swarm): resolve connection timeout issue

The swarm was not properly handling connection timeouts,
causing hangs. Added proper timeout handling and error
propagation.

Fixes #456
```

## Testing

### Unit Tests

```bash
# Run all unit tests
cargo test --lib --features native

# Run specific test
cargo test --lib test_name --features native

# Run with output
cargo test --lib --features native -- --nocapture
```

### Integration Tests

```bash
# Run all integration tests
cargo test --features native

# Run specific integration test
cargo test --test p2p_integration_tests --features native

# Run with nextest (parallel execution)
cargo nextest run --features native
```

### Doc Tests

```bash
# Run documentation tests
cargo test --doc --features native
```

### P2P Integration Tests

These tests spawn multiple processes:

```bash
# Must use single thread to avoid port conflicts
cargo test --test p2p_integration_tests --features native -- --test-threads=1
```

### Writing Tests

#### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration() {
        let config = NetabaseConfig::default();
        assert_eq!(config.backend, StorageBackend::Sled);
    }
}
```

#### Integration Test Example

```rust
#[tokio::test]
async fn test_record_storage() -> Result<()> {
    let mut netabase = Netabase::<TestDefinition>::new()?;
    netabase.start_swarm().await?;

    let record = TestRecord { id: 1, data: "test".to_string() };
    netabase.put_record(record).await?;

    // Assertions...
    Ok(())
}
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench --features native

# Run specific benchmark
cargo bench --bench dht_operations --features native

# With profiling (generates flamegraphs)
./profile_benches.sh
```

## Code Style

### Formatting

```bash
# Check formatting
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

### Linting

```bash
# Run clippy
cargo clippy --features native -- -D warnings

# Fix auto-fixable issues
cargo clippy --fix --features native
```

### Documentation

All public APIs must have documentation:

```rust
/// Brief description of what this does.
///
/// # Arguments
///
/// * `arg` - Description of argument
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function can error and why
///
/// # Examples
///
/// ```rust
/// use netabase::Netabase;
///
/// let netabase = Netabase::new()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn some_function(arg: Type) -> Result<ReturnType> {
    // Implementation
}
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `anyhow::Result` in application code
- Use typed errors (`NetabaseError`) in library code
- Provide context with `.context()` or `.with_context()`

```rust
use anyhow::{Context, Result};

fn example() -> Result<()> {
    some_operation()
        .context("Failed to perform operation")?;
    Ok(())
}
```

### Async Code

- Use `tokio` runtime
- Prefer `async/await` over manual futures
- Use `tokio::select!` for cancellation
- Document blocking operations

```rust
use tokio::time::{timeout, Duration};

async fn with_timeout() -> Result<Response> {
    timeout(Duration::from_secs(30), operation())
        .await
        .context("Operation timed out")?
}
```

## Submitting Changes

### Pull Request Process

1. **Fork and Create Branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make Changes**
   - Write code
   - Add tests
   - Update documentation
   - Run tests locally

3. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

4. **Push to Fork**
   ```bash
   git push origin feature/my-feature
   ```

5. **Create Pull Request**
   - Go to GitHub
   - Click "New Pull Request"
   - Fill out the template
   - Link related issues

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Doc tests pass
- [ ] Manual testing performed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings
- [ ] Tests added/updated
- [ ] All tests passing
```

### Review Process

- Maintainers will review within 1-2 weeks
- Address feedback in new commits
- Once approved, squash and merge
- PR will be merged by maintainers

## Release Process

Releases follow semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes

### Release Checklist

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Run full test suite
4. Create git tag
5. Push to crates.io
6. Create GitHub release

## Feature Development Guidelines

### Adding New Features

1. **Discuss First**: Open an issue to discuss the feature
2. **Design Doc**: For large features, write a design document
3. **Incremental PRs**: Break into smaller, reviewable PRs
4. **Feature Flags**: Use feature flags for experimental features
5. **Documentation**: Update docs and examples
6. **Tests**: Add comprehensive tests

### Performance Considerations

- Benchmark performance-critical code
- Avoid unnecessary allocations
- Use zero-copy where possible
- Profile with flamegraphs (`profile_benches.sh`)
- Document time/space complexity

### Security Considerations

- Never log sensitive data
- Validate all inputs
- Use secure defaults
- Follow OWASP guidelines
- Report vulnerabilities privately

## Architecture Guidelines

### libp2p Integration

- Use existing behaviors when possible
- Follow libp2p patterns
- Handle network events properly
- Implement proper backpressure

### Storage Integration

- Use `netabase_store` abstractions
- Don't bypass type system
- Handle serialization errors
- Consider batch operations

### Type Safety

- Leverage Rust's type system
- Use newtypes for IDs
- Make invalid states unrepresentable
- Prefer compile-time checks

## Getting Help

- **Documentation**: Check [GETTING_STARTED.md](./GETTING_STARTED.md) and [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Issues**: Search existing issues on GitHub
- **Discussions**: Use GitHub Discussions for questions
- **Examples**: Look at code in `examples/` directory

## Resources

### Netabase Resources
- [Getting Started Guide](./GETTING_STARTED.md)
- [Architecture Overview](./ARCHITECTURE.md)
- [Profiling Guide](./PROFILING.md)

### External Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [libp2p Documentation](https://docs.libp2p.io/)
- [tokio Documentation](https://tokio.rs/)

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.

## Recognition

Contributors are recognized in:
- CHANGELOG.md
- GitHub contributors page
- Release notes

Thank you for contributing to Netabase! 🎉
