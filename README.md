<p align="center">
  <svg viewBox="0 0 26 26" width="52" height="52" fill="none">
    <rect width="26" height="26" rx="5" fill="#3b82f6"/>
    <path d="M6 8.5h14M6 13h9M6 17.5h6" stroke="#fff" stroke-width="2" stroke-linecap="round"/>
    <circle cx="20" cy="17.5" r="2.5" fill="#22c55e"/>
  </svg>
</p>

<h1 align="center">Apinox</h1>
<p align="center">
  <strong>Schema-first API Documentation Generator</strong><br>
  Define once in YAML/JSON. Generate <strong>7 formats</strong> from a single source of truth.
</p>

<p align="center">
  <a href="https://github.com/denisetiya/apinox/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/denisetiya/apinox/release.yml?branch=master&label=CI&logo=github" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/denisetiya/apinox?color=blue" alt="License"></a>
  <a href="https://github.com/denisetiya/apinox/releases"><img src="https://img.shields.io/github/v/release/denisetiya/apinox?logo=rust" alt="Release"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.70%2B-f74c00?logo=rust" alt="Rust"></a>
  <a href="https://apinox.denisetiya.site/docs/"><img src="https://img.shields.io/badge/docs-apinox.denisetiya.site-3b82f6" alt="Docs"></a>
</p>

```text
┌─────────────┐     ┌──────────────┐     ┌──────────────────────────────────────────┐
│  Schema     │     │  apinox      │     │  dist/                                   │
│  (YAML/JSON)│ ──→ │  validate +  │ ──→ │  ├── postman/   (.postman_collection.json)│
│             │     │  build       │     │  ├── openapi/   (.openapi.yaml)           │
│  groups     │     │  watch       │     │  ├── docs/      (.md + .scalar.html)     │
│  endpoints  │     │  diff        │     │  ├── insomnia/  (.insomnia.json)          │
│  auth/envs  │     │  migrate     │     │  └── hurl/      (.hurl + .sh)            │
│  examples   │     │  import      │     └──────────────────────────────────────────┘
└─────────────┘     │  sync        │
                    └──────────────┘
```

---

## Why Apinox?

API documentation should come from **one source of truth**, not five hand-maintained files that drift apart over time.

- **Stop duplicating** — write your API shape once, export everywhere
- **No context switching** — Postman, OpenAPI, Markdown, Insomnia, Hurl from a single `apinox build`
- **Validation built-in** — catch broken references, missing fields, typos *before* they reach your team
- **CI-ready** — validate schemas in CI, auto-generate docs on release

---

## Quick Start (30 seconds)

```bash
# 1. Install
curl -sSL https://apinox.denisetiya.site/install.sh | bash

# 2. Create a schema
cat > api-spec.yml << 'EOF'
apinox: "1.0"
name: my-api
version: 1.0.0
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
endpoints:
  - id: list-users
    name: List Users
    group: users
    method: GET
    path: /users
    responses:
      - status: 200
        description: Success
        examples:
          - name: default
            value: [{id: "usr_1", name: "John Doe"}]
EOF

# 3. Validate
apinox validate api-spec.yml

# 4. Generate all 7 formats
apinox build api-spec.yml -o ./dist
```

Done. Your docs are in `dist/`.

---

## Installation

| Platform | Command |
|----------|---------|
| **Linux / macOS** | `curl -sSL https://apinox.denisetiya.site/install.sh \| bash` |
| **Windows** | `irm https://apinox.denisetiya.site/install.ps1 \| iex` |
| **Cargo** | `cargo install apinox` |

Or download the binary from [GitHub Releases](https://github.com/denisetiya/apinox/releases) and place it in your `$PATH`.

---

## Full Documentation

📖 **Detailed docs, schema reference, CLI reference, guides, and FAQ** → **[apinox.denisetiya.site/docs/](https://apinox.denisetiya.site/docs/)**

---

## Schema Format

A single YAML file describing your entire API:

```yaml
apinox: "1.0"                 # Schema spec version (required)
name: my-api                   # API name (required)
version: 1.0.0                 # API version (required)
description: My API            # Optional
base_url: https://api.example.com/v1

# ── Authentication ──────────────────────────
auth:
  default: bearer
  schemes:
    - id: bearer
      type: http;    scheme: bearer
      header: Authorization;  prefix: "Bearer "
    - id: api_key
      type: apiKey;  key: X-API-Key
      in_location: header

# ── Environments ────────────────────────────
environments:
  - name: production
    base_url: https://api.example.com/v1
    vars: { TOKEN: "prod_xxx" }
  - name: development
    base_url: http://localhost:8080
    vars: { TOKEN: "dev_xxx" }

# ── Groups ──────────────────────────────────
groups:
  - id: users;     name: Users;     description: User management
  - id: payments;  name: Payments;  description: Payment processing

# ── Endpoints ───────────────────────────────
endpoints:
  - id: create-user
    name: Create User
    group: users
    method: POST
    path: /users
    description: Create a new user account
    auth: bearer                    # Override default auth (~ for no-auth)
    tags: [users, admin]

    # Path params: {user_id} in the path →
    path_params:
      - name: user_id
        type: string;  required: true
        pattern: "^usr_[a-zA-Z0-9]+$"
        example: usr_abc123

    # Query params
    query_params:
      - name: page;  type: integer;  default: 1;  min: 1;  max: 100
      - name: sort;  type: string;   enum: [asc, desc];  default: asc

    # Request body
    body:
      type: json
      required: true
      schema:
        name:     { type: string, required: true,  min_length: 2,  max_length: 100 }
        email:    { type: email,  required: true }
        password: { type: string, required: true,  sensitive: true,  min_length: 8 }
        role:     { type: string, required: false, enum: [user, admin],  default: user }
      examples:
        - name: create_admin
          value: { name: "Jane Doe",  email: "jane@example.com",  password: "secret",  role: admin }

    # Responses
    responses:
      - status: 201
        description: User created
        examples:
          - name: success
            value: { id: "usr_abc123",  name: "Jane Doe",  email: "jane@example.com",  role: admin }
      - status: 400
        description: Validation error
        use_pattern: validation_error      # ← Reusable error pattern
      - status: 401
        use_pattern: unauthorized

# ── Reusable Error Patterns ────────────────
error_responses:
  patterns:
    - id: unauthorized
      status: 401
      description: Missing or invalid auth token
      example: { error: unauthorized,  message: Authentication required }
    - id: validation_error
      status: 400
      description: Validation failure
      example: { error: validation_error,  message: "email: must be valid" }

# ── Changelog ──────────────────────────────
changelog:
  - version: 2.0.0
    date: 2026-06-18
    changes:
      - endpoint: create-user
        description: Added role field with admin/user enum
  - version: 1.0.0
    date: 2026-06-01
    changes:
      - endpoint: create-user
        description: Initial implementation
```

> ⚠️ **Important:** Response examples must use `examples: [{name: ..., value: ...}]` format. Do **not** use `body:` — it produces empty responses in generated output.

### Field Schema Types

| Type | Description | Validators |
|------|-------------|------------|
| `string` | Text value | `min_length`, `max_length`, `pattern`, `enum` |
| `integer` | Whole number | `min`, `max` |
| `float` | Decimal number | `min`, `max` |
| `boolean` | True/false | — |
| `email` | Email (auto-formatted) | — |
| `date` / `dateTime` | ISO 8601 | `format` |
| `uuid` | UUID v4 (auto-generates) | — |
| `file` | Binary (form-data only) | `accept`, `max_size_mb` |

### Body Types

| Type | Use Case |
|------|----------|
| `json` | REST API payloads — define `schema` map |
| `formdata` | File uploads — define `fields` list |
| `urlencoded` | HTML forms — same as `json` |
| `binary` | Raw file — `mime_type` + `encoding` |
| `raw` | Custom content type — `content_type` override |

### Modular Includes

Split large APIs across files:

```yaml
# main.yml
name: enterprise-api
version: 3.0.0
apinox: "1.0"

includes:
  - path: ./includes/auth.yml
  - path: ./includes/users.yml
    prefix: /v3          # Prefix all endpoint paths with /v3
  - path: ./includes/orders.yml
```

```yaml
# includes/users.yml
endpoints:
  - id: get-user
    name: Get User
    method: GET
    path: /users/{user_id}
    responses:
      - status: 200;  description: User found
        examples:
          - name: default
            value: { id: "usr_1",  name: "John Doe" }
```

---

## CLI Commands

### `validate` — Check schema for errors

```bash
apinox validate api-spec.yml              # Text output
apinox validate api-spec.yml --format json  # CI-friendly JSON
```

Checks: required fields, duplicate IDs, broken auth/group refs, path-param mismatches, invalid methods, missing examples → exit code `0`=valid, `1`=errors.

### `build` — Generate output formats

```bash
apinox build api-spec.yml                         # All 7 formats → ./dist/
apinox build api-spec.yml -o ./docs               # Custom output dir
apinox build api-spec.yml -f postman              # Single format
apinox build api-spec.yml --skip-validate          # Skip validation
```

Format values: `postman`, `openapi`, `markdown`, `scalar`, `insomnia`, `hurl`, `all` (default).

### `watch` — Auto-rebuild on changes

```bash
apinox watch api-spec.yml -o ./dist
```

Filesystem notifications with 300ms debounce.

### `diff` — Show version changes

```bash
apinox diff api-spec.yml 1.0.0 2.0.0
apinox diff api-spec.yml 1.0.0 2.0.0 --format text
```

Uses `changelog` entries in the schema → generates human-readable diff.

### `migrate` — Compare two schema files

```bash
apinox migrate api-v1.yml api-v2.yml
apinox migrate api-v1.yml api-v2.yml -o migration-guide.md
```

Compares actual schema contents (not changelog entries).

### `import` — Convert OpenAPI to Apinox schema

```bash
apinox import swagger.json -o api-spec.yml
apinox import openapi.yaml                     # Output to stdout
```

Supports OpenAPI 3.x and Swagger 2.0.

### `sync` — Push Postman to workspace

```bash
apinox sync api-spec.yml \
  --postman-key PMAK-xxx \
  --workspace wkspc_xxx \
  --collection-name "My API"
```

Or configure `.apinox.toml`:

```toml
[sync.postman]
api_key = "PMAK-xxx"
workspace_id = "1a2b3c4d-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
collection_id = "clctn_xxx"
```

---

## Output Formats

| Format | File | Import Into |
|--------|------|-------------|
| **Postman** | `<name>.postman_collection.json` | Postman app |
| **OpenAPI 3.1** | `<name>.openapi.yaml` | Swagger UI, Redoc, Stoplight |
| **Markdown** | `<name>.md` | Git repos, wikis, docs sites |
| **Scalar** | `<name>.scalar.html` | Browser (interactive, self-contained) |
| **Insomnia** | `<name>.insomnia.json` | Insomnia REST Client |
| **Hurl** | `<name>.hurl` + `<name>.sh` | `hurl --test`, CI pipelines |
| **curl** | `<name>.sh` | Any shell |

### Build output example

```
$ apinox build api-spec.yml -o dist/

  Validated 14 endpoints, 6 groups, 2 auth schemes
  0 errors, 2 warnings (missing examples)

  Generated 7 formats to dist/:

  dist/
  ├── postman/     my-api.postman_collection.json     (18KB)
  ├── openapi/     my-api.openapi.yaml                 (22KB)
  ├── docs/        my-api.md                           (10KB)
  ├── docs/        my-api.scalar.html                  (34KB)
  ├── insomnia/    my-api.insomnia.json                (11KB)
  └── hurl/        my-api.hurl + .sh                    (8KB)

  Done in 12ms
```

---

## Features

| Feature | Description |
|---------|-------------|
| 🔗 **Single Source of Truth** | Define once, generate 7 formats |
| ✅ **Validation** | Catch missing fields, duplicates, broken refs before build |
| 👁 **File Watching** | `watch` auto-rebuilds on schema changes |
| 📥 **OpenAPI Import** | Convert existing OpenAPI/Swagger to Apinox schema |
| 📊 **Schema Diff** | See breaking changes between versions |
| 📋 **Migration Guide** | Generate version-to-version migration docs |
| ☁️ **Postman Sync** | Push collections directly to Postman workspace |
| 📦 **Modular Includes** | Split large APIs into multiple files |
| 📝 **Changelog** | Track endpoint-level changes in the schema |
| 🔐 **Auth Schemes** | Bearer, API Key, Basic, OAuth2 — per-endpoint overrides |
| 🧪 **Faker Fixtures** | Auto-generated Postman dynamic variables |
| 🛡️ **Sensitive Fields** | Mark passwords/tokens — hidden in Postman, redacted from docs |
| 🚀 **CI Ready** | Validate and build in GitHub Actions |

---

## CI/CD Integration

Validate your schema on every push:

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
      - name: Validate
        run: ./apinox validate docs/api-spec.yml --format json
```

Tag a release to auto-build binaries for all platforms:

```bash
git tag v1.0.0 && git push origin v1.0.0
```

The [release workflow](.github/workflows/release.yml) builds for Linux, macOS, Windows (x86_64 + ARM64) and creates a GitHub Release with all binaries.

---

## Build from Source

```bash
git clone https://github.com/denisetiya/apinox.git
cd apinox

# Debug
cargo build

# Release
cargo build --release

# Binary at target/release/apinox
```

**Prerequisites:** Rust 1.70+

### Cross-compile

```bash
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

Or use `./build-all.sh` for all platforms (output → `releases/`).

---

## Project Structure

```
apinox/
├── src/
│   ├── main.rs              # CLI entry (clap)
│   ├── lib.rs               # Module declarations
│   ├── schema/              # Schema data types
│   │   ├── root.rs          # Root schema, groups, changelog
│   │   ├── endpoint.rs      # Endpoint, body, response definitions
│   │   ├── auth.rs          # Auth schemes
│   │   ├── environment.rs   # Environment definitions
│   │   └── types.rs         # Enums (HttpMethod, BodyType, ApinoxType)
│   ├── parser/              # YAML/JSON file parser
│   ├── validator/           # Schema validation logic
│   ├── generator/           # Output format generators
│   │   ├── postman.rs       # Postman Collection v2.1
│   │   ├── openapi.rs       # OpenAPI 3.1 YAML
│   │   ├── markdown.rs      # Markdown docs
│   │   ├── scalar.rs        # Scalar HTML page
│   │   ├── insomnia.rs      # Insomnia v4 import
│   │   └── hurl.rs          # Hurl + curl scripts
│   ├── importer/            # OpenAPI/Swagger import
│   ├── diff/                # Schema diff & migration guide
│   └── sync/                # Postman API sync
│       └── postman.rs
├── landing/                 # Static site (apinox.denisetiya.site)
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── nginx.conf
│   └── public/
│       ├── index.html
│       ├── docs/            # Full documentation page
│       ├── install.sh
│       └── install.ps1
├── test/                    # Sample schemas & test data
├── .github/workflows/
│   └── release.yml
├── Cargo.toml
└── build-all.sh
```

---

## Contributing

1. Fork → create branch (`feat/my-feature`)
2. `cargo test` + `cargo clippy -- -D warnings`
3. Commit with [conventional commits](https://www.conventionalcommits.org/):
   - `feat:` new feature
   - `fix:` bug fix
   - `docs:` documentation
   - `chore:` maintenance
4. Open a Pull Request

---

## License

MIT — see [LICENSE](LICENSE).
