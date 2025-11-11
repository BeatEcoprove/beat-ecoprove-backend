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

### Microservices Overview

The Beat EcoProve backend follows a **polyglot microservices architecture** with event-driven communication, where each service is built using the technology stack best suited for its domain responsibilities.

```
┌─────────────────────────────────────────────────────────────────┐
│                        Client Applications                       │
└────────────┬────────────────────┬────────────────────┬──────────┘
             │                    │                    │
             ▼                    ▼                    ▼
      ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
      │ Auth Service │    │ Core Service │    │   Messaging  │
      │   (Port      │    │   (Port      │    │   Service    │
      │    2000)     │    │    3000)     │    │  (Port 4000) │
      │              │    │              │    │              │
      │   Go/Fiber   │    │  .NET 9.0    │    │    Elixir    │
      └──────┬───────┘    └──────┬───────┘    └──────┬───────┘
             │                   │                    │
             │         ┌─────────┴─────────┐          │
             │         │                   │          │
             └─────────┤   Kafka Broker    ├──────────┘
                       │   (Port 9092)     │
                       └─────────┬─────────┘
                                 │
             ┌───────────────────┼───────────────────┐
             │                   │                   │
             ▼                   ▼                   ▼
      ┌──────────┐        ┌──────────┐       ┌──────────┐
      │PostgreSQL│        │  Redis   │       │ MongoDB  │
      │ (5432)   │        │  (6379)  │       │ (27017)  │
      └──────────┘        └──────────┘       └──────────┘
```

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

Each service maintains its own PostgreSQL database following the **Database per Service** pattern:
- `identity-db`: Authentication and authorization data
- `core-db`: Business domain entities and relationships
- `messaging-db`: Group structures and user relationships

**PostGIS Extension:** Enabled in Core Service for geospatial queries (finding nearby stores, location-based recommendations).

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
| Auth      | Go       | Fiber            | 1.23.2  |
| Core      | C#       | ASP.NET Core     | 9.0     |
| Messaging | Elixir   | Phoenix          | 1.18.0  |

### Infrastructure
| Component  | Technology       | Purpose                           |
|-----------|------------------|-----------------------------------|
| Event Bus | Apache Kafka 4.0 | Asynchronous event streaming      |
| Database  | PostgreSQL 17    | Primary relational data store     |
| Database  | MongoDB 8.0      | Document store for messages       |
| Cache     | Redis 7.x        | Distributed caching & sessions    |
| Gateway   | NGINX            | Reverse proxy (optional)          |

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

   **Development mode** (with live reload and debugging):
   ```bash
   just dev
   ```

   **Production mode** (optimized builds):
   ```bash
   just prod
   ```

   Alternatively, use Docker Compose directly:
   ```bash
   # Development
   podman compose -f docker-compose.yml -f docker-compose.dev.yml up -d

   # Production
   podman compose -f docker-compose.yml -f docker-compose.prod.yml up -d
   ```

5. **Verify services are running**
   ```bash
   podman ps
   ```

   You should see containers for:
   - `auth-service`
   - `core-service`
   - `messaging-service`
   - `kafka-b`
   - `postgres`
   - `mongo`

---

## API Documentation

Each service exposes interactive Swagger/OpenAPI documentation:

| Service   | Swagger URL                              | Description                          |
|-----------|------------------------------------------|--------------------------------------|
| Auth      | http://localhost:2000/swagger/index.html | Authentication & authorization APIs  |
| Core      | http://localhost:3000/swagger            | Business logic & domain APIs         |
| Messaging | http://localhost:4000/api/swagger        | Real-time messaging & notifications  |

---

## Development Workflow

### Building Individual Services

```bash
# Build specific service
nix run .#auth-service
nix run .#core-service
nix run .#messaging-service

# Build all services
nix run .#build-all
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
POSTGRES_DB=identity-db
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
├── docker/                  # Docker configuration files
│   └── postgres/           # PostgreSQL initialization scripts
├── .env                    # Environment configuration
├── .env.example           # Example environment template
├── docker-compose.yml     # Base compose configuration
├── docker-compose.dev.yml # Development overrides
├── docker-compose.prod.yml# Production overrides
├── flake.nix              # Nix flake for reproducible builds
├── flake.lock             # Locked dependency versions
├── services.nix           # Service build configuration
├── Justfile               # Task automation commands
└── README.md              # This file
```

**Microservice Repositories (External):**
- [beat-ecoprove-auth](https://github.com/BeatEcoprove/beat-ecoprove-auth) - Identity & authentication
- [beat-ecoprove-core](https://github.com/BeatEcoprove/beat-ecoprove-core) - Core business logic
- [beat-ecoprove-messaging](https://github.com/BeatEcoprove/beat-ecoprove-messaging) - Real-time messaging

---

## Contributing

Each microservice has its own contribution guidelines. Please refer to their respective repositories for:
- Code style and formatting rules
- Testing requirements
- Pull request process
- Architecture decision records (ADRs)

---

## Troubleshooting

### Services not starting
```bash
# Check container logs
podman logs auth-service
podman logs core-service
podman logs messaging-service

# Check infrastructure logs
podman logs kafka-b
podman logs postgres
podman logs mongo
```

### Database connection issues
- Verify PostgreSQL is running: `podman ps | grep postgres`
- Check credentials in `.env` match Docker Compose configuration
- Ensure `POSTGRES_HOST` matches container name (default: `postgres`)

### Kafka connectivity problems
- Wait for Kafka to fully initialize (~30 seconds)
- Verify `KAFKA_HOST=kafka-b` matches container name
- Check Kafka logs: `podman logs kafka-b`

### Port conflicts
If ports are already in use, modify `.env`:
```env
BEAT_IDENTITY_SERVER=2001
BEAT_API_REST_PORT=3001
BEAT_MESSASSING_SERVER=4001
```

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
