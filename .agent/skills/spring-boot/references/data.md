# Data Access — Spring Data JPA & PostgreSQL

## Database Defaults

- **Engine:** PostgreSQL 17 (configure version in `versions.json`)
- **Schema management:** Hibernate `ddl-auto` — schema derived from `@Entity` classes. Do **not** use Flyway or Liquibase
- **Driver:** `org.postgresql:postgresql` (bundled via start.spring.io)
- **Testcontainers:** `postgres:17-alpine` images
- **Pool:** HikariCP (Spring Boot default)

---

## Database Configuration

`src/main/resources/application.properties`:
```properties
# Datasource
spring.datasource.url=${SPRING_DATASOURCE_URL:jdbc:postgresql://localhost:5432/mydb}
spring.datasource.username=${SPRING_DATASOURCE_USERNAME:user}
spring.datasource.password=${SPRING_DATASOURCE_PASSWORD:password}
spring.datasource.driver-class-name=org.postgresql.Driver

# JPA / Hibernate
spring.jpa.hibernate.ddl-auto=update
spring.jpa.show-sql=false
spring.jpa.properties.hibernate.format_sql=false
spring.jpa.open-in-view=false
```

> **Profiles:** Use `spring.jpa.hibernate.ddl-auto=update` in dev, `validate` in prod. See [CONFIGURATION.md](CONFIGURATION.md) for profile management and secrets handling.

### Hibernate DDL Auto Modes

| Mode | Behavior | Use when |
|------|----------|----------|
| `update` | Creates/alters tables to match entities. Never drops. | **Development** (default) |
| `validate` | Only validates schema matches entities. Fails on mismatch. | **Production** |
| `create` | Drops and recreates schema on startup. | Testing |
| `create-drop` | Like `create`, but also drops on shutdown. | Unit tests |
| `none` | Hibernate does nothing. | Manual schema management |

> Hibernate derives DDL from `@Column`, `@Table`, `@Index`, and other JPA annotations. Keep entities well-annotated for accurate schema generation.
>
> **Tip:** Enable `spring.jpa.show-sql=true` and `spring.jpa.properties.hibernate.format_sql=true` in `application-dev.properties` for development debugging.

---

## JPA Entity Pattern

```java
@Entity
@Table(name = "users", indexes = {
    @Index(name = "idx_email", columnList = "email", unique = true),
    @Index(name = "idx_username", columnList = "username")
})
@EntityListeners(AuditingEntityListener.class)
public class User {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(nullable = false, unique = true, length = 100)
    private String email;

    @Column(nullable = false, length = 100)
    private String password;

    @Column(nullable = false, unique = true, length = 50)
    private String username;

    @Column(nullable = false)
    private Boolean active = true;

    @OneToMany(mappedBy = "user", cascade = CascadeType.ALL, orphanRemoval = true)
    private List<Address> addresses = new ArrayList<>();

    @ManyToMany
    @JoinTable(
        name = "user_roles",
        joinColumns = @JoinColumn(name = "user_id"),
        inverseJoinColumns = @JoinColumn(name = "role_id")
    )
    private Set<Role> roles = new HashSet<>();

    @CreatedDate
    @Column(nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @LastModifiedDate
    @Column(nullable = false)
    private LocalDateTime updatedAt;

    @Version
    private Long version;

    // Constructors
    public User() {}

    // Getters, Setters, helper methods for bidirectional relationships
    public void addAddress(Address address) {
        addresses.add(address);
        address.setUser(this);
    }

    public void removeAddress(Address address) {
        addresses.remove(address);
        address.setUser(null);
    }
}
```

---

## DDD Entity Patterns (Advanced)

> Advanced patterns for Domain-Driven Design projects. Uses Value Objects for IDs and embedded types.
> These patterns complement the basic entity pattern above.

### TSID Identity Generator

```xml
<dependency>
    <groupId>io.hypersistence</groupId>
    <artifactId>hypersistence-utils-hibernate-71</artifactId>
    <version>3.14.1</version>
</dependency>
```

```java
import io.hypersistence.tsid.TSID;

public class IdGenerator {
    private IdGenerator() {}

    public static String generateString() {
        return TSID.Factory.getTsid().toString();
    }
}
```

### Value Object for Primary Key

```java
public record UserId(String id) {
    public UserId {
        if (id == null || id.trim().isBlank()) {
            throw new IllegalArgumentException("User id cannot be null or empty");
        }
    }

    public static UserId of(String id) {
        return new UserId(id);
    }

    public static UserId generate() {
        return new UserId(IdGenerator.generateString());
    }
}
```

### BaseEntity with Auditing

```java
@MappedSuperclass
@EntityListeners(AuditingEntityListener.class)
public abstract class BaseEntity {

    @Column(name = "created_at", nullable = false, updatable = false)
    @CreatedDate
    protected Instant createdAt;

    @Column(name = "updated_at", nullable = false)
    @LastModifiedDate
    protected Instant updatedAt;

    @Version
    private int version;

    public Instant getCreatedAt() { return createdAt; }
    public Instant getUpdatedAt() { return updatedAt; }
}
```

### Entity with Embedded Value Objects

```java
@Entity
@Table(name = "users")
class UserEntity extends BaseEntity {

    @EmbeddedId
    @AttributeOverride(name = "id", column = @Column(name = "id", nullable = false))
    private UserId id;

    @Embedded
    @AttributeOverrides({
        @AttributeOverride(name = "addrLine1", column = @Column(name = "addr_line1", nullable = false)),
        @AttributeOverride(name = "addrLine2", column = @Column(name = "addr_line2")),
        @AttributeOverride(name = "city", column = @Column(name = "city"))
    })
    private Address address;

    @Enumerated(EnumType.STRING)
    @Column(name = "role", nullable = false)
    private Role role;

    protected UserEntity() {} // JPA

    public UserEntity(UserId id, Address address, Role role) {
        this.id = AssertUtil.requireNotNull(id, "User id cannot be null");
        this.address = AssertUtil.requireNotNull(address, "Address cannot be null");
        this.role = AssertUtil.requireNotNull(role, "Role cannot be null");
    }

    // Factory method for creating new entities
    public static UserEntity create(Address address, Role role) {
        return new UserEntity(UserId.generate(), address, role);
    }

    public boolean isAdmin() {
        return role == Role.ROLE_ADMIN;
    }
}
```

### AssertUtil

```java
public class AssertUtil {
    private AssertUtil() {}

    public static <T> T requireNotNull(T obj, String message) {
        if (obj == null)
            throw new IllegalArgumentException(message);
        return obj;
    }
}
```

### DDD Repository with Default Methods

```java
interface UserRepository extends JpaRepository<UserEntity, UserId> {

    Optional<UserEntity> findByEmail(@Param("email") String email);

    // Convenience methods using default interface methods
    default UserEntity getByEmail(String email) {
        return this.findByEmail(email)
                .orElseThrow(() -> new ResourceNotFoundException(
                    "User not found with email: " + email));
    }
}
```

> **Key DDD principles:**
> - Create repositories **only for aggregate roots**
> - Use `@EmbeddedId` with Value Objects for type-safe IDs
> - Validate state in constructors — throw for invalid inputs
> - Add **domain methods** that operate on entity state
> - Use **factory methods** for entity creation

---

## Spring Data JPA Repository

```java
@Repository
public interface UserRepository extends JpaRepository<User, Long>,
                                       JpaSpecificationExecutor<User> {

    Optional<User> findByEmail(String email);

    Optional<User> findByUsername(String username);

    boolean existsByEmail(String email);

    boolean existsByUsername(String username);

    @Query("SELECT u FROM User u LEFT JOIN FETCH u.roles WHERE u.email = :email")
    Optional<User> findByEmailWithRoles(@Param("email") String email);

    @Query("SELECT u FROM User u WHERE u.active = true AND u.createdAt >= :since")
    List<User> findActiveUsersSince(@Param("since") LocalDateTime since);

    @Modifying
    @Query("UPDATE User u SET u.active = false WHERE u.lastLoginAt < :threshold")
    int deactivateInactiveUsers(@Param("threshold") LocalDateTime threshold);

    // Projection for read-only DTOs
    @Query("SELECT new com.example.dto.UserSummary(u.id, u.username, u.email) " +
           "FROM User u WHERE u.active = true")
    List<UserSummary> findAllActiveSummaries();
}
```

---

## Repository with Specifications

```java
public class UserSpecifications {

    public static Specification<User> hasEmail(String email) {
        return (root, query, cb) ->
            email == null ? null : cb.equal(root.get("email"), email);
    }

    public static Specification<User> isActive() {
        return (root, query, cb) -> cb.isTrue(root.get("active"));
    }

    public static Specification<User> createdAfter(LocalDateTime date) {
        return (root, query, cb) ->
            date == null ? null : cb.greaterThanOrEqualTo(root.get("createdAt"), date);
    }

    public static Specification<User> hasRole(String roleName) {
        return (root, query, cb) -> {
            Join<User, Role> roles = root.join("roles", JoinType.INNER);
            return cb.equal(roles.get("name"), roleName);
        };
    }
}

// Usage in service
@Service
public class UserService {
    private final UserRepository userRepository;

    public UserService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    public Page<User> searchUsers(UserSearchCriteria criteria, Pageable pageable) {
        Specification<User> spec = Specification
            .where(UserSpecifications.hasEmail(criteria.email()))
            .and(UserSpecifications.isActive())
            .and(UserSpecifications.createdAfter(criteria.createdAfter()));

        return userRepository.findAll(spec, pageable);
    }
}
```

---

## Transaction Management

```java
@Service
@Transactional(readOnly = true)
public class OrderService {
    private final OrderRepository orderRepository;
    private final PaymentService paymentService;
    private final InventoryService inventoryService;
    private final NotificationService notificationService;

    public OrderService(OrderRepository orderRepository, PaymentService paymentService,
                        InventoryService inventoryService, NotificationService notificationService) {
        this.orderRepository = orderRepository;
        this.paymentService = paymentService;
        this.inventoryService = inventoryService;
        this.notificationService = notificationService;
    }

    @Transactional
    public Order createOrder(OrderCreateRequest request) {
        Order order = Order.builder()
            .customerId(request.customerId())
            .status(OrderStatus.PENDING)
            .build();

        request.items().forEach(item -> {
            inventoryService.reserveStock(item.productId(), item.quantity());
            order.addItem(item);
        });

        order = orderRepository.save(order);

        try {
            paymentService.processPayment(order);
            order.setStatus(OrderStatus.PAID);
        } catch (PaymentException e) {
            order.setStatus(OrderStatus.PAYMENT_FAILED);
            throw e; // Transaction will rollback
        }

        return orderRepository.save(order);
    }

    @Transactional(propagation = Propagation.REQUIRES_NEW)
    public void logOrderEvent(Long orderId, String event) {
        // Separate transaction — will commit even if parent rolls back
        OrderEvent orderEvent = new OrderEvent(orderId, event);
        orderEventRepository.save(orderEvent);
    }

    @Transactional(noRollbackFor = NotificationException.class)
    public void completeOrder(Long orderId) {
        Order order = orderRepository.findById(orderId)
            .orElseThrow(() -> new ResourceNotFoundException("Order not found"));

        order.setStatus(OrderStatus.COMPLETED);
        orderRepository.save(order);

        // Won't rollback transaction if notification fails
        try {
            notificationService.sendCompletionEmail(order);
        } catch (NotificationException e) {
            log.error("Failed to send notification for order {}", orderId, e);
        }
    }
}
```

---

## Auditing Configuration

```java
@Configuration
@EnableJpaAuditing
public class JpaAuditingConfig {

    @Bean
    public AuditorAware<String> auditorProvider() {
        return () -> {
            Authentication authentication = SecurityContextHolder
                .getContext()
                .getAuthentication();

            if (authentication == null || !authentication.isAuthenticated()) {
                return Optional.of("system");
            }

            return Optional.of(authentication.getName());
        };
    }
}

@MappedSuperclass
@EntityListeners(AuditingEntityListener.class)
public abstract class AuditableEntity {

    @CreatedDate
    @Column(nullable = false, updatable = false)
    private LocalDateTime createdAt;

    @CreatedBy
    @Column(nullable = false, updatable = false, length = 100)
    private String createdBy;

    @LastModifiedDate
    @Column(nullable = false)
    private LocalDateTime updatedAt;

    @LastModifiedBy
    @Column(nullable = false, length = 100)
    private String updatedBy;

    // Getters/setters
}
```

---

## Projections

```java
// Interface-based projection
public interface UserSummary {
    Long getId();
    String getUsername();
    String getEmail();

    @Value("#{target.firstName + ' ' + target.lastName}")
    String getFullName();
}

// Class-based projection (DTO)
public record UserSummaryDto(
    Long id,
    String username,
    String email
) {}

// Usage
public interface UserRepository extends JpaRepository<User, Long> {
    List<UserSummary> findAllBy();

    <T> List<T> findAllBy(Class<T> type);
}

// Service usage
List<UserSummary> summaries = userRepository.findAllBy();
List<UserSummaryDto> dtos = userRepository.findAllBy(UserSummaryDto.class);
```

---

## Performance Optimization

### Avoiding N+1 Queries

The most common JPA performance issue. Use `JOIN FETCH` or `@EntityGraph`:

```java
// BAD: triggers N+1 — one query per order's items
List<Order> orders = orderRepository.findAll();
orders.forEach(o -> o.getItems().size()); // N extra queries

// GOOD: single query with JOIN FETCH
@Query("SELECT o FROM Order o JOIN FETCH o.items WHERE o.status = :status")
List<Order> findByStatusWithItems(@Param("status") String status);

// GOOD: declarative with @EntityGraph
@EntityGraph(attributePaths = {"items"})
List<Order> findByStatus(String status);
```

Enable Hibernate statistics in development to detect N+1 issues:
```properties
# Development only — disable in production
spring.jpa.properties.hibernate.generate_statistics=true
logging.level.org.hibernate.stat=DEBUG
```

### Pagination

Always paginate large result sets. Never use unbounded `findAll()` in production:

```java
// Repository
Page<AppUser> findByActiveTrue(Pageable pageable);

// Controller
@GetMapping("/users")
Page<AppUser> listUsers(@RequestParam(defaultValue = "0") int page,
                        @RequestParam(defaultValue = "20") int size) {
    return userRepository.findByActiveTrue(PageRequest.of(page, size, Sort.by("login")));
}
```

### Batch Operations

For bulk inserts/updates, configure Hibernate batching:

```properties
spring.jpa.properties.hibernate.jdbc.batch_size=25
spring.jpa.properties.hibernate.order_inserts=true
spring.jpa.properties.hibernate.order_updates=true
```

```java
// BAD: N individual INSERT statements
items.forEach(item -> repository.save(item));

// GOOD: batched — Hibernate groups INSERTs
repository.saveAll(items);
```

### Connection Pool Sizing (HikariCP)

```properties
# connections = (2 × CPU cores) + effective_spindle_count
# For a typical 4-core server with SSD:
spring.datasource.hikari.maximum-pool-size=10
spring.datasource.hikari.minimum-idle=5
spring.datasource.hikari.idle-timeout=300000
spring.datasource.hikari.max-lifetime=1800000
spring.datasource.hikari.connection-timeout=30000
```

### Caching

For read-heavy data, enable Hibernate second-level cache:

```properties
spring.jpa.properties.hibernate.cache.use_second_level_cache=true
spring.jpa.properties.hibernate.cache.region.factory_class=org.hibernate.cache.jcache.JCacheRegionFactory
```

```java
@Entity
@Cache(usage = CacheConcurrencyStrategy.READ_WRITE)
public class Country { ... }
```

For application-level caching, use Spring's `@Cacheable`:
```java
@Cacheable("users")
public AppUser findByLogin(String login) {
    return userRepository.findByLogin(login);
}
```

---

## Testcontainers Integration

```java
@TestConfiguration(proxyBeanMethods = false)
class TestcontainersConfiguration {  // package-private (Boot 4 requirement)
    @Bean
    @ServiceConnection
    PostgreSQLContainer postgresContainer() {
        return new PostgreSQLContainer("postgres:17-alpine")
            .withReuse(true);
    }
}
```

Use `@Import(TestcontainersConfiguration.class)` in integration tests.

---

## Docker Compose (Dev)

`compose.yaml` (used by `spring-boot-docker-compose`):
```yaml
services:
  postgres:
    image: postgres:17-alpine
    environment:
      POSTGRES_DB: mydb
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD", "pg_isready", "-U", "user"]
      interval: 10s
      timeout: 5s
      retries: 5
volumes:
  postgres_data:
```

---

## Production Tips

- **Pooling:** Use HikariCP defaults; tune `maximum-pool-size` and `connection-timeout`
- **Indexes:** Add indexes via JPA annotations (`@Index` in `@Table`) or manual DDL
- **Secrets:** Inject via environment variables or Vault/Key Vault; never commit plaintext
- **Schema Validation:** Keep `spring.jpa.hibernate.ddl-auto=validate` in prod
- **UTF-8:** Ensure DB encoding is UTF8 (default for official Postgres images)

---

## Local Developer Experience

- Enable `spring-boot-docker-compose` (Boot 3.1+) to auto-start `compose.yaml` on `./mvnw spring-boot:run`
- Provide `.env.sample` with placeholders: `SPRING_DATASOURCE_PASSWORD`, etc. (see [Project Setup](PROJECT-SETUP.md))

---

## Observability

- Expose Postgres metrics via `pg_stat_statements`; integrate with Micrometer if needed
- Consider **pgBouncer** for high-connection scenarios; document in ops runbook

---

## Troubleshooting

- Common error: `FATAL: password authentication failed` — verify `spring.datasource.*` and `compose.yaml` env vars match
- Timeouts in CI: increase Testcontainers startup timeout or use `withReuse(true)` + `~/.testcontainers.properties`

---

## Quick Reference

| Annotation | Purpose |
|------------|---------|
| `@Entity` | Marks class as JPA entity |
| `@Table` | Specifies table details and indexes |
| `@Id` | Marks primary key field |
| `@GeneratedValue` | Auto-generated primary key strategy |
| `@Column` | Column constraints and mapping |
| `@OneToMany/@ManyToOne` | One-to-many/many-to-one relationships |
| `@ManyToMany` | Many-to-many relationships |
| `@JoinColumn/@JoinTable` | Join column/table configuration |
| `@Transactional` | Declares transaction boundaries |
| `@Query` | Custom JPQL/native queries |
| `@Modifying` | Marks query as UPDATE/DELETE |
| `@EntityGraph` | Defines fetch graph for associations |
| `@Version` | Optimistic locking version field |

## References

- [Spring Boot Data Access](https://docs.spring.io/spring-boot/reference/data/sql.html)
- [Hibernate Database Schema Generation](https://docs.jboss.org/hibernate/orm/current/userguide/html_single/Hibernate_User_Guide.html#schema-generation)
- [Testcontainers PostgreSQL Module](https://java.testcontainers.org/modules/databases/postgres/)
- [Docker Deployment Guide](DOCKER.md) — `compose.yaml` setup
- [Configuration Best Practices](CONFIGURATION.md) — externalized config & secrets
- [Project Setup](PROJECT-SETUP.md) — `.env.sample` for database credentials
