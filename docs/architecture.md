# CFD-Broker Architecture Overview

## Overall Flow Sequence Diagram

```mermaid
sequenceDiagram
    participant Web as Next.js Web App
    participant API as Express API Server
    participant Kafka
    participant DBProc as DB Processor
    participant DB as PostgreSQL
    participant Redis
    participant Email as Email Service

    Note over Web,Email: Example: User Signup Flow
    Web->>API: POST /api/v1/auth/signup (email)
    API->>API: Validate with Zod (schemas)
    API->>Kafka: user-existence-check
    Kafka->>DBProc: Consume message
    DBProc->>DB: Prisma: findUnique user
    DB-->>DBProc: User data or null
    DBProc->>Kafka: user-existence-response
    Kafka-->>API: Response
    API->>API: Generate OTP
    API->>Redis: SET otp:{email} hashedOTP EX 300
    API->>Email: Send OTP email (Nodemailer)
    Email-->>API: Email sent
    API-->>Web: 200 "OTP sent"

    Web->>API: POST /api/v1/auth/verify (email, otp, userData)
    API->>Redis: GET otp:{email}
    Redis-->>API: hashedOTP
    API->>API: bcrypt.compare
    API->>Redis: DEL otp:{email}
    API->>API: bcrypt.hash password
    API->>Kafka: user-creation-request
    Kafka->>DBProc: Consume
    DBProc->>DB: Prisma: create user
    DB-->>DBProc: New user
    DBProc->>Kafka: user-creation-response
    Kafka-->>API: Response
    API->>API: jwt.sign ({id, email})
    API->>API: Set httpOnly cookie
    API->>Redis: SET jwt:{token} EX 3600
    API-->>Web: 200 "User created"

    Web->>API: POST /api/v1/auth/signin (email, password)
    API->>Kafka: user-authentication-request
    Kafka->>DBProc: Consume
    DBProc->>DB: Prisma: findUnique user
    DB-->>DBProc: hashedPassword
    DBProc->>Kafka: user-authentication-response
    Kafka-->>API: Response
    API->>API: bcrypt.compare
    API->>API: jwt.sign
    API->>Redis: SET jwt:{token}
    API-->>Web: 200 "Signed in"

    Web->>API: GET /api/v1/auth/me (with cookie)
    API->>API: requireAuth: jwt.verify
    API->>Redis: GET jwt:{token}
    Redis-->>API: Session data
    API-->>Web: 200 {user: {id, email}}

    Web->>API: POST /api/v1/auth/logout
    API->>Redis: DEL jwt:{token}
    API-->>Web: 200 "Logged out"
```

## Architecture Description

### Core Components

1. **Web App (Next.js)**: Frontend client for user interactions.
2. **API Server (Express)**: Handles HTTP requests, validation, auth, OTP, JWT.
3. **Kafka**: Async messaging for decoupling API and DB operations.
4. **DB Processor**: Consumes Kafka messages, performs DB queries via Prisma.
5. **PostgreSQL**: Relational DB for persistent user data.
6. **Redis**: In-memory cache for sessions and OTPs.
7. **Email Service**: SMTP for OTP emails (Gmail via Nodemailer).

### Shared Packages

- **Schemas (/packages/schemas)**: Zod for input validation.
- **UI (/packages/ui)**: Reusable React components.
- **Kafka (/packages/kafka)**: Shared Kafka client.
- **Redis (/packages/redis)**: Shared Redis client.
- **DB (/packages/db)**: Prisma client and migrations.
- **TypeScript Config (/packages/typescript-config)**: Shared TS settings.

### Key Design Patterns

- **Event-Driven**: Kafka for async, scalable communication.
- **Stateless API**: JWT + Redis for sessions (no server-side state).
- **Security**: bcrypt for passwords, OTP for verification, httpOnly cookies.
- **Monorepo**: Turborepo for managing packages and builds.

## Performance Notes

- **Signup**: ~3.5s (email bottleneck)
- **Signin**: ~200ms (Kafka + bcrypt)
- **Verify OTP**: ~270ms (Kafka + creation)
- **Logout /me**: <10ms (Redis ops)

## Deployment

- Containerize with Docker (separate containers for API, DBProc, Kafka, Redis).
- Use env vars for configs (DB URLs, secrets).
- Scale DBProc independently for high load.