# REST Endpoints

Below is an example endpoint reference. Replace these with your own API routes.

## Endpoints

| Method | Path             | Description        |
|--------|------------------|--------------------|
| GET    | `/v1/users`      | List all users     |
| POST   | `/v1/users`      | Create a new user  |
| GET    | `/v1/users/:id`  | Get user by ID     |
| DELETE | `/v1/users/:id`  | Delete a user      |

## Example Request

```bash
curl https://api.example.com/v1/users
```

## Example Response

```json
{
  "users": [
    { "id": 1, "name": "Alice" },
    { "id": 2, "name": "Bob" }
  ]
}
```

All requests require a valid token — see [[authentication]] for details.
For an overview of the API, visit [[overview|API Overview]].
