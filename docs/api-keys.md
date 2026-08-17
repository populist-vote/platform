# Self-service API keys

Registered users manage API keys from the **API keys** section of their
profile. The profile uses these authenticated GraphQL operations:

- `apiKeys` lists the current user's active keys.
- `createApiKey(name: String!)` creates a key and returns its secret once.
- `revokeApiKey(id: ID!)` immediately revokes a key owned by the current user.

Key-management operations require an interactive access-token or cookie
session. An API key cannot list, create, or revoke keys.

## Authentication flow

Keys use the `pop_` prefix followed by 32 random bytes encoded as unpadded
base64url. The server stores a SHA-256 hash, the first eight encoded characters
for identification, ownership, and audit timestamps. It never stores the full
secret.

The shared HTTP bearer resolver accepts either an existing access JWT or an API
key from:

```http
Authorization: Bearer pop_...
```

After a key is resolved, the server loads its owner and current organization
roles into the same request-local claims used by cookie and JWT authentication.
This makes one key work across GraphQL and versioned REST routes and ensures
permission changes apply without reissuing it. Last-use time is updated at most
once every five minutes to avoid a write on every API request.

## Operational notes

- Only confirmed users can create keys.
- Each user may have at most 10 active keys.
- Active key names must be unique per user, case-insensitively.
- Revocation is immediate; revoked keys are retained for audit history but are
  excluded from authentication and self-service lists.
- Deleting a user cascades to all of their API keys.
- Apply `db/migrations/20260805120000_CreateUserApiKeys.up.sql` before deploying
  code that accepts self-service keys.
