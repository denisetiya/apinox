# Apinox

**Schema-first API Documentation Generator** — Define your API once in YAML/JSON, generate **7 output formats** from a single source of truth.

```
Schema (YAML/JSON) → Postman Collection
                   → OpenAPI 3.1 Spec
                   → Markdown Docs
                   → Scalar Interactive Docs
                   → Insomnia Import
                   → Hurl Test Scripts
                   → Curl Commands
```

[![CI](https://github.com/denisetiya/apinox/actions/workflows/release.yml/badge.svg)](https://github.com/denisetiya/apinox/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Linux / macOS](#linux--macos)
  - [Windows](#windows)
  - [Manual](#manual)
- [Quick Start](#quick-start)
- [Schema Format](#schema-format)
  - [Root Fields](#root-fields)
  - [Auth](#auth)
  - [Environments](#environments)
  - [Groups](#groups)
  - [Endpoints](#endpoints)
  - [Reusable Error Patterns](#reusable-error-patterns)
  - [Schema Includes (Modular)](#schema-includes-modular)
- [CLI Commands](#cli-commands)
  - [`validate`](#validate)
  - [`build`](#build)
  - [`watch`](#watch)
  - [`diff`](#diff)
  - [`migrate`](#migrate)
  - [`import`](#import)
  - [`sync`](#sync)
- [Output Formats](#output-formats)
  - [Postman Collection](#postman-collection)
  - [OpenAPI 3.1](#openapi-31)
  - [Markdown Documentation](#markdown-documentation)
  - [Scalar Interactive Docs](#scalar-interactive-docs)
  - [Insomnia Import](#insomnia-import)
  - [Hurl Test Scripts](#hurl-test-scripts)
- [CI/CD Integration](#cicd-integration)
- [Changelog & Diff](#changelog--diff)
- [Modular Schemas](#modular-schemas)
- [Postman Sync](#postman-sync)
- [Build from Source](#build-from-source)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **Single source of truth** — Define endpoints, schemas, auth, and examples once
- **7 output formats** — Postman, OpenAPI 3.1, Markdown, Scalar, Insomnia, Hurl, curl
- **Validation** — Catch missing fields, duplicate IDs, broken auth/group refs, path-param mismatches before generating
- **File watching** — `apinox watch` auto-rebuilds on schema changes (great for dev workflow)
- **OpenAPI import** — Convert existing OpenAPI 3.x / Swagger 2.0 specs to Apinox schema
- **Schema diff** — Compare two schema versions (breaking changes, additions, removals)
- **Migration guide** — Generate human-readable migration guides between schema versions
- **Postman sync** — Push generated collections directly to Postman workspace
- **Modular includes** — Split large schemas into multiple files with `includes`
- **Changelog tracking** — Endpoint-level changelog within the schema for version tracking
- **Cross-platform** — Linux, macOS, Windows (x86_64 + ARM64)

---

## Installation

### Linux / macOS

```bash
curl -sSL https://apinox.denisetiya.site/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://apinox.denisetiya.site/install.ps1 | iex
```

### Manual

Download the latest binary from the [releases page](https://github.com/denisetiya/apinox/releases):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `apinox-linux-x86_64` |
| Linux ARM64 | `apinox-linux-aarch64` |
| macOS x86_64 | `apinox-macos-x86_64` |
| macOS ARM64 | `apinox-macos-aarch64` |
| Windows x86_64 | `apinox-windows-x86_64.exe` |

Place the binary in your `$PATH` and make it executable:

```bash
chmod +x apinox-*
sudo mv apinox-linux-x86_64 /usr/local/bin/apinox
```

Verify:

```bash
apinox --version
apinox --help
```

---

## Quick Start

1. Create a schema file `api-spec.yml`:

```yaml
apinox: "1.0"
name: my-api
version: 1.0.0
description: My first API
base_url: https://api.example.com/v1

auth:
  default: bearer
  schemes:
    - id: bearer
      type: http
      scheme: bearer
      header: Authorization
      prefix: "Bearer "

groups:
  - id: users
    name: Users
    description: User management

endpoints:
  - id: list-users
    name: List Users
    group: users
    method: GET
    path: /users
    query_params:
      - name: page
        type: integer
        required: false
        description: Page number
    responses:
      - status: 200
        description: Success
        examples:
          - name: default
            value:
              - id: "usr_1"
                name: "John Doe"
                email: "john@example.com"
```

2. Validate:

```bash
apinox validate api-spec.yml
```

3. Generate all docs:

```bash
apinox build api-spec.yml -o ./dist
```

4. See the output:

```
dist/
├── docs/
│   ├── my-api.md
│   └── my-api.scalar.html
├── hurl/
│   ├── my-api.hurl
│   └── my-api.sh
├── insomnia/
│   └── my-api.insomnia.json
├── openapi/
│   └── my-api.openapi.yaml
└── postman/
    └── my-api.postman_collection.json
```

---

## Schema Format

### Root Fields

```yaml
apinox: "1.0"                  # Schema spec version (required)
name: my-api                   # API name (required)
version: 1.0.0                 # API version (required)
description: My API description # Optional
base_url: https://api.example.com/v1  # Optional base URL
```

### Auth

```yaml
auth:
  default: bearer       # Default auth scheme for all endpoints
  schemes:
    - id: bearer
      type: http        # http | apiKey | basic | oauth2
      scheme: bearer    # bearer | basic | digest
      header: Authorization
      prefix: "Bearer "
      description: JWT Bearer token
    - id: api_key
      type: apiKey
      key: X-API-Key
      in_location: header  # header | query
      description: API key authentication
```

Endpoints can override auth per-endpoint:
```yaml
- id: public-endpoint
  method: GET
  path: /public
  auth: ~              # null = no auth
```

### Environments

```yaml
environments:
  - name: production
    base_url: https://api.example.com/v1
    vars:
      TOKEN: prod_token
  - name: development
    base_url: http://localhost:8080
    vars:
      TOKEN: dev_token
```

### Groups

```yaml
groups:
  - id: users
    name: Users
    description: User management
    auth: bearer         # Optional group-level auth override
  - id: payments
    name: Payments
    description: Payment processing
```

### Endpoints

Full endpoint definition:

```yaml
endpoints:
  - id: create-user
    name: Create User
    group: users                    # References group id
    method: POST
    path: /users
    description: |
      Create a new user account.
      Returns the created user object.
    deprecated: false                # Optional
    tags: [users, admin]             # Optional tags
    auth: bearer                     # Auth override (null = no auth)
    servers:                         # Optional per-endpoint server URL
      production: https://api.example.com/v1
      development: http://localhost:8080
    rate_limit:                      # Optional rate limit info
      requests: 100
      window: 1m

    # Path parameters
    path_params:
      - name: user_id
        type: string
        required: true
        pattern: "^usr_[a-zA-Z0-9]+$"
        description: User ID
        example: usr_abc123

    # Query parameters
    query_params:
      - name: page
        type: integer
        required: false
        min: 1
        max: 100
        default: 1
        description: Page number
      - name: sort
        type: string
        required: false
        enum: [asc, desc]
        default: asc
        description: Sort order

    # Request headers
    headers:
      - name: Idempotency-Key
        value: ~
        required: false
        description: Prevent duplicate requests
      - name: Content-Type
        value: application/json
        required: true

    # Request body
    body:
      type: json                      # json | formdata | urlencoded | binary | raw
      required: true
      description: User creation payload
      content_type: application/json   # Optional override
      schema:                          # For JSON/urlencoded
        name:
          type: string
          required: true
          min_length: 2
          max_length: 100
          description: Full name
        email:
          type: email
          required: true
          description: Email address
        role:
          type: string
          required: false
          enum: [user, admin]
          default: user
          description: User role
      examples:
        - name: create_admin
          description: Create an admin user
          value:
            name: Jane Doe
            email: jane@example.com
            role: admin

    # Responses
    responses:
      - status: 201
        description: User created
        headers:
          - name: Location
            value: /users/usr_abc123
        content_type: application/json
        examples:
          - name: success
            value:
              id: usr_abc123
              name: Jane Doe
              email: jane@example.com
              role: admin
              created_at: "2026-06-18T10:00:00Z"
      - status: 400
        description: Validation error
        examples:
          - name: validation_error
            value:
              error: validation_error
              message: "email: Must be a valid email address"
      - status: 401
        description: Unauthorized
        examples:
          - name: unauthorized
            value:
              error: unauthorized
              message: Invalid or expired token
```

**Form-data body** (file uploads):

```yaml
body:
  type: formdata
  fields:
    - name: avatar
      type: file
      required: true
      accept: [image/jpeg, image/png, image/webp]
      max_size_mb: 5
      description: Avatar image
    - name: crop
      type: string
      default: square
      enum: [square, circle, original]
      description: Crop mode
```

### Reusable Error Patterns

Define common error responses once and reference them:

```yaml
error_responses:
  patterns:
    - id: unauthorized
      status: 401
      description: Missing or invalid auth token
      example:
        error: unauthorized
        message: Authentication required
    - id: not_found
      status: 404
      description: Resource not found
      example:
        error: not_found
        message: Resource not found

endpoints:
  - id: get-user
    method: GET
    path: /users/{user_id}
    responses:
      - status: 200
        description: Success
        examples:
          - name: default
            value:
              id: usr_abc123
              name: John Doe
      - status: 401
        use_pattern: unauthorized       # Reuse error pattern
      - status: 404
        use_pattern: not_found         # Reuse error pattern
```

### Schema Includes (Modular)

Split large schemas across multiple files:

```yaml
# main.yml
name: my-api
version: 2.0.0
apinox: "1.0"

includes:
  - path: ./includes/auth.yml
  - path: ./includes/users.yml
    prefix: /v2          # Prefix all endpoint paths with /v2
```

```yaml
# includes/users.yml
endpoints:
  - id: get-user
    name: Get User
    method: GET
    path: /users/{user_id}
    responses:
      - status: 200
        description: User found
        examples:
          - name: default
            value:
              id: usr_1
              name: John Doe
```

---

## CLI Commands

### `validate`

Validate schema syntax, references, and detect issues.

```bash
# Text output (default)
apinox validate api-spec.yml

# JSON output (for CI integration)
apinox validate api-spec.yml --format json
```

**Validates:**
- Required fields: `name`, `version`, `apinox`
- Duplicate endpoint IDs, group IDs, response example names
- Auth scheme references (default + per-endpoint)
- Group references on endpoints
- Path parameter declarations match actual path templates
- HTTP method validity (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- Response examples exist
- Unused auth schemes (warning)
- Groups without endpoints (info)

### `build`

Generate output files from schema.

```bash
# Generate all formats
apinox build api-spec.yml

# Custom output directory
apinox build api-spec.yml -o ./docs/dist

# Single format
apinox build api-spec.yml -f postman
apinox build api-spec.yml -f openapi
apinox build api-spec.yml -f markdown
apinox build api-spec.yml -f scalar
apinox build api-spec.yml -f insomnia
apinox build api-spec.yml -f hurl

# Skip validation (build even with warnings)
apinox build api-spec.yml --skip-validate
```

### `watch`

Watch schema file and auto-rebuild on changes.

```bash
apinox watch api-spec.yml -o ./dist
```

Uses filesystem notifications via the `notify` crate. Rebuilds with a 300ms debounce.

### `diff`

Show changes between two schema versions. Requires `changelog` entries in the schema.

```yaml
changelog:
  - version: 2.0.0
    date: 2026-06-18
    changes:
      - endpoint: list-users
        description: Added cursor pagination
      - endpoint: create-user
        description: Added role field
        from_type: user
        to_type: admin
      - endpoint: delete-user
        description: Deprecated
```

```bash
apinox diff api-spec.yml 1.0.0 2.0.0
apinox diff api-spec.yml 1.0.0 2.0.0 --format text
```

### `migrate`

Generate migration guide between two schema files (schema diff mode — compares actual schemas, not changelog entries).

```bash
apinox migrate api-v1.yml api-v2.yml
apinox migrate api-v1.yml api-v2.yml -o migration-guide.md
```

### `import`

Import OpenAPI 3.x or Swagger 2.0 spec to Apinox schema.

```bash
# Output to file
apinox import swagger.json -o api-spec.yml

# Output to stdout
apinox import openapi.yaml
```

### `sync`

Push Postman collection directly to a Postman workspace.

```bash
# Using CLI args
apinox sync api-spec.yml \
  --postman-key pm_api_key_xxx \
  --workspace wkspc_xxx \
  --collection-name "My API"

# Update existing collection
apinox sync api-spec.yml \
  --postman-key pm_api_key_xxx \
  --workspace wkspc_xxx \
  --collection-id clctn_xxx

# Using .apinox.toml config
apinox sync api-spec.yml
```

Config file (`.apinox.toml`):

```toml
[sync.postman]
api_key = "pm_api_key_xxx"
workspace_id = "wkspc_xxx"
collection_id = "clctn_xxx"
```

---

## Output Formats

### Postman Collection

Interactive API collection ready for Postman:

```bash
apinox build api-spec.yml -f postman
# Output: ./dist/postman/<name>.postman_collection.json
```

Includes:
- All endpoints with methods and paths
- Path/query/header parameters
- Request body schemas and examples
- Response examples
- Auth configuration
- Environment variables

### OpenAPI 3.1

Standard OpenAPI specification:

```bash
apinox build api-spec.yml -f openapi
# Output: ./dist/openapi/<name>.openapi.yaml
```

Compatible with Swagger UI, Redoc, Stoplight, and any OpenAPI 3.1 tool.

### Markdown Documentation

Human-readable API documentation:

```bash
apinox build api-spec.yml -f markdown
# Output: ./dist/docs/<name>.md
```

Clean markdown with endpoint groups, request/response schemas, and examples.

### Scalar Interactive Docs

Beautiful interactive API reference using [Scalar](https://scalar.com):

```bash
apinox build api-spec.yml -f scalar
# Output: ./dist/docs/<name>.scalar.html
```

Self-contained HTML file with interactive testing UI.

### Insomnia Import

Import into Insomnia REST Client:

```bash
apinox build api-spec.yml -f insomnia
# Output: ./dist/insomnia/<name>.insomnia.json
```

### Hurl Test Scripts

Runnable test scripts using [Hurl](https://hurl.dev):

```bash
apinox build api-spec.yml -f hurl
# Output: ./dist/hurl/<name>.hurl + ./dist/hurl/<name>.sh
```

```bash
# Run tests
hurl --test ./dist/hurl/my-api.hurl

# Or use the shell script (curl-based)
bash ./dist/hurl/my-api.sh
```

---

## CI/CD Integration

### GitHub Actions Release

Tag a version to auto-build and release binaries for all platforms:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The `.github/workflows/release.yml` workflow:
1. Builds for Linux (x86_64 + ARM64), macOS (x86_64 + ARM64), Windows (x86_64)
2. Creates a GitHub Release with auto-generated release notes
3. Attaches all platform binaries

### CI Validation

```yaml
# .github/workflows/validate.yml
name: Validate API Schema

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Download apinox
        run: |
          curl -sL https://github.com/denisetiya/apinox/releases/latest/download/apinox-linux-x86_64 -o apinox
          chmod +x apinox
      - name: Validate schema
        run: ./apinox validate docs/api-spec.yml --format json
```

---

## Changelog & Diff

Track API changes over time directly in the schema:

```yaml
changelog:
  - version: 2.0.0
    date: 2026-06-18
    changes:
      - endpoint: list-users
        description: Added cursor pagination with nextCursor and hasMore fields
      - endpoint: get-user
        description: Response now includes role and permissions

  - version: 1.0.0
    date: 2026-06-01
    changes:
      - endpoint: list-users
        description: Initial implementation
```

View changes:

```bash
apinox diff docs/api-spec.yml 1.0.0 2.0.0
```

Output:

```markdown
## Changes from 1.0.0 to 2.0.0

### Added
- **list-users**: Added cursor pagination with nextCursor and hasMore fields

### Changed
- **get-user**: Response now includes role and permissions
```

---

## Modular Schemas

For large APIs, split schemas into multiple files:

```yaml
# api-spec.yml
name: enterprise-api
version: 3.1.0
apinox: "1.0"
base_url: https://api.example.com/v3

auth:
  default: bearer
  schemes:
    - id: bearer
      type: http
      scheme: bearer
      header: Authorization
      prefix: "Bearer "

groups:
  - id: users
    name: Users
  - id: orders
    name: Orders
  - id: payments
    name: Payments

includes:
  - path: ./includes/users.yml
  - path: ./includes/orders.yml
    prefix: /v3
  - path: ./includes/payments.yml
```

**Available fields in included files:**
- `endpoints` — merged into main schema's endpoints
- `auth` scheme definitions (merge)
- `environments` (merge)

Output directory structure for includes following the `path` attribute (relative to parent schema).

---

## Postman Sync

Push generated Postman collections directly to your Postman workspace:

```bash
# One-time sync
apinox sync api-spec.yml \
  --postman-key PMAK-xxx \
  --workspace 1a2b3c4d-xxxx-xxxx-xxxx-xxxxxxxxxxxx

# Or configure .apinox.toml
echo '[sync.postman]
api_key = "PMAK-xxx"
workspace_id = "1a2b3c4d-xxxx-xxxx-xxxx-xxxxxxxxxxxx"' > .apinox.toml

apinox sync api-spec.yml
```

**Features:**
- Creates new collection or updates existing one
- Auto-finds collection by name
- Uploads complete collection with auth, examples, and environments

---

## Build from Source

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- Cross-compilation targets (optional):
  - `rustup target add aarch64-unknown-linux-gnu`
  - `rustup target add x86_64-pc-windows-gnu`
  - `rustup target add x86_64-apple-darwin`
  - `rustup target add aarch64-apple-darwin`

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Cross-compile (Linux x86_64 → ARM64)
cargo build --release --target aarch64-unknown-linux-gnu
```

The binary is at `target/release/apinox`.

### Build All Platforms

```bash
./build-all.sh
```

Requires cross-compilation toolchains installed. Output goes to `releases/`.

---

## Project Structure

```
apinox/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Module declarations
│   ├── schema/              # Schema data types
│   │   ├── mod.rs
│   │   ├── root.rs          # Root schema, groups, changelog
│   │   ├── endpoint.rs      # Endpoint, body, response definitions
│   │   ├── auth.rs          # Auth schemes
│   │   ├── environment.rs   # Environment definitions
│   │   └── types.rs         # Enums (HttpMethod, BodyType, ApinoxType)
│   ├── parser/              # YAML/JSON schema file parser
│   │   └── mod.rs
│   ├── validator/           # Schema validation logic
│   │   └── mod.rs
│   ├── generator/           # Output format generators
│   │   ├── mod.rs
│   │   ├── postman.rs       # Postman Collection v2.1
│   │   ├── openapi.rs       # OpenAPI 3.1 YAML
│   │   ├── markdown.rs      # Markdown docs
│   │   ├── scalar.rs        # Scalar HTML page
│   │   ├── insomnia.rs      # Insomnia v4 import
│   │   └── hurl.rs          # Hurl + curl scripts
│   ├── importer/            # OpenAPI/Swagger import
│   │   └── mod.rs
│   ├── diff/                # Schema diff & migration guide
│   │   └── mod.rs
│   └── sync/                # Postman API sync
│       ├── mod.rs
│       └── postman.rs
├── landing/                 # Landing page (apinox.denisetiya.site)
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── nginx.conf
│   └── public/
│       ├── index.html       # Marketing/landing page
│       ├── install.sh       # Linux/macOS installer script
│       └── install.ps1      # Windows installer script
├── test/                    # Test schemas and sample outputs
│   ├── api-spec.yml
│   ├── atomix-cloud-api.yml
│   ├── sample-openapi.yml
│   ├── modular-config.yml
│   └── imported-api.yml
├── .github/workflows/
│   └── release.yml          # GitHub Actions release workflow
├── Cargo.toml
├── build-all.sh
└── README.md
```

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run linter (`cargo clippy -- -D warnings`)
6. Commit using conventional commits (`feat:`, `fix:`, `docs:`, `chore:`)
7. Push and open a Pull Request

### Commit Convention

```
type(scope): description

feat(api): add cursor pagination support
fix(generator): handle empty response examples
docs(readme): add quick-start guide
chore(deps): upgrade clap to v4
```

---

## License

MIT License — see [LICENSE](LICENSE) for details.
