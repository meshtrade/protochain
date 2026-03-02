# Contributing to Protochain

Thank you for your interest in contributing to Protochain! We welcome contributions from the community.

## Table of Contents

- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Code Style](#code-style)
- [Testing](#testing)
- [Guidelines](#guidelines)
- [License](#license)

## Development Setup

Before contributing, we recommend reading the [README](README.md) to understand the project architecture and looking at existing issues and pull requests.

### Prerequisites
```bash
# Required tools
rustc --version    # Rust 1.70+
go version         # Go 1.21+
golangci-lint --version # Built for Go 1.21+
solana --version   # Solana CLI tools
buf --version      # Protocol buffer tools
```

### Protobuf Code Generation
This project makes use of protobuf for defining APIs and related types. <a href="https://buf.build/">Buf</a> is used to generate code from the protobuf definitions.

The flow when adding or updating proto files is as follows:

1. `buf lint` - perform linting checks on proto files.
2. `./scripts/code-gen/generate/all.sh` - executes entire code-gen pipeline

## How to Contribute

### Reporting Bugs

Open an issue with:

- Steps to reproduce the issue
- Expected vs actual behavior
- Your environment (OS, Rust version, Go version, TS version)
- Relevant logs or error messages

### Suggesting Features

Open an issue describing:

- The feature and the problem it solves
- Any implementation ideas (optional)

### Contributing Code

1. Fork the repository and create a branch for your work
2. Write code and tests
    - Unit & Integration tests
3. Submit a pull request

A maintainer will review the PR as soon as we have availability! 

## Code Style

### Linting

To run linting checks (from the project root): 
```bash
./scripts/lint/all.sh
```

## Testing

```bash
./scripts/lint/test.sh
```

## Guidelines

- Follow language-specific naming conventions
- Write clear, self-documenting code
- Add comments for complex logic
- Write/Update tests when making changes to API functionality

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
