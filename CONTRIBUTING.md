# Contributing to Omne Validator

Thank you for your interest in contributing to the Omne Validator! This document provides guidelines for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Code of Conduct

By participating in this project, you agree to abide by our Code of Conduct. Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/omne-validator.git
   cd omne-validator
   ```
3. Add the upstream repository as a remote:
   ```bash
   git remote add upstream https://github.com/omne-network/omne-validator.git
   ```

## Development Setup

### Prerequisites

- Rust 1.70+ with Cargo
- Git
- A text editor or IDE (VS Code recommended)

### Setup

1. Install Rust dependencies:
   ```bash
   cargo check
   ```

2. Install development tools:
   ```bash
   # Install clippy for linting
   rustup component add clippy
   
   # Install rustfmt for formatting
   rustup component add rustfmt
   ```

3. Run tests to ensure everything works:
   ```bash
   cargo test
   ```

## Making Changes

### Branch Naming

- Feature branches: `feature/your-feature-name`
- Bug fixes: `fix/issue-description`
- Documentation: `docs/what-you-changed`

### Code Style

We use standard Rust conventions:

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix any warnings
- Follow Rust naming conventions
- Add documentation for public APIs
- Include tests for new functionality

### Commit Messages

Follow conventional commits:

```
type(scope): description

Examples:
feat(consensus): add PoVERA block validation
fix(p2p): resolve connection timeout issue
docs(readme): update installation instructions
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration_tests
```

### Writing Tests

- Unit tests: Place in the same file as the code being tested
- Integration tests: Place in the `tests/` directory
- Documentation tests: Include examples in doc comments

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        // Test implementation
    }

    #[tokio::test]
    async fn test_async_function() {
        // Async test implementation
    }
}
```

## Submitting Changes

### Pull Request Process

1. Update your fork with the latest changes:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. Create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. Make your changes and commit them:
   ```bash
   git add .
   git commit -m "feat(scope): your change description"
   ```

4. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

5. Create a pull request on GitHub

### Pull Request Requirements

- [ ] Code follows style guidelines
- [ ] Tests pass (`cargo test`)
- [ ] Code compiles without warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated if needed
- [ ] Commit messages follow conventional commits
- [ ] PR description explains the changes

### Review Process

1. Automated checks must pass (CI/CD)
2. At least one maintainer review required
3. Address any feedback or requested changes
4. Maintainer will merge when approved

## Release Process

Releases follow semantic versioning (SemVer):

- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backwards compatible
- **Patch** (0.0.1): Bug fixes, backwards compatible

### Types of Contributions

- **Bug Reports**: Create an issue with reproduction steps
- **Feature Requests**: Create an issue describing the feature
- **Code Contributions**: Follow the process above
- **Documentation**: Improvements to docs, comments, examples
- **Testing**: Additional test cases, test improvements

### Areas Needing Contribution

- [ ] Performance optimizations
- [ ] Additional test coverage
- [ ] Documentation improvements
- [ ] Example implementations
- [ ] CI/CD improvements
- [ ] Security audits

## Getting Help

- **Discord**: Join our [Discord server](https://discord.gg/omne)
- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Email**: dev@omne.network for security issues

## License

By contributing to Omne Validator, you agree that your contributions will be licensed under the Apache 2.0 License.
