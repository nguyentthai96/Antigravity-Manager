# Spring Microservices Architecture Patterns

Patterns for building microservices architectures with Spring Boot and Spring Cloud.

---

## Service Decomposition

### Strategies

| Strategy | Description |
|---|---|
| Business Capability | Organize services by business function (Product, Order, Payment) |
| Domain-Driven Design | Use DDD bounded contexts to identify service boundaries |
| Data Ownership | Each service owns its data — no shared databases |
| Team Structure | Align service boundaries with team boundaries (Conway's Law) |

### Principles

- Single Responsibility Principle per service
- High cohesion within service, low coupling between services
- Independent deployment and scaling
- Technology diversity allowed per service
- API-first design
- Fail fast and degrade gracefully

### Best Practices

- Start with larger services, split as needed (don't over-microservice)
- Identify clear service boundaries using domain modeling
- Avoid shared databases — each service has its own
- Define clear API contracts with OpenAPI/Swagger
- Document service responsibilities and SLAs
- Use event-driven communication for loose coupling
- Implement idempotency for distributed operations

---

## API Gateway

### Spring Cloud Gateway

Reactive API gateway with routing, load balancing, circuit breaker integration, rate limiting, and request/response transformation.

```properties
# application.properties
spring.cloud.gateway.routes[0].id=product-service
spring.cloud.gateway.routes[0].uri=lb://product-service
spring.cloud.gateway.routes[0].predicates[0]=Path=/api/v1/products/**
```

### Gateway Patterns

| Pattern | Description |
|---|---|
| Routing | Route requests to appropriate backend services |
| Aggregation | Combine multiple service responses into single response |
| Transformation | Transform requests/responses (headers, body) |
| Authentication | Centralized authentication and authorization |
| Rate Limiting | Control request rate per client/service |
| Caching | Cache responses to reduce backend load |
| Circuit Breaker | Protect against downstream service failures |
| Retry | Retry failed requests with exponential backoff |

### Best Practices

- Use Spring Cloud Gateway for Spring Boot applications
- Implement health checks for routing decisions
- Add correlation IDs for request tracing
- Implement request/response logging
- Use circuit breaker for downstream services
- Cache static responses
- Implement rate limiting per client
- Use load balancing for service instances

---

## Service Discovery

### Implementations

| Technology | Use Case |
|---|---|
| Eureka | Spring Cloud Netflix ecosystem |
| Consul | Service mesh features, multi-datacenter |
| Kubernetes | K8s native DNS-based discovery |
| Zookeeper | Existing Zookeeper infrastructure |

### Discovery Patterns

| Pattern | Description |
|---|---|
| Client-side discovery | Client queries service registry to find instances |
| Server-side discovery | Load balancer queries registry and routes requests |
| Self-registration | Service registers itself with registry |
| Third-party registration | Service registrar handles registration |

### Best Practices

- Use Eureka for Spring Cloud applications
- Use Kubernetes Service discovery when on K8s
- Implement health checks for service status
- Configure proper lease renewal intervals
- Use service names instead of hardcoded URLs
- Implement retry logic for service discovery
- Monitor service registry health

---

## Config Management

### Implementations

| Technology | Use Case | Configuration |
|---|---|---|
| Spring Cloud Config | Centralized config with Git backend | `spring.cloud.config.uri` |
| Kubernetes ConfigMap | K8s native config | `configMapRef` in deployment |
| HashiCorp Vault | Advanced secret management | `spring.cloud.vault.*` |

### Patterns

| Pattern | Description |
|---|---|
| Centralized config | Single source of truth for configuration |
| Environment-specific | Different configs per environment (dev, prod) |
| Refresh mechanism | `@RefreshScope` for dynamic config updates |
| Secret management | Secure storage of sensitive configuration |
| Version control | Config stored in Git repository |

### Best Practices

- Use Spring Cloud Config for centralized config
- Store config in version control (Git)
- Use `@RefreshScope` for dynamic updates
- Separate secrets from regular config
- Use environment-specific profiles
- Implement config encryption for sensitive data
- Monitor config server health

---

## Circuit Breaker

### Resilience4j

Lightweight fault tolerance library for Spring Boot — circuit breaker, retry, rate limiter, bulkhead, and time limiter.

```properties
# application.properties
resilience4j.circuitbreaker.instances.productService.slidingWindowSize=10
resilience4j.circuitbreaker.instances.productService.failureRateThreshold=50
resilience4j.circuitbreaker.instances.productService.waitDurationInOpenState=10s
```

### Circuit Breaker States

| State | Behavior |
|---|---|
| **Closed** | Normal operation, requests pass through |
| **Open** | Circuit open, requests fail fast |
| **Half-Open** | Testing if service recovered, limited requests allowed |

### Resilience Patterns

| Pattern | Description |
|---|---|
| Circuit Breaker | Open circuit when failure threshold reached |
| Fallback | Return default response when circuit is open |
| Retry | Retry failed requests with exponential backoff |
| Timeout | Fail fast when service doesn't respond |
| Bulkhead | Isolate resources to prevent resource exhaustion |

### Best Practices

- Use Resilience4j for Spring Boot 3+
- Configure appropriate failure thresholds
- Implement fallback mechanisms
- Monitor circuit breaker metrics
- Use retry with circuit breaker
- Set proper timeout values
- Implement bulkhead pattern for resource isolation
- Log circuit state changes

---

## Distributed Tracing

### Implementations

| Technology | Use Case | Configuration |
|---|---|---|
| Micrometer Tracing | Spring-native tracing | `management.tracing.*` |
| Zipkin | Tracing visualization | `management.zipkin.tracing.endpoint` |
| Jaeger | High-performance tracing | `management.tracing.export.jaeger.*` |
| OpenTelemetry | Vendor-neutral, multi-language | OpenTelemetry SDK |

### Trace Concepts

| Concept | Description |
|---|---|
| Trace | Complete request path across services |
| Span | Individual operation within a trace |
| Correlation ID | Unique ID propagated across services |
| Baggage | Context data propagated with trace |
| Sampling | Reduce trace volume in high-throughput systems |

### Best Practices

- Use Micrometer Tracing with Spring Boot
- Propagate trace context across service calls
- Add custom spans for business operations
- Use correlation IDs in logs
- Configure appropriate sampling rates
- Monitor trace volume and performance
- Use Zipkin or Jaeger for visualization
- Add trace context to error messages

---

## Event-Driven Architecture

### Implementations

| Technology | Use Case | Configuration |
|---|---|---|
| Spring Cloud Stream | Event-driven abstraction | Multiple binders |
| Apache Kafka | High-throughput streaming | `spring.kafka.*` |
| RabbitMQ | Flexible routing patterns | `spring.rabbitmq.*` |

### Event Patterns

| Pattern | Description |
|---|---|
| Event Sourcing | Store state as sequence of events |
| CQRS | Separate read and write models |
| Saga | Distributed transaction pattern using events |
| Publish/Subscribe | One-to-many event distribution |
| Event Replay | Replay events to rebuild state |
| Idempotency | Handle duplicate events safely |

### Best Practices

- Use Spring Cloud Stream for abstraction
- Use Kafka for high-throughput scenarios
- Implement idempotent event handlers
- Use consumer groups for load balancing
- Implement event versioning
- Handle event ordering when needed
- Monitor event processing lag
- Implement dead letter queues for failed events
- Use schema registry for event schemas

---

## Saga Pattern

### Orchestration vs Choreography

| Type | When to Use | Description |
|---|---|---|
| Orchestration | Complex workflows, centralized control | Central orchestrator coordinates saga |
| Choreography | Simple workflows, loose coupling | Services coordinate through events |

### Saga Concepts

| Concept | Description |
|---|---|
| Compensating transaction | Undo previous operations on failure |
| Saga step | Individual operation in saga |
| Saga coordinator | Orchestrates saga execution |
| Compensation | Rollback operation for each step |

### Best Practices

- Use orchestration for complex workflows
- Use choreography for simple workflows
- Implement idempotent compensation
- Log all saga steps for debugging
- Handle partial failures gracefully
- Set timeouts for saga steps
- Monitor saga completion rates

---

## CQRS

### Components

| Side | Purpose | Optimization |
|---|---|---|
| Command | Write operations | Optimized for writes |
| Query | Read operations | Optimized for reads (denormalized views) |

### Use Cases

- High read/write ratio
- Different read and write requirements
- Need to scale reads independently
- Complex query requirements

### Best Practices

- Use CQRS when read/write patterns differ significantly
- Keep command and query models separate
- Use events to sync read models
- Accept eventual consistency
- Monitor read model lag
- Implement read model rebuild capability

---

## API Versioning

### Strategies

| Strategy | Example |
|---|---|
| URL versioning | `/api/v1/products` |
| Header versioning | `Accept: application/vnd.api.v1+json` |
| Query parameter | `?version=1` |
| Content negotiation | Via `Accept` header |

### Best Practices

- Use URL versioning for REST APIs
- Maintain backward compatibility when possible
- Deprecate old versions gradually
- Document versioning strategy
- Monitor version usage

---

## Distributed Caching

### Implementations

| Technology | Use Case | Configuration |
|---|---|---|
| Redis | Distributed cache | `spring.data.redis.*` |
| Hazelcast | In-memory data grid | Hazelcast config |
| Caffeine | Single instance local cache | Caffeine builder |

### Caching Patterns

| Pattern | Description |
|---|---|
| Cache-Aside | Application manages cache |
| Write-Through | Write to cache and database |
| Write-Behind | Write to cache, async to database |
| Cache Invalidation | Invalidate cache on updates |

### Best Practices

- Use Redis for distributed caching
- Implement cache invalidation strategy
- Set appropriate TTL values
- Monitor cache hit rates
- Use cache keys with service prefix
- Handle cache failures gracefully
- Consider cache warming on startup

---

## Service Mesh

### Implementations

| Technology | Use Case |
|---|---|
| Istio | Kubernetes, advanced traffic management features |
| Linkerd | Lightweight, simple, high performance |

### Benefits

- Traffic management without code changes
- Automatic mTLS
- Observability out of the box
- Policy enforcement

### Best Practices

- Consider service mesh for complex deployments
- Use Istio for Kubernetes
- Start with basic features
- Monitor mesh performance
- Use for cross-cutting concerns
