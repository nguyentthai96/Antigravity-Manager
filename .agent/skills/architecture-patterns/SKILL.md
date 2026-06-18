---
name: architecture-patterns
description: "Deep-dive into Clean Architecture, Onion Architecture, and Hexagonal Architecture (Ports & Adapters). Pattern selection, layer mapping, Spring Boot project structure, and migration guides."
risk: safe
source: community
date_added: "2026-02-27"
---

# Architecture Patterns

Deep-dive reference for the three dominant backend architecture patterns. Covers theory, practical implementation with Spring Boot, and decision framework for pattern selection.

## Use this skill when

- Designing new backend systems from scratch
- Refactoring monolithic applications for better maintainability
- Establishing architecture standards for your team
- Choosing between Clean, Onion, or Hexagonal architecture
- Mapping architecture layers to Spring Boot project structure
- Planning microservices decomposition

## Do not use this skill when

- You need small, localized refactors (use `refactoring` rules)
- The task is frontend-only without backend architecture changes
- You need architecture documentation/visualization (use `software-architecture`)

---

## Pattern Overview

### Comparison Table

| Aspect | Clean Architecture | Onion Architecture | Hexagonal Architecture |
|--------|-------------------|-------------------|----------------------|
| **Creator** | Robert C. Martin (2012) | Jeffrey Palermo (2008) | Alistair Cockburn (2005) |
| **Core idea** | Dependency Rule inward | Domain at center, infra at edges | Ports & Adapters |
| **Layers** | Entities → Use Cases → Interface Adapters → Frameworks | Domain → Domain Services → Application → Infrastructure | Application ↔ Ports ↔ Adapters |
| **Key principle** | Dependencies point inward | Inner layers never reference outer | App is symmetric: driving & driven sides |
| **Best for** | Complex business logic + multiple UIs | Domain-heavy applications | Integration-heavy systems |
| **Spring Boot fit** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## Clean Architecture (Uncle Bob)

### Dependency Rule

> Source code dependencies can only point **inward**. Nothing in an inner circle can know anything about an outer circle.

### Layer Structure

```
┌──────────────────────────────────────┐
│         Frameworks & Drivers         │  ← Spring Boot, JPA, Web
│  ┌──────────────────────────────┐    │
│  │      Interface Adapters      │    │  ← Controllers, Gateways, Presenters
│  │  ┌──────────────────────┐    │    │
│  │  │      Use Cases       │    │    │  ← Application business rules
│  │  │  ┌──────────────┐    │    │    │
│  │  │  │   Entities   │    │    │    │  ← Enterprise business rules
│  │  │  └──────────────┘    │    │    │
│  │  └──────────────────────┘    │    │
│  └──────────────────────────────┘    │
└──────────────────────────────────────┘
```

### Spring Boot Mapping

```
src/main/java/com/example/
├── domain/                          # Entities (inner circle)
│   ├── model/
│   │   ├── User.java                # Domain entity (POJO, no JPA)
│   │   └── Order.java
│   ├── repository/                  # Repository INTERFACES only
│   │   └── UserRepository.java      # Port (interface)
│   └── exception/
│       └── UserNotFoundException.java
│
├── application/                     # Use Cases
│   ├── usecase/
│   │   ├── CreateUserUseCase.java
│   │   └── GetOrderUseCase.java
│   ├── port/
│   │   ├── in/                      # Driving ports (input)
│   │   │   └── CreateUserCommand.java
│   │   └── out/                     # Driven ports (output)
│   │       └── NotificationPort.java
│   └── service/
│       └── UserApplicationService.java
│
├── adapter/                         # Interface Adapters
│   ├── in/
│   │   ├── web/                     # Controllers
│   │   │   ├── UserController.java
│   │   │   ├── dto/
│   │   │   │   ├── UserCreateRequest.java
│   │   │   │   └── UserResponse.java
│   │   │   └── mapper/
│   │   │       └── UserMapper.java
│   │   └── messaging/              # Message listeners
│   │       └── OrderEventListener.java
│   └── out/
│       ├── persistence/            # JPA implementation
│       │   ├── JpaUserRepository.java   # Implements domain repository
│       │   ├── UserJpaEntity.java       # JPA entity (separate from domain)
│       │   └── UserJpaMapper.java
│       └── notification/
│           └── EmailNotificationAdapter.java
│
└── config/                          # Frameworks & Drivers
    ├── SecurityConfig.java
    └── JpaConfig.java
```

### Key Rules

1. **Domain** has ZERO framework imports (no `@Entity`, no `@Service`)
2. **Use Cases** depend only on domain interfaces
3. **Adapters** implement domain interfaces
4. **Config** wires everything together via Spring DI

---

## Onion Architecture

### Core Principle

> The domain model is at the center. Each outer layer can depend on layers closer to the center, but NOT on layers further from the center.

### Layer Structure

```
┌─────────────────────────────────────┐
│          Infrastructure             │  ← DB, APIs, UI, Messaging
│  ┌─────────────────────────────┐    │
│  │     Application Services    │    │  ← Orchestration, Commands
│  │  ┌─────────────────────┐    │    │
│  │  │   Domain Services   │    │    │  ← Business logic operations
│  │  │  ┌─────────────┐    │    │    │
│  │  │  │ Domain Model │    │    │    │  ← Entities, Value Objects
│  │  │  └─────────────┘    │    │    │
│  │  └─────────────────────┘    │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

### Spring Boot Mapping

```
src/main/java/com/example/
├── domain/
│   ├── model/          # Domain Model (center)
│   │   ├── User.java
│   │   └── Money.java  # Value Object
│   ├── service/        # Domain Services
│   │   └── PricingService.java
│   └── repository/     # Repository interfaces
│       └── UserRepository.java
│
├── application/        # Application Services
│   ├── UserService.java
│   └── dto/
│       └── UserDto.java
│
└── infrastructure/     # Infrastructure (outermost)
    ├── persistence/
    │   └── JpaUserRepository.java
    ├── web/
    │   └── UserController.java
    └── config/
        └── AppConfig.java
```

### vs Clean Architecture

- Simpler naming (no "use case" / "adapter" terminology)
- More intuitive for teams new to layered architecture
- Less rigid about port/adapter symmetry

---

## Hexagonal Architecture (Ports & Adapters)

### Core Principle

> The application is a hexagon. The inside contains business logic. The outside connects to the world through **ports** (interfaces) and **adapters** (implementations).

### Structure

```
          ┌─────────────┐
   HTTP ──┤  Driving     │
   CLI  ──┤  Adapters    │
   MQ   ──┤  (Primary)   │
          │   ┌─────┐    │
          │   │ App │    │
          │   │Logic│    │
          │   └─────┘    │
          │  Driven      ├── Database
          │  Adapters    ├── Email
          │  (Secondary) ├── External API
          └─────────────┘
```

### Spring Boot Mapping

```
src/main/java/com/example/
├── application/
│   ├── domain/
│   │   ├── User.java
│   │   └── UserService.java   # Core business logic
│   ├── port/
│   │   ├── driving/           # Primary ports (input)
│   │   │   ├── CreateUserPort.java
│   │   │   └── GetUserPort.java
│   │   └── driven/            # Secondary ports (output)
│   │       ├── UserPersistencePort.java
│   │       └── NotificationPort.java
│   └── service/
│       └── UserServiceImpl.java  # Implements driving ports
│
├── adapter/
│   ├── driving/               # Primary adapters
│   │   ├── rest/
│   │   │   └── UserRestAdapter.java  # @RestController
│   │   └── messaging/
│   │       └── UserKafkaAdapter.java # @KafkaListener
│   └── driven/                # Secondary adapters
│       ├── persistence/
│       │   └── UserJpaAdapter.java   # Implements UserPersistencePort
│       └── notification/
│           └── EmailAdapter.java     # Implements NotificationPort
│
└── config/
    └── BeanConfig.java
```

### Key Advantage

- **Symmetry**: The app doesn't know if it's driven by HTTP, CLI, tests, or message queue
- **Testability**: Swap any adapter with a mock/stub
- Best fit for **integration-heavy** Spring Boot microservices

---

## Decision Framework

```
START
  │
  ├── Complex domain logic? ──YES──→ Clean Architecture or Onion
  │                                    │
  │                                    ├── Team familiar with DDD? ──YES──→ Clean Architecture
  │                                    └── Team new to layered arch? ──YES──→ Onion (simpler naming)
  │
  ├── Many integrations (APIs, MQ, DBs)? ──YES──→ Hexagonal Architecture
  │
  ├── Simple CRUD service? ──YES──→ Standard layered (Controller → Service → Repository)
  │
  └── Unsure? ──→ Start with Hexagonal (easiest to refactor later)
```

---

## Anti-Patterns

- ❌ Domain entities with `@Entity` / `@Table` annotations (leaking infrastructure into domain)
- ❌ Business logic in controllers
- ❌ Use cases depending on specific frameworks (Spring, JPA)
- ❌ Skipping the port layer (coupling application to adapters directly)
- ❌ Mapping everything everywhere (over-engineering for simple CRUDs)
- ❌ Circular dependencies between layers

## References

- `skills/domain-driven-design/` — DDD tactical/strategic patterns
- `skills/senior-architect/` — Comprehensive architecture toolkit
- `skills/software-architecture/` — Architecture documentation & visualization
- `rules/spring-boot.md` — Project-specific Spring Boot rules
