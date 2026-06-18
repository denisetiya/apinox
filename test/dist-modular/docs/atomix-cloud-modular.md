# atomix-cloud-modular API Documentation

**Version:** `1.0.0`

Modular test with includes

**Base URL:** `https://api.denisetiya.site/v1`

## Environments

- **production** — `https://api.denisetiya.site/v1`

## Authentication

**Default scheme:** `bearer`

### bearer (HTTP BEARER)


## Table of Contents

- [Products (1 endpoint)](#products)

## Products

Product catalog

### 🟢 `/products` 

**List Products** — `GET`

Get all products

#### Query Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `page` | `integer` | ✅ | — |
| `per_page` | `integer` | ✅ | — |

#### Example cURL

```bash
curl -X GET 'https://api.denisetiya.site/v1/products?page=1&per_page=1'
```

#### Responses

##### ✅ 200 `Product list`

*success*

```json
{
  "data": [
    {
      "id": "prod_001",
      "name": "Widget",
      "price": 99000
    }
  ],
  "meta": {
    "page": 1,
    "total": 42
  }
}
```

---

