<div align="center">
    <img src="logo.png" alt="Beekeeper Logo" width="200">
</div>

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Contributors](https://img.shields.io/github/contributors/abdihakim148/beekeeper)
![Issues](https://img.shields.io/github/issues/abdihakim148/beekeeper)

</div>

# Beekeeper

Beekeeper is an open-source Authentication, Authorization, and User Management system. It supports both third-party and custom OAuth2.0 and OpenID Connect (OpenIDC) implementations. Built with Rust, it leverages the hexagonal architecture to allow easy swapping of adaptors, databases, and input methods.

## Features

- **Security**: Utilizes Argon2 for password hashing and PASETO for secure token management.
- **Flexible Architecture**: Hexagonal architecture for easy integration and adaptability.
- **In-Memory Database**: Currently supports an in-memory database for quick setup and testing.
- **HTTP API**: Built with Actix for handling HTTP requests.

## Planned Features

### Databases
- [x] In-Memory database support
- [ ] SQLite database support
- [ ] MongoDB database support
- [ ] DynamoDB database support
- [ ] PostgreSQL database support

### Domain
- [ ] Third-party OAuth2.0 and OpenID Connect integration
- [ ] Own OAuth2.0 and OpenID Connect implementation
- [ ] Email validation
- [ ] Logging
- [ ] Organization Management
- [ ] Service Management
- [ ] Comprehensive Roles and Permissions Strategy

### Interface
- [x] HTTP API with Actix
- [ ] GRPC input method
- [ ] Lambda input method
- [ ] YAML configuration support
- [x] JSON configuration support

## Installation

To install Beekeeper, ensure you have Rust installed on your system. Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/abdihakim148/beekeeper.git
cd beekeeper
cargo build --release
```

## Getting Started

To get started with Beekeeper, clone the repository and follow the setup instructions in the [documentation](docs/SETUP.md).

## Contributing

Contributions are welcome! Please read the [contributing guidelines](CONTRIBUTING.md) first.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

For questions or suggestions, please open an issue or contact the maintainers.

---
