# Authentication

All API requests must include a bearer token in the `Authorization` header.

## Obtaining a Token

```bash
curl -X POST https://api.example.com/v1/auth/token \
  -d '{"username": "alice", "password": "secret"}'
```

Response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_in": 3600
}
```

## Using the Token

Include the token in every request:

```bash
curl -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..." \
  https://api.example.com/v1/users
```

:::warning{title="Security"}
Never commit tokens to version control. Use environment variables or a secrets
manager instead.
:::

See [[endpoints|REST Endpoints]] for the available routes, or head back to the
[[overview|API Overview]].
