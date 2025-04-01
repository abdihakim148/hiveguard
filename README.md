<div align="center">
    <img src="logo.svg" alt="Hiveguard Logo" width="100%">
</div>

<div align="center">

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Contributors](https://img.shields.io/github/contributors/abdihakim148/hiveguard)](https://github.com/abdihakim148/hiveguard/graphs/contributors)
[![Issues](https://img.shields.io/github/issues/abdihakim148/hiveguard)](https://github.com/abdihakim148/hiveguard/issues)
[![Build Status](https://travis-ci.com/abdihakim148/hiveguard.svg?branch=main)](https://travis-ci.com/abdihakim148/hiveguard)

</div>

# Hiveguard

Hiveguard is an open-source Authentication, Authorization, and User Management system built with Rust. It provides a robust, flexible, and secure solution for identity and access management using modern software design principles.

> **Note:** Hiveguard is built with a modular **Ports and Adapters** architecture. We welcome contributions! Feel free to implement an adapter for any input or output interface (like a specific database, messaging queue, or API protocol), even if it's not listed in the planned features.

---

## Core Technologies

- **Language**: Rust
- **Core Design**: Hexagonal (Ports and Adapters) Architecture
- **Concurrency**: Tokio asynchronous runtime
- **Serialization/Deserialization**: Serde
- **Configuration**: Serde JSON with environment variable support for secrets

*Specific technologies for interfaces like web frameworks, databases, hashing algorithms, and token formats are implemented as adapters. See the [Features](#features) section for details on currently supported and planned adapters.*

---

## Architecture

Hiveguard implements a **Hexagonal (Ports and Adapters) Architecture**, which provides:

-   **Clear Separation of Concerns**: Domain logic is isolated from application and infrastructure details.
-   **Flexibility**: Easily swap infrastructure components (like databases or email providers) by implementing corresponding ports.
-   **Testability**: Domain logic can be tested independently of external services.

**Key Architectural Components:**
- **Domain (`src/domain`)**: Contains the core business logic, entities (User, Organisation, etc.), value objects, and domain services (Authentication, Verification, etc.). It is independent of any specific framework or infrastructure.
- **Ports (`src/ports`)**: Defines the interfaces (traits) connecting the domain to the outside world.
    - **Input Ports**: Define how external actors (like web requests or CLI commands) interact with the application (e.g., `Config` loading).
    - **Output Ports**: Define interfaces for infrastructure concerns that the domain logic needs (e.g., `database` operations, `verify` operations).
- **Adapters (`src/adaptors`)**: Provide concrete implementations (adapters) for the ports defined in `src/ports`.
    - **Input Adapters**: Implement input ports (e.g., Actix Web controllers handling HTTP requests).
    - **Output Adapters**: Implement output ports (e.g., an in-memory database adapter, an SMTP email verification adapter).

---

## Features

**Legend:**

*   `[x]` - **Completed**
*   `[~]` - **Partially Completed**
*   `[ ]` - **Planned / Not Started**

### Domain Features (`src/domain`)

-   **`[x]` Core Entities:** `User`, `Organisation`, `Service`, `Member`, `Role`, `Resource`, `Scope`, `Grant`, `Permission` defined with strong typing.
-   **`[x]` Value Objects:** Custom types for `Id` (ObjectId wrapper), `EmailAddress`, `Phone`, `Contact` (handles Email/Phone/Both), `Number`, `Value` (dynamic type), etc., with validation and Serde support.
-   **`[x]` Authentication Service:**
    -   Defines `Authentication` trait for user registration, password-based login, and token authorization.
    -   Includes `Password` trait for hashing and verification (abstracted from specific algorithm).
    -   Includes `Paseto` trait for token signing and verification (abstracted from specific token format/version).
-   **`[~]` Verification Service:**
    -   Defines `Verification` trait for initiating and confirming contact ownership (Email/Phone). Abstracted from the delivery method.
-   **`[~]` OAuth Client Service:**
    -   Defines `OAuth` trait for client-side OAuth2 flows (authorization URL generation, code exchange, user info retrieval).
-   **`[x]` Configuration Loading:** Defines `Config` trait for loading application settings.
-   **`[x]` Flexible Error Handling:** Domain-specific errors defined in `src/domain/types/error.rs`, implementing `ErrorTrait`.
-   **`[ ]` Authorization Logic:** RBAC based on Roles, Grants, Permissions, Scopes (types defined, but enforcement logic not fully implemented).
-   **`[ ]` JWT Support:** Planned alternative token format alongside PASETO.

---

### Interface Ports & Adapters (`src/ports`, `src/adaptors`)

#### Input Interfaces (Driving Adapters)

*   **Configuration Loading (`ports::inputs::config::Config`)**
    *   **`[x]` JSON File Adapter:** Implemented directly within the `Config` struct in `domain::types::config`, loads from `config.json` by default.
*   **API Protocols**
    *   **`[~]` HTTP API (`adaptors::inputs::api::actix`)**
        *   **Adapter:** Actix Web (v4)
        *   **Status:** Partially implemented. Provides endpoints for signup, login, user info/patching, verification request/confirm, OAuth initiation/callback. Requires `http` feature flag.
    *   **`[ ]` gRPC API**
    *   **`[ ]` GraphQL API**
    *   **`[ ]` Command Line Interface (CLI)**

#### Output Interfaces (Driven Adapters)

*   **Database Operations (`ports::outputs::database`)**
    *   Defines traits for CRUD operations: `CreateItem`, `GetItem`, `GetItems`, `UpdateItem`, `DeleteItem`.
    *   **Adapters:**
        *   **`[x]` In-Memory Adapter (`adaptors::outputs::database::memory`)**: Thread-safe (`RwLock`) implementation with indexing. Requires `memory` feature flag.
        *   **`[ ]` PostgreSQL Adapter**
        *   **`[ ]` MySQL/MariaDB Adapter**
        *   **`[ ]` MongoDB Adapter**
        *   **`[ ]` SQLite Adapter**
*   **Contact Verification (`ports::outputs::verify::Verify`)**
    *   Defines trait for initiating and confirming contact verification via different channels (Email, SMS, Whatsapp).
    *   **Adapters:**
        *   **`[x]` SMTP Adapter (`adaptors::outputs::verify::smtp`)**: Sends verification emails using SMTP via Lettre. Requires `smtp` and `email` feature flags. Includes HTML template.
        *   **`[x]` Twilio Adapter (`adaptors::outputs::verify::twilio`)**: Sends verification via Twilio Verify API (SMS/Whatsapp/Email). Requires `twilio-phone` or `twilio-email` feature flags. Supports custom codes or Twilio-generated codes.
        *   **`[ ]` Other Providers (e.g., SendGrid, AWS SES/SNS)**

---

## Configuration

Hiveguard uses a `config.json` file (by default) for its settings. If the file doesn't exist on startup, a default one will be generated.

**Key Configuration Sections:**

*   **`name`**: (String) The name of the application/issuer (e.g., `"Hiveguard"`).
*   **`domain`**: (String) The domain where the application is hosted (e.g., `"auth.example.com"`). Used for constructing verification links.
*   **`database`**: (Object) Configuration for the database adapter. For the default `memory` adapter, this is an empty object `{}`.
*   **`argon`**: (Object) Settings for password hashing:
    *   `algorithm`: (String Enum) `"Argon2d"`, `"Argon2i"`, or `"Argon2id"` (default).
    *   `version`: (Integer) `16` (for 0x10) or `19` (for 0x13 - default).
    *   `params`: (Object) Argon2 parameters:
        *   `memory_cost`: (Integer) Memory cost in KiB (e.g., `19456`).
        *   `time_cost`: (Integer) Number of iterations (e.g., `2`).
        *   `parallelism`: (Integer) Degree of parallelism (e.g., `1`).
        *   `output_length`: (Integer, Optional) Desired hash length in bytes.
    *   `pepper`: (String, Optional) Secret pepper. Can use environment variable substitution (e.g., `"$PEPPER_SECRET"`).
*   **`paseto`**: (Object) Settings for PASETO tokens:
    *   `path`: (String) Path to the file storing PASETO keys (e.g., `"paseto_keys.json"`).
    *   `ttl`: (Integer) Token Time-To-Live in seconds (e.g., `86400` for 24 hours).
*   **`verifyer`**: (Object) Configuration for the contact verification adapter. Examples:
    *   **SMTP:**
        ```json
        {
          "url": "smtp://localhost:1025",
          "credentials": { "username": "user", "password": "$SMTP_PASSWORD" },
          "sender": "\"Hiveguard Verification\" <noreply@example.com>"
        }
        ```
    *   **Twilio:**
        ```json
        {
          "account_sid": "$TWILIO_ACCOUNT_SID",
          "service_sid": "$TWILIO_VERIFY_SERVICE_SID",
          "credentials": { "username": "$TWILIO_ACCOUNT_SID", "password": "$TWILIO_AUTH_TOKEN" },
          "friendly_name": "MyApp Verification", // Optional
          "custom_code": false // Use Twilio's code generation or Hiveguard's
          // "base_url": "https://verify.twilio.com/v2" // Optional, defaults to Twilio Verify API
        }
        ```
*   **`oauth`**: (Object) Configuration for OAuth clients (acting as a client to other providers). Keys are provider names (e.g., `"google"`, `"github"`).
    *   `<provider_name>`: (Object)
        *   `client_id`: (String) Client ID from the provider.
        *   `client_secret`: (String) Client secret from the provider (use env var: `"$GOOGLE_CLIENT_SECRET"`).
        *   `auth_url`: (String) Provider's authorization endpoint URL.
        *   `token_url`: (String) Provider's token endpoint URL.
        *   `userinfo_url`: (String) Provider's user info endpoint URL.
        *   `scopes`: (Array of Strings) OAuth scopes to request (e.g., `["openid", "email", "profile"]`).
        *   `fields`: (Array of Strings/Null) Mapping from provider fields to Hiveguard `User` fields (`id`, `username`, `first_name`, `last_name`, `email`, `email_verified`, `phone`, `phone_verified`, `password`). Use `null` to use Hiveguard's default field name. Length must match `User::FIELDS`.

---

**Example `config.json`:**

```json
{
  "name": "Hiveguard",
  "domain": "localhost:8080",
  "database": {}, // Settings for the 'memory' database adapter
  "argon": {
    "algorithm": "Argon2id",
    "version": 19,
    "params": {
      "memory_cost": 19456,
      "time_cost": 2,
      "parallelism": 1,
      "output_length": null
    },
    "pepper": "$ARGON_PEPPER" // Load from ARGON_PEPPER environment variable
  },
  "paseto": {
    "path": "paseto_keys.json",
    "ttl": 3600 // 1 hour
  },
  "verifyer": { // Example using SMTP
    "url": "smtp://localhost:1025", // Mailhog/Mailpit local instance
    "credentials": null, // No auth for local testing
    "sender": "\"Hiveguard\" <noreply@localhost>"
  },
  "oauth": {
    "google": {
      "client_id": "$GOOGLE_CLIENT_ID",
      "client_secret": "$GOOGLE_CLIENT_SECRET",
      "auth_url": "https://accounts.google.com/o/oauth2/v2/auth",
      "token_url": "https://oauth2.googleapis.com/token",
      "userinfo_url": "https://openidconnect.googleapis.com/v1/userinfo",
      "scopes": ["openid", "email", "profile"],
      "fields": [
        null, // id (use default)
        "email", // username
        "given_name", // first_name
        "family_name", // last_name
        "email", // email
        "email_verified", // email_verified
        null, // phone (use default)
        null, // phone_verified (use default)
        null // password (use default)
      ]
    }
    // Add other providers like "github", "facebook", etc.
  }
}
```

---

## Installation & Setup

**Prerequisites:**

-   Rust (stable toolchain recommended)
-   Cargo (comes with Rust)

**Steps:**

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/abdihakim148/hiveguard.git
    cd hiveguard
    ```
2.  **Configure:**
    *   Create a `config.json` file in the project root (or let the application generate a default one on first run). See the [Configuration](#configuration) section for details.
    *   Set up any required environment variables for secrets (e.g., `ARGON_PEPPER`, `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, etc.).
3.  **Build:**
    ```bash
    # For development
    cargo build

    # For release
    cargo build --release
    ```
4.  **Run:**
    ```bash
    # Development
    cargo run

    # Release
    ./target/release/hiveguard
    ```
    The application will start, typically listening on `127.0.0.1:8080` (configurable via Actix/environment if needed, though not directly in `config.json` currently).

---

## API Endpoints

(Current primary endpoints - more details to be added)

- `POST /signup`: Register a new user.
- `POST /login`: Authenticate a user with password.
- `GET /users/`: Get current user's information (requires token).
- `PATCH /users/`: Update current user's information (requires token).
- `POST /verify/request`: Request a verification code for a contact.
- `GET /verify/confirm/{code}`: Confirm verification using a code or magic link ID.
- `GET /login/oauth/{provider}`: Initiate OAuth login flow.
- `GET /login/oauth/{provider}/confirm`: Handle OAuth callback.

---

## Contributing

We welcome contributions! Please see our `CONTRIBUTING.md` (to be created) for details on how to get started, including code style, testing, and pull request processes.

**Key areas for contribution:**

-   Implementing planned features (Database adapters, OAuth Server, RBAC).
-   Expanding API endpoints for managing Organisations, Services, Members, Roles.
-   Improving test coverage.
-   Enhancing documentation.
-   Adding examples.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Contact & Support

-   **Issues:** For bug reports, feature requests, or questions, please use the [GitHub Issues](https://github.com/abdihakim148/hiveguard/issues) tracker.
