# Contributing to QRNG-DD

We welcome contributions from the community! This document provides guidelines for contributing to the QRNG Data Diode project.

## Ways to Contribute

- **Bug Reports**: Submit detailed bug reports via [GitHub Issues](https://github.com/vbocan/qrng-data-diode/issues)
- **Feature Requests**: Propose new features through GitHub Issues with the `enhancement` label
- **Code Contributions**: Submit pull requests for bug fixes or new features
- **Documentation**: Improve documentation, add examples, or clarify existing content
- **Testing**: Add test cases, improve test coverage, or report edge cases
- **Research**: Contribute to academic publications, benchmarks, or case studies

## Getting Started

1. **Fork the Repository**
   ```bash
   git clone https://github.com/vbocan/qrng-data-diode.git
   cd qrng-data-diode
   ```

2. **Set Up Development Environment**
   - Install Rust 1.75 or later
   - Install Docker and Docker Compose
   - Install OpenSSL development libraries

3. **Build and Test**
   ```bash
   cargo build --release
   cargo test --all
   ```

## Development Workflow

### Creating a Feature Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b bugfix/your-bugfix-name
```

### Making Changes

1. **Write Clean Code**: Follow Rust conventions and use `cargo fmt` and `cargo clippy`
2. **Add Tests**: All new functionality should include appropriate tests
3. **Update Documentation**: Update README.md, inline docs, and relevant guides
4. **Commit Messages**: Use clear, descriptive commit messages

Example commit message format:
```
feat: Add support for custom QRNG sources

- Implement SourceProvider trait
- Add configuration for custom endpoints
- Include tests for new functionality

Closes #123
```

### Testing Your Changes

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features

# Run all tests
cargo test --all

# Run specific component tests
cargo test -p qrng-collector
cargo test -p qrng-gateway
cargo test -p qrng-mcp

# Run with verbose output
cargo test -- --nocapture
```

### Submitting a Pull Request

1. **Push Your Changes**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request**
   - Go to the GitHub repository
   - Click "New Pull Request"
   - Select your feature branch
   - Fill out the PR template with:
     - Description of changes
     - Related issue numbers
     - Testing performed
     - Breaking changes (if any)

3. **Code Review Process**
   - Maintainers will review your PR
   - Address any feedback or requested changes
   - Once approved, your PR will be merged

## Code Standards

### Rust Style Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` with default settings
- Address all `cargo clippy` warnings
- Write idiomatic Rust code

### Documentation Standards

- Add doc comments (`///`) for all public items
- Include examples in doc comments where helpful
- Update README.md for user-facing changes
- Add entries to CHANGELOG.md (if exists)

### Testing Standards

- Write unit tests for new functionality
- Add integration tests for API endpoints
- Include edge case tests
- Maintain or improve code coverage

## Project Structure

```
qrng-data-diode/
├── qrng-collector/     # Entropy collection component
├── qrng-gateway/       # Gateway REST API component
├── qrng-mcp/          # Model Context Protocol server
├── qrng-core/         # Shared core library
├── docs/              # Documentation
├── examples/          # Usage examples
└── scripts/           # Utility scripts
```

## Key Areas for Contribution

### High Priority

1. **Additional QRNG Sources**: Add support for more quantum random number sources
2. **Performance Optimizations**: Improve throughput and latency
3. **Documentation**: Expand guides, add tutorials, create diagrams
4. **Testing**: Increase test coverage, add chaos engineering tests
5. **Security Audits**: Review cryptographic implementations

### Enhancement Ideas

1. **Monitoring**: Enhanced metrics, Grafana dashboards, alerting
2. **Configuration**: More flexible configuration options
3. **Deployment**: Kubernetes manifests, Terraform modules
4. **Quality Validation**: Additional randomness tests (NIST suite, Dieharder)
5. **MCP Features**: New MCP tools, enhanced AI agent capabilities

## Reporting Issues

When reporting issues, please include:

- **Environment**: OS, Rust version, deployment method (Docker/native)
- **Steps to Reproduce**: Detailed steps to reproduce the issue
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Logs**: Relevant log output (with sensitive data removed)
- **Configuration**: Relevant configuration (with secrets redacted)

## Security Issues

**DO NOT** report security vulnerabilities through public GitHub issues.

Instead, please email security concerns to: valer.bocan@upt.ro

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fixes (if any)

We will respond within 48 hours and work with you to address the issue.

## License

By contributing to QRNG-DD, you agree that your contributions will be licensed under the MIT License.

## Questions?

- **GitHub Discussions**: Ask questions in [Discussions](https://github.com/vbocan/qrng-data-diode/discussions)
- **Email**: valer.bocan@upt.ro
- **Issues**: For bug reports and feature requests

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors.

### Our Standards

- **Be Respectful**: Treat everyone with respect and consideration
- **Be Collaborative**: Work together constructively
- **Be Professional**: Focus on technical merit
- **Be Patient**: Help newcomers learn and grow

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Personal attacks or trolling
- Publishing others' private information
- Other conduct inappropriate in a professional setting

---

Thank you for contributing to QRNG-DD! Your efforts help advance open-source quantum random number distribution and make quantum entropy accessible to researchers and practitioners worldwide.
