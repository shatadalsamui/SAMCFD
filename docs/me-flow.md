# /me Flow Documentation

## Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant API Server
    participant Redis

    User->>API Server: GET /api/v1/auth/me (with auth_token cookie)
    API Server->>API Server: requireAuth middleware: Get auth_token from cookies
    alt No token
        API Server-->>User: 401 No token
    else Token exists
        API Server->>API Server: jwt.verify(token, SECRET)
        alt Invalid JWT
            API Server-->>User: 401 Invalid token
        else Valid JWT
            API Server->>Redis: GET jwt:{token}
            alt No session in Redis
                API Server-->>User: 401 Invalid token
            else Session exists
                API Server->>API Server: Set req.user = {id, email}
                API Server-->>User: 200 {user: {id, email}}
            end
        end
    end
```

## Flow Description

1. **Middleware Check**: `requireAuth` extracts the `auth_token` from cookies.
2. **JWT Verification**: Validates the JWT signature and expiration.
3. **Session Validation**: Checks if the token exists in Redis (session store).
4. **User Data**: If valid, attaches decoded user data (`id`, `email`) to `req.user`.
5. **Response**: Returns the user object in JSON.

## Performance Notes

- Total time: ~2ms
- Extremely fast: JWT verify (~1ms), Redis GET (~1ms), no DB hits.

## Error Handling

- No token: 401 No token
- Invalid/expired JWT: 401 Invalid token
- - Missing Redis session: 401 Invalid token (handles token reuse/theft)
