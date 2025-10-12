# CFD-Broker: Real-Time CFD Trading Platform

CFD-Broker is a monorepo-based real-time CFD (Contract for Difference) trading platform built with Turborepo, featuring microservices architecture for scalability and modularity. It provides user authentication, real-time price streaming via WebSocket, and a responsive web interface for trading simulations.

## Architecture Overview

The platform uses a microservices architecture with event-driven communication via Kafka, in-memory caching with Redis, and a PostgreSQL database for persistence. Key components include:

- **Frontend**: Next.js web app for user interactions.
- **API Server**: Express.js server handling HTTP requests, authentication, and WebSocket broadcasting.
- **DB Processor**: Kafka consumer for database operations.
- **Poller**: WebSocket client polling real-time prices from Backpack Exchange.
- **Shared Packages**: Reusable libraries for UI, schemas, Kafka, Redis, and DB clients.

### High-Level Architecture Diagram

```mermaid
graph TB
    subgraph "Frontend Layer"
        Web[Web App<br/>Next.js]
    end

    subgraph "API Layer"
        API[API Server<br/>Express.js<br/>WebSocket Server]
    end

    subgraph "Event-Driven Layer"
        Kafka[Kafka<br/>Message Broker]
    end

    subgraph "Data Layer"
        DB[PostgreSQL<br/>User Data]
        Redis[Redis<br/>Cache & Sessions]
    end

    subgraph "External Services"
        Poller[Poller<br/>WebSocket Client<br/>Backpack Exchange]
        Email[Email Service<br/>Gmail SMTP]
    end

    Web --> API
    API --> Kafka
    API --> Redis
    API --> Email
    Poller --> Kafka
    Kafka --> DBProc[DB Processor<br/>Kafka Consumer]
    DBProc --> DB
    API --> WebSocket[WebSocket<br/>Real-Time Prices]
    Web --> WebSocket
```

### Data Flow
1. **User Signup/Signin**: Web app → API Server → Kafka → DB Processor → DB (with OTP via Email).
2. **Price Streaming**: Poller → Kafka → API Server → WebSocket → Web App.
3. **Authentication**: JWT + Redis sessions for secure access.

## Project Structure

This Turborepo workspace is organized as follows:

```
.gitignore
index.ts
package.json
README.md
tsconfig.json
turbo.json
apps/
├── api-server/          # Express.js API server (auth, prices, WebSocket)
├── db-processor/        # Kafka consumer for DB operations
├── poller/              # WebSocket poller for real-time prices
└── web/                 # Next.js frontend app
docs/                    # Documentation (flows, architecture)
packages/                # Shared libraries
├── db/                  # Prisma client and migrations
├── eslint-config/       # ESLint configurations
├── kafka/               # Kafka producer/consumer client
├── redis/               # Redis client
├── schemas/             # Zod validation schemas
├── typescript-config/   # TypeScript configurations
└── ui/                  # Reusable React components
```

## Services (Microservices)

### API Server ([`apps/api-server`](apps/api-server ))
- **Framework**: Express.js with TypeScript.
- **Responsibilities**:
  - Handles HTTP requests for authentication (signup, signin, verify OTP, logout, /me).
  - Manages WebSocket server for real-time price broadcasting.
  - Validates inputs with Zod schemas.
  - Communicates with Kafka for user checks/creations.
  - Caches OTPs and sessions in Redis.
  - Sends OTP emails via Gmail SMTP.
- **Ports**: HTTP on 3001, WebSocket on 8080.
- **Key Features**: JWT-based auth, httpOnly cookies, bcrypt hashing.

### DB Processor ([`apps/db-processor`](apps/db-processor ))
- **Framework**: Node.js with KafkaJS.
- **Responsibilities**:
  - Consumes Kafka messages for user existence checks, creations, and authentications.
  - Interacts with PostgreSQL via Prisma for CRUD operations.
  - Sends responses back via Kafka.
- **Key Features**: Event-driven, stateless, scalable for high DB load.

### Poller ([`apps/poller`](apps/poller ))
- **Framework**: Node.js with WebSocket client.
- **Responsibilities**:
  - Connects to Backpack Exchange WebSocket for real-time price data (SOL_USDC, BTC_USDC, ETH_USDC, DOGE_USDC, BNB_USDC).
  - Publishes price updates to Kafka topic `'price-updates'`.
  - Handles reconnections and errors.
- **Key Features**: Fault-tolerant polling, schema validation before publishing.

### Web App ([`apps/web`](apps/web ))
- **Framework**: Next.js with React.
- **Responsibilities**:
  - Provides UI for user signup/signin, OTP verification, and live price display.
  - Connects to API WebSocket for real-time updates.
  - Uses shared UI components.
- **Ports**: Runs on 3000.
- **Key Features**: Responsive design, client-side WebSocket integration.

## Shared Packages

### `@repo/ui` ([`packages/ui`](packages/ui ))
- Reusable React components (e.g., Button, Card, Code).
- Built with TypeScript and styled for consistency.

### `@repo/schemas` ([`packages/schemas`](packages/schemas ))
- Zod schemas for input validation (SignupSchema, SigninSchema, VerifyOtpSchema, PriceUpdateSchema).
- Ensures data integrity across services.

### `@repo/kafka` ([`packages/kafka`](packages/kafka ))
- Singleton Kafka client with producer and consumer.
- Handles connections and topic subscriptions.

### `@repo/redis` ([`packages/redis`](packages/redis ))
- Redis client for caching and sessions.
- Includes health checks and connection management.

### `@repo/db` ([`packages/db`](packages/db ))
- Prisma client for PostgreSQL interactions.
- Includes migrations and seed scripts.

### `@repo/eslint-config` & `@repo/typescript-config`
- Shared linting and TypeScript configurations for consistency.

## Setup and Installation

### Prerequisites
- Node.js/Bun, PostgreSQL, Redis, Kafka (e.g., via Docker).
- Gmail account for SMTP (set app password).

### Installation
1. Clone the repo: `git clone <repo-url>`.
2. Install dependencies: `bun install`.
3. Set up environment variables (copy [`apps/api-server/.env.example`](apps/api-server/.env.example ) to `.env` in each app/package):
   - [`apps/api-server/.env`](apps/api-server/.env ): [`PORT=3001`](apps/api-server/src/index.ts ), [`JWT_SECRET=<secret>`](apps/api-server/src/controllers/signin/signin.ts ), [`GMAIL_USER=<email>`](apps/api-server/src/services/mail/otpMail.ts ), [`GMAIL_APP_PASSWORD=<password>`](apps/api-server/src/services/mail/otpMail.ts ), [`WS_PORT=8080`](apps/api-server/src/controllers/prices/prices.ts ).
   - [`packages/kafka/.env.example`](packages/kafka/.env.example ): [`KAFKA_BROKER=localhost:9092`](apps/db-processor/src/kafka/kafkaConsumer.ts ).
   - [`packages/redis/.env.example`](packages/redis/.env.example ): [`REDIS_URL=redis://localhost:6379`](packages/redis/src/client.ts ).
   - [`packages/db/env.example`](packages/db/env.example ): `DATABASE_URL=postgresql://user:pass@localhost:5432/cfdbroker`.
   - [`apps/poller/src/config.ts`](apps/poller/src/config.ts ): [`BACKPACK_WS_URL=wss://ws.backpack.exchange/`](apps/poller/src/config.ts ).
4. Run DB migrations: `cd packages/db && bun run db:push`.
5. Seed DB (optional): [`bun run prisma/seed.ts`](apps/db-processor/src/handlers/userAuthenticationHandler.ts ).

## Running the Project

Use Turborepo for development:

```bash
# Start all services
turbo dev

# Or start individually
turbo dev --filter=api-server
turbo dev --filter=web
turbo dev --filter=poller
turbo dev --filter=db-processor
```

- **API Server**: `bun run start` in [`apps/api-server`](apps/api-server ).
- **Web App**: `npm run dev` in [`apps/web`](apps/web ).
- **Poller**: `bun run dev` in [`apps/poller`](apps/poller ).
- **DB Processor**: `bun run dev` in [`apps/db-processor`](apps/db-processor ).

Ensure Kafka, Redis, and PostgreSQL are running (e.g., via Docker Compose).

## Performance Metrics

Based on internal benchmarks:
- **Signup**: ~3.5s (bottleneck: email sending via SMTP).
- **Signin**: ~200ms (Kafka round-trip + bcrypt).
- **Verify OTP**: ~270ms (Kafka + user creation).
- **Logout /me**: <10ms (Redis ops).
- **Price Updates**: Real-time via WebSocket, <50ms latency from poller to client.
- Scalability: DB Processor can scale independently; API handles 1000+ concurrent WebSocket clients.

## Contributing

- Follow ESLint/Prettier rules.
- Use Zod for validation.
- Test Kafka/Redis connections.
- Update docs for new features.

## Useful Links

- [Turborepo Docs](https://turborepo.com/docs)
- [Prisma Docs](https://www.prisma.io/docs)
- [KafkaJS Docs](https://kafka.js.org/)
- [Backpack Exchange API](https://docs.backpack.exchange/)

For issues, open a PR or issue in the repo.