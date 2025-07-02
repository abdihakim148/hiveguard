# Hiveguard: A Hexagonal Authentication System in Rust 🛡️🐝

![Hiveguard Logo](logo.svg)

## Project Overview

Hiveguard is a robust and highly adaptable authentication system meticulously crafted in Rust, leveraging the principles of Hexagonal Architecture (Ports and Adapters). Our core philosophy is to provide developers with unparalleled flexibility, allowing them to integrate their preferred database technologies, API protocols, and verification mechanisms without being constrained by rigid, pre-configured choices.

This project aims to deliver a comprehensive authentication solution that is both secure and extensible, empowering developers to build resilient applications with ease.

## 🌟 Key Features

### Architectural Design
*   **Hexagonal Architecture (Ports & Adapters)**: Designed for maximum flexibility and testability, ensuring that business logic remains independent of external concerns like databases, UI, or third-party services. This allows for easy swapping of adapters (e.g., different databases, email providers, or API protocols) without altering the core domain logic.

### Authentication Mechanisms
*   **Simple Authentication**: Robust user registration, login, and password management.
*   **3rd Party OAuth2.0 & OpenID Connect (OIDC) Integration**: Seamlessly connect with popular identity providers (e.g., Google, GitHub, Facebook) for streamlined user authentication.
*   **Self-Hosted OAuth2.0 & OpenID Connect Server**: The capability to act as an OAuth2.0 and OIDC provider, allowing other applications to authenticate against Hiveguard.

### Security & Tokenization
*   **Argon2 Password Hashing**: Utilizes the industry-recommended Argon2 algorithm for secure and computationally intensive password hashing, protecting against brute-force attacks.
*   **Flexible Tokenization (JWT / PASETO)**: Choose between JSON Web Tokens (JWT) or Platform-Agnostic Security Tokens (PASETO) for secure, tamper-proof token generation, configurable via a simple feature flag.

### Configuration
*   **JSON-based Configuration**: Start with a straightforward JSON configuration for easy setup and management.
*   **Future YAML Support**: Planned support for YAML configuration to offer more advanced and human-readable configuration options.

## 🚀 Feature Status

We believe in transparency and clear communication regarding our development progress. Below is an overview of the features and their current status:

| Feature Category | Feature | Status | Icon | Notes |
| :--------------- | :------ | :----- | :--- | :---- |
| **Core Auth** | User Registration & Login | Implemented | ✅ | Basic user lifecycle management. |
| | Password Hashing (Argon2) | Implemented | ✅ | Secure password storage. |
| | Session Management | Implemented | ✅ | Handling user sessions. |
| | Token Generation (JWT/PASETO) | In-Progress | 🚧 | Core logic for tokenization is being developed, feature flag for choice is planned. |
| **Adapters** | DynamoDB Database Adapter | Implemented | ✅ | Ready for use with AWS DynamoDB. |
| | Email Verification | Implemented | ✅ | Via `lettre` crate and `email` feature. |
| **Integrations** | 3rd Party OAuth2.0/OIDC Client | In-Progress | 🚧 | Integration with external providers is actively being worked on. |
| | Self-Hosted OAuth2.0/OIDC Server | Planned | 💡 | Future development to enable Hiveguard as an identity provider. |
| **Configuration** | JSON Configuration | Implemented | ✅ | Initial configuration setup. |
| | YAML Configuration | Planned | 💡 | To be added for enhanced configuration flexibility. |
| **Verification** | Phone Number Verification | Planned | 💡 | Support for SMS-based verification. |
| **Advanced** | Multi-Factor Authentication (MFA) | Unplanned | ⚪ | Considered for future iterations based on demand. |
| | Role-Based Access Control (RBAC) | Unplanned | ⚪ | Considered for future iterations based on demand. |
| | Audit Logging | Unplanned | ⚪ | Considered for future iterations based on demand. |

**Legend:**
*   ✅ **Implemented**: Feature is complete and available.
*   🚧 **In-Progress**: Feature is currently under active development.
*   💡 **Planned**: Feature is designed and scheduled for future implementation.
*   ⚪ **Unplanned**: Feature is not currently on the roadmap but may be considered later.

## ✨ Features of a Robust Authentication System (General)

A truly robust authentication system goes beyond basic login. Hiveguard aims to incorporate many of these principles:

*   **Secure Password Storage**: Using strong hashing algorithms (like Argon2) with proper salting.
*   **Multi-Factor Authentication (MFA)**: Adding extra layers of security (e.g., TOTP, SMS, biometrics).
*   **Session Management**: Securely handling user sessions, including expiration, invalidation, and renewal.
*   **Password Reset & Recovery**: Secure mechanisms for users to regain access to their accounts.
*   **Account Lockout & Brute-Force Protection**: Mitigating automated attack attempts.
*   **Rate Limiting**: Preventing abuse of authentication endpoints.
*   **Email/Phone Verification**: Confirming user identity during registration or sensitive operations.
*   **OAuth2.0 & OpenID Connect Support**: For seamless integration with external identity providers and building federated identity solutions.
*   **Role-Based Access Control (RBAC)**: Managing permissions and access levels for different user roles.
*   **Audit Logging**: Recording authentication events for security monitoring and compliance.
*   **Secure Token Management**: Proper generation, storage, and validation of access and refresh tokens.
*   **Cross-Site Request Forgery (CSRF) Protection**: Preventing unauthorized commands from a trusted user.
*   **Security Headers**: Implementing HTTP security headers (e.g., HSTS, CSP) for web-based authentication.
*   **Regular Security Audits**: Continuously reviewing and improving the security posture.

## 🛠️ Getting Started

*(Detailed instructions on how to set up, build, and run Hiveguard will be provided here.)*

## 🤝 Contributing

We welcome contributions from the community! If you're interested in improving Hiveguard, please refer to our contribution guidelines.

## 📄 License

This project is licensed under the [MIT License](LICENSE).