<div align="center">
    <img src="logo.png" alt="Beekeeper Logo" width="200">
</div>

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Contributors](https://img.shields.io/github/contributors/abdihakim148/beekeeper)
![Issues](https://img.shields.io/github/issues/abdihakim148/beekeeper)

</div>

# Beekeeper

Beekeeper is an open-source Authentication, Authorization, and User Management system built with Rust. It provides a robust, flexible, and secure solution for identity and access management using modern software design principles.

## Technology Stack

- **Language**: Rust
- **Web Framework**: Actix Web
- **Authentication**: 
  - PASETO (Platform-Agnostic Security Tokens)
  - Ed25519 Signatures
- **Password Hashing**: Argon2
- **Serialization**: Serde
- **Database**: In-Memory (with plans for multiple backends)

## Architecture

Beekeeper implements a Hexagonal (Ports and Adapters) Architecture, which provides:
- Clear separation of concerns
- Flexibility in swapping components
- Ease of testing
- Decoupled domain logic from infrastructure

Key architectural components:
- **Domain**: Core business logic and types
- **Ports**: Interfaces for input and output operations
- **Adaptors**: Concrete implementations of ports

## Features

### Completed
- [x] Secure password hashing with Argon2
- [x] PASETO token generation and validation
- [x] In-Memory database with thread-safe operations
- [x] HTTP API with Actix Web
- [x] JSON configuration support
- [x] User registration and management
- [x] Flexible error handling
- [x] Comprehensive type system with strong serialization

### Planned
- [ ] Multiple database backend support
- [ ] OAuth2.0 and OpenID Connect implementation
- [ ] Enhanced logging
- [ ] More authentication methods
- [ ] Advanced role and permission management

## Configuration

Beekeeper supports JSON-based configuration with flexible secret management:
- Environment variable secrets
- Multi-level secret retrieval
- Default configuration generation

Example configuration structure:
```json
{
  "database": { ... },
  "argon": {
    "algorithm": "Argon2id",
    "version": 19,
    "params": { ... }
  },
  "paseto": { ... },
  "mailer": { ... }
}
```

## Installation

Prerequisites:
- Rust (stable version)
- Cargo

```bash
git clone https://github.com/abdihakim148/beekeeper.git
cd beekeeper
cargo build --release
```

## Getting Started

1. Clone the repository
2. Configure your settings
3. Run the application
4. Explore the API documentation

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on how to get started.

## License

MIT License - See [LICENSE](LICENSE) for complete details.

## Contact & Support

- GitHub Issues: Report bugs or request features
- Email: [Maintainer Email]

---
