# Pet Store API API Documentation

**Version:** `1.0.0`

A sample pet store API

**Base URL:** `https://petstore.example.com/v1`

## Environments

- **Production** — `https://petstore.example.com/v1` — Production

## Authentication

**Default scheme:** `bearerAuth`

### bearerAuth (HTTP BEARER)

**Header:** `Authorization`
**Prefix:** `Bearer `

## Table of Contents

- [pets (2 endpoints)](#pets)

## pets

Pet operations

### 🟢 `/pets` 

**List all pets** — `GET`

**Tags:** `pets`

#### Query Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `limit` | `integer` | ❌ | — |

#### Example cURL

```bash
curl -X GET 'https://petstore.example.com/v1/pets?limit=1'
```

#### Responses

##### ✅ 200 `A list of pets`

---

### 🔵 `/pets` 

**Create a pet** — `POST`

**Tags:** `pets`

#### Request Body

**Content-Type:** `json`
**Actual Content-Type:** `application/json`
**Required:** ✅

**Schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `breed` | `string` | ❌ | — |
| `name` | `string` | ✅ | — |

#### Example cURL

```bash
curl -X POST 'https://petstore.example.com/v1/pets' \
  -d '{}'
```

#### Responses

##### ✅ 201 `Pet created`

---

