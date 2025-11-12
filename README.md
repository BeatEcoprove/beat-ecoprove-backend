# Beat EcoProve Backend

## Overview

Beat EcoProve is a modern, scalable platform designed to promote sustainable living through a circular economy for clothing and apparel. The backend is built as a distributed microservices architecture that powers eco-conscious communities by facilitating clothing sharing, tracking sustainability metrics, and rewarding environmentally responsible behavior.

The platform enables users to:
- Manage their profiles and sustainability journey
- Discover and interact with local retail stores
- Track and share apparel items in a circular economy
- Communicate and collaborate with other eco-conscious users
- Earn rewards (XP/points) for sustainable actions

## Architecture

### System Overview

The Beat EcoProve backend follows a **polyglot microservices architecture** with event-driven communication, fronted by a high-performance **Pingora-based API Gateway**. Each service is built using the technology stack best suited for its domain responsibilities.

This repository contains the **Rust-based Pingora reverse proxy** that serves as the unified entry point for all client requests, providing intelligent routing, request tracking, and security features.

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Applications                       │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  Pingora Proxy   │
                    │  (Port 9000)     │
                    │                  │
                    │  Rust/Pingora    │
                    └────────┬─────────┘
                             │
             ┌───────────────┼───────────────┐
             │               │               │
             ▼               ▼               ▼
      ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
      │ Auth Service │ │ Core Service │ │  Messaging   │
      │   (Port      │ │   (Port      │ │   Service    │
      │    2000)     │ │    3000)     │ │ (Port 4000)  │
      │              │ │              │ │              │
      │   Go/Fiber   │ │  .NET 9.0    │ │   Elixir     │
      └──────┬───────┘ └──────┬───────┘ └──────┬───────┘
             │                │                 │
             │         ┌──────┴──────┐          │
             │         │             │          │
             └─────────┤ Kafka Broker├──────────┘
                       │ (Port 9092) │
                       └──────┬──────┘
                              │
             ┌────────────────┼────────────────┐
             │                │                │
             ▼                ▼                ▼
      ┌──────────┐     ┌──────────┐    ┌──────────┐
      │PostgreSQL│     │  Redis   │    │ MongoDB  │
      │ (5432)   │     │  (6379)  │    │ (27017)  │
      └──────────┘     └──────────┘    └──────────┘
```

### API Gateway (Pingora Proxy)

**This Repository** contains the Pingora-based reverse proxy implementation.
**Technology:** Rust 1.91+ with Pingora 0.6.0
**Port:** 9000 (configurable via `PROXY_HOST`/`PROXY_PORT`)

**Key Features:**
- **Intelligent Routing**: Path-based routing with prefix stripping and API versioning
- **Request Tracking**: Automatic X-Request-ID header injection for distributed tracing
- **Security Headers**: Adds X-Frame-Options (DENY) and X-Content-Type-Options (nosniff)
- **High Performance**: Built on LinkedIn's Pingora framework for exceptional throughput
- **Configuration-Driven**: JSON-based routing rules in `config/services.json`

**Routing Configuration (`config/services.json`):**
```json
{
  "services": [
    {
      "name": "auth-service",
      "host": "auth-service",
      "port": 2000,
      "prefix": "/auth",
      "strip_prefix": true,
      "skip_prefix": ["/.well-known", "/swagger", "/health"]
    },
    {
      "name": "core-service",
      "host": "core-service",
      "port": 3000,
      "prefix": "/core",
      "strip_prefix": true,
      "skip_prefix": ["/swagger"]
    },
    {
      "name": "messaging-service",
      "host": "messaging-service",
      "port": 4000,
      "prefix": "/messaging",
      "strip_prefix": true,
      "skip_prefix": ["/swagger"]
    },
    {
      "name": "websocket",
      "host": "messaging-service",
      "port": 4000,
      "prefix": "/socket",
      "strip_prefix": false
    }
  ]
}
```

**Example Request Flow:**
```
Client Request: GET http://localhost:9000/auth/api/v1/users/profile
                     ↓
Pingora Proxy: Adds X-Request-ID, security headers
                     ↓
Routes to: http://auth-service:2000/api/v1/users/profile
           (prefix "/auth" stripped)
```

### Microservices Overview

### Service Responsibilities

#### 1. Auth Service (Identity & Authentication)
**Repository:** [beat-ecoprove-auth](https://github.com/BeatEcoprove/beat-ecoprove-auth)
**Technology:** Go 1.23+ with Fiber framework
**Port:** 2000

**Responsibilities:**
- OAuth2-style authentication with RS256 JWT signing
- User identity management and multi-profile support
- Role-based access control (RBAC) and permissions
- Password recovery flows with secure code generation
- Token lifecycle management via Redis
- Publishing authentication events to Kafka

**Data Storage:**
- PostgreSQL: User credentials, profiles, roles, and permissions
- Redis: Active sessions, token blacklists, and temporary codes

**Key Dependencies:**
- GORM for database operations
- Fiber for HTTP routing
- Redis for distributed caching
- Kafka for event streaming

---

#### 2. Core Service (Business Logic)
**Repository:** [beat-ecoprove-core](https://github.com/BeatEcoprove/beat-ecoprove-core)
**Technology:** .NET 9.0 with ASP.NET Core
**Port:** 3000

**Responsibilities:**
- User profile administration and sustainability tracking
- Retail store/location management with geospatial queries
- Apparel item catalog and lifecycle tracking
- Gamification system (XP, levels, achievements)
- Sustainability metrics and analytics
- Publishing domain events to Kafka

**Data Storage:**
- PostgreSQL with PostGIS: Users, stores (with geo-coordinates), apparel items, transactions
- Redis: Query result caching, frequently accessed data

**Key Dependencies:**
- Entity Framework Core for ORM
- Carter for minimal APIs
- PostGIS for location-based features
- Kafka for event streaming
- OpenTelemetry for observability

---

#### 3. Messaging Service (Real-time Communication)
**Repository:** [beat-ecoprove-messaging](https://github.com/BeatEcoprove/beat-ecoprove-messaging)
**Technology:** Elixir 1.18+ with Phoenix Framework
**Port:** 4000

**Responsibilities:**
- Real-time chat via WebSocket channels
- Group management for community collaboration
- Borrow request tracking and coordination
- Push notifications for invites and activities
- Reward distribution for social engagement
- Publishing messaging events to Kafka

**Data Storage:**
- PostgreSQL: Users, groups, memberships, invitations
- MongoDB: Message history and chat logs (scalable document storage)
- Redis: User presence, online status, and caching

**Key Dependencies:**
- Phoenix Framework for WebSockets
- Ecto for PostgreSQL operations
- MongoDB driver for message storage
- Kafka for event streaming

---

### Infrastructure Components

#### Apache Kafka (Event Bus)
**Port:** 9092 (internal), 9094 (external)

Kafka serves as the central nervous system of the architecture, enabling asynchronous, event-driven communication between services. All microservices publish and consume domain events through Kafka topics.

**Event Flow Examples:**
- `user.registered` (Auth) → Core creates profile, Messaging initializes chat capability
- `item.shared` (Core) → Messaging notifies interested groups, Auth updates permissions
- `message.sent` (Messaging) → Core awards XP points
- `profile.updated` (Core) → Auth syncs user data, Messaging updates user info

#### PostgreSQL (Primary Database)
**Port:** 5432
**Version:** PostgreSQL 17 with PostGIS 3

Each service maintains its own PostgreSQL database following the **Database per Service** pattern:
- `auth_service`: Authentication and authorization data
- `core_service`: Business domain entities and relationships
- `messaging_service`: Group structures and user relationships

**PostGIS Extension:** Enabled for geospatial queries (finding nearby stores, location-based recommendations).

**Initialization:** Databases are automatically created via `docker/postgres/init/` scripts on first startup.

#### MongoDB (Document Store)
**Port:** 27017

Used exclusively by the Messaging Service for high-volume message storage. The document-oriented nature of MongoDB is ideal for chat message history which doesn't require strict relational constraints.

#### Redis (Distributed Cache)
**Port:** 6379

Shared caching layer used by all services for:
- JWT token blacklisting and session management
- Frequently accessed read-heavy data
- Rate limiting and request throttling
- User presence and online status
- Temporary data (password reset codes, verification tokens)

---

## Technology Stack Summary

### Languages & Frameworks
| Service    | Language | Framework        | Version |
|-----------|----------|------------------|---------|
| Proxy     | Rust     | Pingora          | 1.91+   |
| Auth      | Go       | Fiber            | 1.23.2  |
| Core      | C#       | ASP.NET Core     | 9.0     |
| Messaging | Elixir   | Phoenix          | 1.18.0  |

### Infrastructure
| Component  | Technology       | Purpose                           |
|-----------|------------------|-----------------------------------|
| Gateway   | Pingora 0.6      | High-performance reverse proxy    |
| Event Bus | Apache Kafka 4.0 | Asynchronous event streaming      |
| Database  | PostgreSQL 17    | Primary relational data store     |
| Database  | MongoDB 8.0      | Document store for messages       |
| Cache     | Redis Stack      | Distributed caching & sessions    |

### Container Orchestration
- **Nix Flakes**: Reproducible builds and dependency management
- **Podman/Docker**: Container runtime and image management
- **Docker Compose**: Multi-container orchestration

---

## Getting Started

### Prerequisites

- **Nix** with flakes enabled (recommended) OR
- **Podman** or **Docker** with Compose support
- **Just** command runner (optional but recommended)

### Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/BeatEcoprove/beat-ecoprove-backend.git
   cd beat-ecoprove-backend
   ```

2. **Configure environment variables**
   ```bash
   cp .env.example .env
   ```

   Edit `.env` and configure the following sections:
   - **Proxy settings** (PROXY_HOST, PROXY_PORT - default: 9000)
   - **Database credentials** (PostgreSQL, MongoDB)
   - **JWT settings** (issuer, audience, expiration)
   - **Service ports** (Auth: 2000, Core: 3000, Messaging: 4000)
   - **Kafka broker** (host and port)
   - **Redis connection** (host and port)

3. **Build microservice images**
   ```bash
   just build-images
   ```

   This uses Nix flakes to build all three microservices as container images.

4. **Start the platform**

   **Development mode** (with live reload, debugging, and admin UIs):
   ```bash
   just dev
   ```

   Development mode includes:
   - MongoDB Express UI: http://localhost:8081
   - Redis Stack UI: http://localhost:8001

   **Production mode** (optimized builds with persistent volumes):
   ```bash
   just prod
   ```

   **Alternative: Docker Compose directly**
   ```bash
   # Development
   podman compose -f docker-compose.yml -f docker-compose.dev.yml up -d

   # Production
   podman compose -f docker-compose.yml -f docker-compose.prod.yml up -d
   ```

5. **Run the proxy locally (development)**
   ```bash
   just serve
   ```

   This starts the Pingora proxy using `cargo run` for local development without containers.

6. **Verify services are running**
   ```bash
   podman ps
   ```

   You should see containers for:
   - `proxy` (Pingora reverse proxy - port 9000)
   - `auth-service` (port 2000)
   - `core-service` (port 3000)
   - `messaging-service` (port 4000)
   - `kafka-b` (ports 9092, 9094)
   - `postgres` (port 5432)
   - `mongo` (port 27017)
   - `redis` (port 6379)

7. **Access the platform**

   All requests go through the proxy at http://localhost:9000:
   - Auth API: http://localhost:9000/auth/*
   - Core API: http://localhost:9000/core/*
   - Messaging API: http://localhost:9000/messaging/*
   - WebSocket: ws://localhost:9000/socket

---

## API Documentation

### Through Proxy (Recommended)

Access all services through the unified gateway at port 9000:

| Service   | Proxy URL                                      | Description                          |
|-----------|------------------------------------------------|--------------------------------------|
| Auth      | http://localhost:9000/auth/swagger/index.html  | Authentication & authorization APIs  |
| Core      | http://localhost:9000/core/swagger             | Business logic & domain APIs         |
| Messaging | http://localhost:9000/messaging/api/swagger    | Real-time messaging & notifications  |

### Direct Service Access

For debugging or development, services can be accessed directly:

| Service   | Direct URL                               | Description                          |
|-----------|------------------------------------------|--------------------------------------|
| Auth      | http://localhost:2000/swagger/index.html | Authentication & authorization APIs  |
| Core      | http://localhost:3000/swagger            | Business logic & domain APIs         |
| Messaging | http://localhost:4000/api/swagger        | Real-time messaging & notifications  |

---

## Development Workflow

### Available Just Commands

This project uses [Just](https://github.com/casey/just) as a task runner. Available commands:

```bash
just serve           # Run Pingora proxy locally with Cargo (development)
just build-images    # Build all microservice Docker images using Nix
just compose-dev     # Start services with development configuration
just compose-build   # Start services with production configuration
just dev            # Full dev workflow: build-images + compose-dev
just prod           # Full prod workflow: build-images + compose-build
```

### Building Individual Services

```bash
# Build specific service
nix run .#auth-service
nix run .#core-service
nix run .#messaging-service

# Build all services
nix run .#build-all
```

### Developing the Proxy

The Rust proxy can be developed independently:

```bash
# Run with hot-reload (requires cargo-watch)
cargo watch -x run

# Build for production
cargo build --release

# Run tests (when available)
cargo test

# Check code without building
cargo check
```

### Database Migrations

Each service manages its own database migrations:

**Auth Service (Go):**
```bash
# Inside auth-service directory
make migrate          # Run pending migrations
make rollback         # Rollback last migration
make migration-status # Check migration status
```

**Core Service (.NET):**
```bash
# Inside core-service directory
just migration-push   # Apply migrations
dotnet ef migrations add <MigrationName>  # Create new migration
```

**Messaging Service (Elixir):**
```bash
# Inside messaging-service directory
mix ecto.migrate      # Run migrations
mix ecto.rollback     # Rollback migration
mix ecto.gen.migration <migration_name>  # Generate migration
```

### Running Tests

Each service has its own test suite:

```bash
# Auth Service
cd beat-ecoprove-auth && make test

# Core Service
cd beat-ecoprove-core && dotnet test

# Messaging Service
cd beat-ecoprove-messaging && mix test
```

---

## Environment Variables

### Global Configuration

| Variable              | Description                        | Default           |
|-----------------------|------------------------------------|-------------------|
| `CONTAINER_TOOL`      | Container runtime (podman/docker)  | `podman`          |
| `PROXY_HOST`          | Proxy bind address                 | `0.0.0.0`         |
| `PROXY_PORT`          | Proxy listen port                  | `9000`            |

### Service Ports

| Variable                  | Description              | Default |
|---------------------------|--------------------------|---------|
| `BEAT_IDENTITY_SERVER`    | Auth service port        | `2000`  |
| `BEAT_API_REST_PORT`      | Core service port        | `3000`  |
| `BEAT_MESSASSING_SERVER`  | Messaging service port   | `4000`  |

### Database Configuration

**PostgreSQL:**
```env
POSTGRES_HOST=postgres
POSTGRES_PORT=5432
POSTGRES_USER=auth
POSTGRES_PASSWORD=<secure-password>
POSTGRES_DB=auth_service

# Additional databases created automatically:
# - core_service
# - messaging_service
```

**MongoDB:**
```env
MONGO_HOST=localhost
MONGO_PORT=27017
MONGO_USERNAME=messaging
MONGO_PASSWORD=<secure-password>
MONGO_DB=messaging
```

### Infrastructure

**Kafka:**
```env
KAFKA_HOST=kafka-b
KAFKA_PORT=9094
```

**Redis:**
```env
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_DB=0
```

### Security

**JWT Configuration:**
```env
JWT_ISSUER=Beatecoprove
JWT_AUDIENCE=http://localhost:2000
JWT_ACCESS_EXPIRED=30
JWT_REFRESH_EXPIRED=30
JWKS_URL=http://localhost:2000
```

---

## Architecture Principles

### 1. Microservices Pattern
Each service is independently deployable, scalable, and maintainable. Services communicate exclusively through well-defined APIs and asynchronous events.

### 2. Event-Driven Architecture
Kafka enables loose coupling between services. Domain events are published when significant state changes occur, allowing other services to react asynchronously.

### 3. Database per Service
Each microservice owns its database schema. Cross-service data access happens only through APIs or events, never through direct database queries.

### 4. Polyglot Persistence
Different data storage technologies are chosen based on service requirements:
- PostgreSQL for transactional, relational data
- MongoDB for high-volume document storage
- Redis for ephemeral, high-speed caching

### 5. Clean Architecture
All services follow Clean Architecture / Domain-Driven Design principles with clear separation between:
- **Presentation Layer**: HTTP handlers, WebSocket channels
- **Application Layer**: Use cases, business workflows
- **Domain Layer**: Core business logic and entities
- **Infrastructure Layer**: Database access, external integrations

### 6. Security by Design
- RS256 JWT tokens for stateless authentication
- JWKS endpoint for public key distribution
- Role-based access control (RBAC)
- Secure password hashing with bcrypt
- Token rotation and blacklisting

---

## Inter-Service Communication

### Synchronous (HTTP/REST)
Used sparingly for:
- Client-to-service requests
- Health checks and service discovery
- Real-time queries requiring immediate responses

### Asynchronous (Kafka Events)
Primary communication mechanism for:
- User lifecycle events (registration, profile updates)
- Domain events (item shared, message sent, XP earned)
- Cross-service data synchronization
- Audit logging and analytics

**Example Event Flow:**
```
1. User registers via Auth Service
   └─> Publishes: user.registered

2. Core Service consumes event
   └─> Creates user profile
   └─> Publishes: profile.created

3. Messaging Service consumes event
   └─> Initializes chat capabilities
   └─> Publishes: messaging.initialized
```

---

## Monitoring & Observability

### Logging
- Structured logging in JSON format
- Correlation IDs for request tracing across services
- Centralized log aggregation (implementation-specific)

### Metrics
- **Core Service**: OpenTelemetry instrumentation
- Service health endpoints: `/health` or `/api/health`
- Kafka consumer lag monitoring

### Tracing
- Distributed tracing support via OpenTelemetry (Core Service)
- Request flow visualization across microservices

---

## Deployment

### Development Environment
```bash
just dev
# Starts all services with hot-reload and debug logging
```

### Production Environment
```bash
just prod
# Builds optimized images and starts services
```

### Container Registry
Images are tagged as:
- `localhost/auth-service:1.0.0`
- `localhost/core-service:1.0.0`
- `localhost/messaging-service:1.0.0`

---

## Project Structure

```
beat-ecoprove-backend/
├── src/                     # Rust proxy source code (~222 lines)
│   ├── main.rs             # Entry point - initializes Pingora server
│   ├── proxy.rs            # BeatProxy implementation with routing logic
│   ├── routing.rs          # Service routing configuration parser
│   ├── error.rs            # Custom error types
│   └── lib.rs              # Module declarations
│
├── config/                 # Configuration files
│   └── services.json       # Service routing configuration
│
├── docker/                 # Docker configuration files
│   └── postgres/           # PostgreSQL with PostGIS
│       ├── Dockerfile      # PostgreSQL 17 + PostGIS 3
│       └── init/           # Database initialization scripts
│           ├── 001-extensions.sql      # Create PostGIS extension
│           └── 002-create-databases.sql # Create service databases
│
├── nix/                    # Nix configuration
│   └── services.nix        # Service build definitions
│
├── Cargo.toml              # Rust dependencies and project metadata
├── Cargo.lock              # Locked Rust dependency versions
├── Dockerfile              # Multi-stage build for Rust proxy
├── .env                    # Environment configuration (not in git)
├── .env.example            # Example environment template
├── docker-compose.yml      # Base compose configuration
├── docker-compose.dev.yml  # Development overrides (with admin UIs)
├── docker-compose.prod.yml # Production overrides (with volumes)
├── flake.nix               # Nix flake for reproducible builds
├── flake.lock              # Locked Nix dependency versions
├── Justfile                # Task automation commands
├── .gitignore              # Git ignore rules
├── .dockerignore           # Docker ignore rules
└── README.md               # This file
```

### Rust Dependencies (Cargo.toml)

```toml
[dependencies]
pingora = "0.6.0"              # High-performance proxy framework
pingora-core = "0.6.0"         # Core Pingora components
pingora-proxy = "0.6.0"        # HTTP proxy layer
tokio = { version = "1.44", features = ["full"] }  # Async runtime
async-trait = "0.1.89"         # Async trait support
env_logger = "0.11.8"          # Logging configuration
log = "0.4.27"                 # Logging facade
serde = { version = "1.0", features = ["derive"] }  # Serialization
serde_json = "1.0"             # JSON support
uuid = { version = "1.16", features = ["v4"] }  # UUID generation
```

**Microservice Repositories (External):**
- [beat-ecoprove-auth](https://github.com/BeatEcoprove/beat-ecoprove-auth) - Identity & authentication (Go)
- [beat-ecoprove-core](https://github.com/BeatEcoprove/beat-ecoprove-core) - Core business logic (.NET)
- [beat-ecoprove-messaging](https://github.com/BeatEcoprove/beat-ecoprove-messaging) - Real-time messaging (Elixir)

---

## Contributing

Each microservice has its own contribution guidelines. Please refer to their respective repositories for:
- Code style and formatting rules
- Testing requirements
- Pull request process
- Architecture decision records (ADRs)

---

## Troubleshooting

### Proxy not responding
```bash
# Check proxy container logs
podman logs proxy

# Verify proxy is listening on correct port
podman ps | grep proxy

# Test proxy health (should reach one of the services)
curl http://localhost:9000/auth/health
```

**Common issues:**
- Ensure `PROXY_HOST` and `PROXY_PORT` are correctly set in `.env`
- Verify `config/services.json` exists and is valid JSON
- Check that backend services are running before starting proxy

### Services not starting
```bash
# Check container logs
podman logs proxy
podman logs auth-service
podman logs core-service
podman logs messaging-service

# Check infrastructure logs
podman logs kafka-b
podman logs postgres
podman logs mongo
podman logs redis
```

### Database connection issues
- Verify PostgreSQL is running: `podman ps | grep postgres`
- Check credentials in `.env` match Docker Compose configuration
- Ensure `POSTGRES_HOST` matches container name (default: `postgres`)
- Database names should be: `auth_service`, `core_service`, `messaging_service`

### Kafka connectivity problems
- Wait for Kafka to fully initialize (~30 seconds)
- Verify `KAFKA_HOST=kafka-b` matches container name
- Check Kafka logs: `podman logs kafka-b`
- Kafka needs time to create topics on first startup

### Port conflicts
If ports are already in use, modify `.env`:
```env
PROXY_PORT=9001              # Change proxy port
BEAT_IDENTITY_SERVER=2001    # Change auth service port
BEAT_API_REST_PORT=3001      # Change core service port
BEAT_MESSASSING_SERVER=4001  # Change messaging service port
```

### Routing issues
If requests aren't reaching the correct service:
1. Check `config/services.json` for correct routing rules
2. Verify service names in Docker Compose match routing config
3. Check proxy logs for routing decisions: `podman logs proxy`
4. Ensure service prefixes match (e.g., `/auth`, `/core`, `/messaging`)

### Cannot access Swagger/API documentation
- **Through proxy**: Use `http://localhost:9000/auth/swagger/index.html`
- **Direct access**: Use `http://localhost:2000/swagger/index.html`
- Ensure services are fully started before accessing Swagger
- Check service logs if Swagger doesn't load

---

## License

This project is part of the Beat EcoProve ecosystem. Please refer to individual service repositories for licensing information.

---

## Support

For issues, questions, or contributions:
- Auth Service: https://github.com/BeatEcoprove/beat-ecoprove-auth/issues
- Core Service: https://github.com/BeatEcoprove/beat-ecoprove-core/issues
- Messaging Service: https://github.com/BeatEcoprove/beat-ecoprove-messaging/issues

---

**Built with sustainability in mind. Code with purpose.**
