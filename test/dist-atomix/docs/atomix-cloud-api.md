# atomix-cloud-api API Documentation

**Version:** `1.0.0`

Atomix Cloud — self-hosted PaaS platform API (GitHub OAuth, projects, deployments, databases, domains)

**Base URL:** `https://cloud.denisetiya.site/api`

## Environments

- **production** — `https://cloud.denisetiya.site/api` — Production server
- **local** — `http://localhost:8080` — Local development

## Authentication

**Default scheme:** `jwt`

### jwt (HTTP BEARER)

JWT token from GitHub OAuth login

**Header:** `Authorization`
**Prefix:** `Bearer `

## Table of Contents

- [Authentication (5 endpoints)](#authentication)
- [Projects (5 endpoints)](#projects)
- [Deployments (4 endpoints)](#deployments)

## Authentication

GitHub OAuth login & user session

### 🟢 `/auth/github` 

**Initiate GitHub OAuth** — `GET`

Redirect to GitHub OAuth consent screen. Returns 302 to github.com/login/oauth/authorize.

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/auth/github'
```

#### Responses

##### 🔀 302 `Redirect to GitHub`

**Response Headers:**

- `Location`: `*dynamic*`

---

### 🟢 `/auth/github/callback` 

**GitHub OAuth Callback** — `GET`

GitHub redirects here with ?code=xxx. Exchanges code for token, creates/updates user, returns JWT.

#### Query Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `code` | `string` | ✅ | OAuth authorization code from GitHub |
| `state` | `string` | ❌ | CSRF state parameter |

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/auth/github/callback?code=example-code&state=example-state'
```

#### Responses

##### ✅ 200 `Login successful`

*success*

```json
{
  "token": "eyJhbG...NiIs...",
  "user": {
    "avatar_url": "https://avatars.githubusercontent.com/u/12345",
    "created_at": "2026-01-15T08:30:00Z",
    "email": "deni@example.com",
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "username": "denisetiya"
  }
}
```

##### ⚠️ 401 `Authentication required`

##### ❌ 500 `Internal server error`

---

### 🟢 `/auth/me` 

**Get Current User** — `GET`

Get authenticated user profile from JWT token.

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/auth/me'
```

#### Responses

##### ✅ 200 `User profile`

*success*

```json
{
  "avatar_url": "https://avatars.githubusercontent.com/u/12345",
  "created_at": "2026-01-15T08:30:00Z",
  "email": "deni@example.com",
  "github_id": 12345678,
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "denisetiya"
}
```

##### ⚠️ 401 `Authentication required`

---

### 🔵 `/auth/logout` 

**Logout** — `POST`

Invalidate current JWT session.

#### Example cURL

```bash
curl -X POST 'https://cloud.denisetiya.site/api/auth/logout'
```

#### Responses

##### ✅ 200 `Logged out`

---

### 🟢 `/auth/github/repos` 

**List User GitHub Repos** — `GET`

List authenticated user's GitHub repositories (via GitHub API with user's token).

#### Query Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `page` | `integer` | ✅ | — |
| `per_page` | `integer` | ✅ | — |
| `sort` | `string` | ✅ | — |

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/auth/github/repos?page=1&per_page=1&sort=example-sort'
```

#### Responses

##### ✅ 200 `List of repos`

*sample*

```json
[
  {
    "clone_url": "https://github.com/denisetiya/atomix-cloud.git",
    "default_branch": "main",
    "description": "Self-hosted PaaS",
    "full_name": "denisetiya/atomix-cloud",
    "language": "Rust",
    "name": "atomix-cloud",
    "private": false
  },
  {
    "clone_url": "https://github.com/denisetiya/my-astro-blog.git",
    "default_branch": "main",
    "description": null,
    "full_name": "denisetiya/my-astro-blog",
    "language": "TypeScript",
    "name": "my-astro-blog",
    "private": false
  }
]
```

##### ⚠️ 401 `Authentication required`

---

## Projects

Repository project management

### 🟢 `/projects` 

**List Projects** — `GET`

List all projects for authenticated user.

#### Query Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `page` | `integer` | ✅ | — |
| `per_page` | `integer` | ✅ | — |

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/projects?page=1&per_page=1'
```

#### Responses

##### ✅ 200 `Paginated project list`

*default*

```json
{
  "page": 1,
  "per_page": 20,
  "projects": [
    {
      "created_at": "2026-03-01T10:00:00Z",
      "framework": "axum",
      "id": "proj_a1b2c3",
      "language": "rust",
      "name": "atomix-cloud",
      "port": 8080,
      "repo_branch": "main",
      "repo_url": "https://github.com/denisetiya/atomix-cloud.git",
      "status": "running",
      "updated_at": "2026-06-15T14:30:00Z"
    }
  ],
  "total": 1
}
```

##### ⚠️ 401 `Authentication required`

---

### 🔵 `/projects` 

**Create Project** — `POST`

Create a new project from a GitHub repository. Clones repo, detects framework, triggers first deploy.

#### Request Body

**Content-Type:** `json`
**Required:** ✅

**Schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `build_command` | `string` | ❌ | Custom build command |
| `start_command` | `string` | ❌ | Custom start command |
| `name` | `string` | ✅ | Project display name |
| `repo_url` | `string` | ✅ | Git clone URL |
| `repo_branch` | `string` | ❌ | Branch to deploy |
| `framework` | `string` | ❌ | Override auto-detected framework (axum, astro, nextjs, express, flask, etc) |
| `port` | `integer` | ❌ | Container internal port (auto-detected if omitted) |

**Examples:**

*rust-project*

```json
{
  "name": "atomix-cloud",
  "repo_branch": "main",
  "repo_url": "https://github.com/denisetiya/atomix-cloud.git"
}
```

*node-project*

```json
{
  "framework": "nextjs",
  "name": "my-nextjs-app",
  "port": 3000,
  "repo_branch": "main",
  "repo_url": "https://github.com/denisetiya/my-nextjs-app.git"
}
```

#### Example cURL

```bash
curl -X POST 'https://cloud.denisetiya.site/api/projects' \
  -d '{
     "name": "atomix-cloud",
     "repo_branch": "main",
     "repo_url": "https://github.com/denisetiya/atomix-cloud.git"
   }'
```

#### Responses

##### ✅ 201 `Project created`

*created*

```json
{
  "created_at": "2026-06-17T12:00:00Z",
  "framework": "axum",
  "id": "proj_x9y8z7",
  "language": "rust",
  "name": "atomix-cloud",
  "port": 8080,
  "repo_branch": "main",
  "repo_url": "https://github.com/denisetiya/atomix-cloud.git",
  "status": "pending",
  "updated_at": "2026-06-17T12:00:00Z"
}
```

##### ⚠️ 401 `Authentication required`

##### ⚠️ 409 `Project with same name already exists`

---

### 🟢 `/projects/{id}` 

**Get Project** — `GET`

Get project details by ID.

#### Path Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `string` | ✅ | Project ID (e.g. proj_a1b2c3) |

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/projects/id-example'
```

#### Responses

##### ✅ 200 `Project details`

##### ⚠️ 404 `Resource not found`

---

### 🟠 `/projects/{id}` 

**Update Project** — `PUT`

Update project settings. Triggers new deployment if build/start commands changed.

#### Path Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `string` | ✅ | — |

#### Request Body

**Content-Type:** `json`
**Required:** ✅

**Schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | ❌ | — |
| `framework` | `string` | ❌ | — |
| `build_command` | `string` | ❌ | — |
| `repo_branch` | `string` | ❌ | — |
| `port` | `integer` | ❌ | — |
| `start_command` | `string` | ❌ | — |

#### Example cURL

```bash
curl -X PUT 'https://cloud.denisetiya.site/api/projects/id-example' \
  -d '{}'
```

#### Responses

##### ✅ 200 `Updated project`

##### ⚠️ 404 `Resource not found`

---

### 🔴 `/projects/{id}` 

**Delete Project** — `DELETE`

Delete project and all associated resources (containers, env vars, domains, databases).

#### Path Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `string` | ✅ | — |

#### Example cURL

```bash
curl -X DELETE 'https://cloud.denisetiya.site/api/projects/id-example'
```

#### Responses

##### ✅ 200 `Deleted`

##### ⚠️ 404 `Resource not found`

---

## Deployments

Build, deploy, rollback, logs

### 🔵 `/projects/{id}/deploy` 

**Trigger Deployment** — `POST`

Trigger a new deployment. Pulls latest code, builds, and deploys.

#### Path Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `string` | ✅ | — |

#### Example cURL

```bash
curl -X POST 'https://cloud.denisetiya.site/api/projects/id-example/deploy'
```

#### Responses

##### ✅ 201 `Deployment triggered`

*triggered*

```json
{
  "branch": "main",
  "build_logs": "",
  "commit_message": "feat: add database provisioning",
  "commit_sha": "abc1234",
  "created_at": "2026-06-17T12:05:00Z",
  "deploy_logs": "",
  "finished_at": null,
  "id": "dpl_m1n2o3",
  "project_id": "proj_a1b2c3",
  "status": "building"
}
```

##### ⚠️ 404 `Resource not found`

##### ⚠️ 429 `Rate limit exceeded`

---

### 🟢 `/projects/{id}/deployments` 

**List Deployments** — `GET`

List all deployments for a project, newest first.

#### Path Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `string` | ✅ | — |

#### Query Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `page` | `integer` | ✅ | — |
| `per_page` | `integer` | ✅ | — |

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/projects/id-example/deployments?page=1&per_page=1'
```

#### Responses

##### ✅ 200 `Deployment list`

##### ⚠️ 404 `Resource not found`

---

### 🟢 `/deployments/{id}` 

**Get Deployment** — `GET`

Get single deployment details.

#### Path Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `string` | ✅ | — |

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/deployments/id-example'
```

#### Responses

##### ✅ 200 `Deployment details`

##### ⚠️ 404 `Resource not found`

---

### 🟢 `/deployments/{id}/logs` 

**Get Deployment Logs** — `GET`

Get build and deploy logs for a deployment.

#### Path Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | `string` | ✅ | — |

#### Example cURL

```bash
curl -X GET 'https://cloud.denisetiya.site/api/deployments/id-example/logs'
```

#### Responses

##### ✅ 200 `Logs`

*success*

```json
{
  "build_logs": "[build] Pulling latest commits...\n[build] Detected framework: axum (Rust)\n[build] cargo build --release\n[build]    Compiling atomix-cloud v0.1.0\n[build]     Finished release [optimized] in 42.3s\n[build] Build complete (43.1s)\n",
  "deploy_logs": ""
}
```

---

