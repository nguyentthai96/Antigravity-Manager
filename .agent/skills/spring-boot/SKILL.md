---
name: spring-boot
description: >
    Spring Boot 4.x / Spring Framework 7.0 backend development — REST APIs, declarative HTTP clients,
    JPA 3.2, Spring Security 7, built-in resilience, API versioning, observability with OpenTelemetry,
    GraalVM native images, Spring Modulith, Thymeleaf, ArchUnit architecture tests, and Taskfile automation.
    Use this skill:
      * When developing Spring Boot applications using Spring MVC, Spring Data JPA, Spring Modulith, Spring Security
      * To create recommended Spring Boot package structure
      * To implement REST APIs, entities/repositories, service layer, modular monoliths
      * To use Thymeleaf view templates for building web applications
      * To write tests for REST APIs and Web applications
      * To write ArchUnit tests for testing architecture
      * To configure Maven plugins and quality tools
      * To use Spring Boot's Docker Compose support for local development
      * To create Taskfile for easier execution of common tasks
metadata:
  version: "3.1.0"
  domain: backend
  language: java, kotlin
  triggers: Spring Boot 4, Spring Framework 7, Spring Security 7, Spring Data JPA, Spring WebFlux, Java REST API, Kotlin Spring, Microservices Java, Jakarta EE 11, Spring Modulith, Thymeleaf, ArchUnit, Taskfile, Maven Config
  role: specialist
  scope: implementation
  output-format: code
  platform-baseline:
    java: "25"
    kotlin: "2.2+"
    spring-boot: "4.x"
    spring-framework: "7.0"
    jakarta-ee: "11"
    jackson: "3.0"
---

# Spring Boot 4.x Skill — Java/Kotlin Backend Development

Enterprise Spring Boot 4.x / Spring Framework 7.0 backend development with Jakarta EE 11, Jackson 3, JSpecify null safety, built-in resilience, and modern Java 25+ / Kotlin 2.2+ patterns.

This agent skill helps you create Spring Boot backend projects following production-grade best practices. It provides tools and scripts to quickly bootstrap Spring Boot applications using [https://start.spring.io](https://start.spring.io), supporting both **Java** and **Kotlin** as primary languages.

---

## Critical Changes — Spring Boot 3.x → 4.x

> **IMPORTANT**: These breaking changes must be applied to ALL generated code.

| Area | Spring Boot 3.x (OLD) | Spring Boot 4.x (NEW) |
|------|----------------------|----------------------|
| Testing | `@MockBean` | `@MockitoBean` |
| Testing | `@SpyBean` | `@MockitoSpyBean` |
| Jackson | `com.fasterxml.jackson` packages | `tools.jackson` packages (annotations stay `com.fasterxml.jackson.annotation`) |
| Jackson | `ObjectMapper` (mutable) | `JsonMapper` (immutable, builder pattern) |
| Jackson | Checked exceptions (`IOException`) | Unchecked exceptions |
| Jackson | Timestamps for dates | ISO-8601 strings by default |
| Null Safety | `@Nullable` (Spring) | `@Nullable` / `@NullMarked` (JSpecify) |
| Resilience | `spring-retry` library | Built-in `@Retryable` + `@EnableResilientMethods` |
| API Versioning | Custom implementations | First-class `@ApiVersion` support |
| HTTP Clients | OpenFeign | Declarative HTTP Interface Clients |
| Observability | Micrometer only | OpenTelemetry native integration |
| JPA | JPA 3.1 | JPA 3.2 (record projections, programmatic queries) |
| Servlet | Servlet 6.0 | Servlet 6.1 (virtual threads support) |
| Validation | Bean Validation 3.0 | Bean Validation 3.1 |
| Docker base | `eclipse-temurin:21` | `eclipse-temurin:25` |
| Build tool | Gradle 8 | Gradle 9 recommended |
| Modularization | Monolithic auto-config | Modular auto-configuration JARs |

---

## Version Management

Centralized versions live in `versions.json`. All scripts read from it via `scripts/lib/versions.mjs`. Update this file to bump Java, Spring Boot fallback, Postgres, Testcontainers, etc.

---

## Prerequisites

1. **Java 25** installed (JDK)
2. **Docker** installed and running (for Testcontainers and containerized deployments)
3. **Maven** or **Gradle** (Maven is the default; Gradle support available for Kotlin projects)

---

## Core Workflow

1. **Analyze** — Understand requirements, identify service boundaries, APIs, data models
2. **Design** — Plan architecture, confirm design before coding
3. **Implement** — Build with constructor injection and layered architecture
4. **Secure** — Add Spring Security 7, OAuth2, method security; verify tests pass
5. **Test** — Write unit, integration tests; run `./mvnw test` and confirm all pass
6. **Deploy** — Configure health checks via Actuator; validate `/actuator/health` returns UP

---

## Quick Start Templates

### Entity (Jakarta EE 11 / JPA 3.2)

```java
@Entity
@Table(name = "products")
public class Product {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @NotBlank
    private String name;

    @DecimalMin("0.0")
    private BigDecimal price;

    // Getters/Setters (no Lombok — keep it explicit)
}
```

### Repository

```java
public interface ProductRepository extends JpaRepository<Product, Long> {
    List<Product> findByNameContainingIgnoreCase(String name);
}
```

### Service

```java
@Service
@Transactional(readOnly = true)
public class ProductService {
    private final ProductRepository repo;

    public ProductService(ProductRepository repo) {
        this.repo = repo;
    }

    public List<Product> search(String name) {
        return repo.findByNameContainingIgnoreCase(name);
    }

    @Transactional
    public Product create(ProductRequest request) {
        var product = new Product();
        product.setName(request.name());
        product.setPrice(request.price());
        return repo.save(product);
    }
}
```

### REST Controller

```java
@RestController
@RequestMapping("/api/v1/products")
@Validated
public class ProductController {
    private final ProductService service;

    public ProductController(ProductService service) {
        this.service = service;
    }

    @GetMapping
    public List<Product> search(@RequestParam(defaultValue = "") String name) {
        return service.search(name);
    }

    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    public Product create(@Valid @RequestBody ProductRequest request) {
        return service.create(request);
    }
}
```

### DTO (Record)

```java
public record ProductRequest(
    @NotBlank String name,
    @DecimalMin("0.0") BigDecimal price
) {}
```

### Global Exception Handler

```java
@RestControllerAdvice
public class GlobalExceptionHandler {
    @ExceptionHandler(MethodArgumentNotValidException.class)
    @ResponseStatus(HttpStatus.BAD_REQUEST)
    public Map<String, String> handleValidation(MethodArgumentNotValidException ex) {
        return ex.getBindingResult().getFieldErrors().stream()
            .collect(Collectors.toMap(FieldError::getField,
                    error -> error.getDefaultMessage() != null ? error.getDefaultMessage() : "Invalid"));
    }

    @ExceptionHandler(EntityNotFoundException.class)
    @ResponseStatus(HttpStatus.NOT_FOUND)
    public Map<String, String> handleNotFound(EntityNotFoundException ex) {
        return Map.of("error", ex.getMessage());
    }
}
```

### Test Slice (`@MockitoBean`)

```java
@WebMvcTest(ProductController.class)
class ProductControllerTest {
    @Autowired MockMvc mockMvc;
    @MockitoBean ProductService service; // ⚡ NOT @MockBean

    @Test
    void createProduct_validRequest_returns201() throws Exception {
        var product = new Product();
        product.setName("Widget");
        when(service.create(any())).thenReturn(product);

        mockMvc.perform(post("/api/v1/products")
                .contentType(MediaType.APPLICATION_JSON)
                .content("""{"name":"Widget","price":10.0}"""))
            .andExpect(status().isCreated())
            .andExpect(jsonPath("$.name").value("Widget"));
    }
}
```

---

## Spring Boot 4.x New Features

### Built-in Resilience (`@Retryable`)

```java
@Configuration
@EnableResilientMethods  // ⚡ Activates built-in resilience
public class ResilienceConfig {}

@Service
public class ExternalApiService {
    private final RestClient restClient;

    public ExternalApiService(RestClient.Builder builder) {
        this.restClient = builder.baseUrl("https://api.example.com").build();
    }

    @Retryable(maxAttempts = 3, backoff = @Backoff(delay = 1000, multiplier = 2.0))
    public ExternalData fetchData(String id) {
        return restClient.get()
            .uri("/data/{id}", id)
            .retrieve()
            .body(ExternalData.class);
    }

    @ConcurrencyLimit(10)  // ⚡ Built-in concurrency protection
    public Report generateReport(ReportRequest request) {
        // Heavy computation...
        return report;
    }
}
```

### Declarative HTTP Interface Client

```java
public interface UserServiceClient {
    @GetExchange("/users/{id}")
    UserDto getUser(@PathVariable Long id);

    @PostExchange("/users")
    UserDto createUser(@RequestBody CreateUserRequest request);

    @GetExchange("/users")
    List<UserDto> listUsers(@RequestParam(defaultValue = "0") int page);
}

@Configuration
public class HttpClientConfig {
    @Bean
    public UserServiceClient userServiceClient(RestClient.Builder builder) {
        RestClient restClient = builder.baseUrl("http://user-service:8080/api/v1").build();
        return HttpServiceProxyFactory
            .builderFor(RestClientAdapter.create(restClient))
            .build()
            .createClient(UserServiceClient.class);
    }
}
```

### First-Class API Versioning

```java
@RestController
@RequestMapping("/api/products")
public class ProductController {

    @GetMapping
    @ApiVersion("1")
    public List<ProductV1Dto> getProductsV1() {
        // V1 response format
    }

    @GetMapping
    @ApiVersion("2")
    public List<ProductV2Dto> getProductsV2() {
        // V2 response format with additional fields
    }
}
```

```properties
# application.properties
spring.mvc.api-versioning.strategy=path   # path | header | query | media-type
spring.mvc.api-versioning.header-name=X-API-Version
```

### Jackson 3.0 Configuration

```java
@Configuration
public class JacksonConfig {
    @Bean
    public JsonMapper jsonMapper() {
        return JsonMapper.builder()
            .addModule(new JavaTimeModule())
            .disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS)
            .enable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
            .build();  // ⚡ Immutable — thread-safe by design
    }
}
```

### JSpecify Null Safety

```java
import org.jspecify.annotations.NullMarked;
import org.jspecify.annotations.Nullable;

@NullMarked  // ⚡ All parameters/returns non-null by default
@Service
public class UserService {

    public UserDto findById(Long id) {
        return userRepository.findById(id)
            .map(UserDto::from)
            .orElseThrow(() -> new ResourceNotFoundException("User not found: " + id));
    }

    public @Nullable UserDto findByEmail(String email) {
        return userRepository.findByEmail(email)
            .map(UserDto::from)
            .orElse(null);
    }
}
```

### OpenTelemetry Observability

```properties
# application.properties
management.tracing.sampling.probability=1.0
management.otlp.tracing.endpoint=http://localhost:4318/v1/traces
management.otlp.metrics.export.enabled=true
management.otlp.metrics.export.url=http://localhost:4318/v1/metrics
management.endpoints.web.exposure.include=health,info,metrics,prometheus
```

### Virtual Threads (Servlet 6.1)

```properties
# application.properties
spring.threads.virtual.enabled=true
```

### Caching (`@Cacheable` / `@CacheEvict`)

```java
@Configuration
@EnableCaching
public class CacheConfig {
    @Bean
    public CacheManager cacheManager() {
        CaffeineCacheManager manager = new CaffeineCacheManager();
        manager.setCaffeine(Caffeine.newBuilder()
            .expireAfterWrite(Duration.ofMinutes(10))
            .maximumSize(1000));
        return manager;
    }
}

@Service
@Transactional(readOnly = true)
public class ProductService {
    private final ProductRepository repo;

    public ProductService(ProductRepository repo) {
        this.repo = repo;
    }

    @Cacheable(value = "products", key = "#id")
    public Product findById(Long id) {
        return repo.findById(id)
            .orElseThrow(() -> new EntityNotFoundException("Product not found: " + id));
    }

    @CacheEvict(value = "products", key = "#id")
    @Transactional
    public Product update(Long id, ProductRequest request) {
        var product = findById(id);
        product.setName(request.name());
        product.setPrice(request.price());
        return repo.save(product);
    }

    @CacheEvict(value = "products", allEntries = true)
    @Transactional
    public void deleteAll() {
        repo.deleteAll();
    }
}
```

#### Cache Providers

| Provider | Use Case |
|----------|----------|
| Caffeine | Local in-process cache (single instance) |
| Redis | Distributed cache (multi-instance / microservices) |
| EhCache | JVM-level cache with overflow to disk |

### Async Operations (`@EnableAsync` / `@Async`)

```java
@Configuration
@EnableAsync
public class AsyncConfig {
    @Bean
    public TaskExecutor taskExecutor() {
        ThreadPoolTaskExecutor executor = new ThreadPoolTaskExecutor();
        executor.setCorePoolSize(5);
        executor.setMaxPoolSize(20);
        executor.setQueueCapacity(100);
        executor.setThreadNamePrefix("async-");
        executor.setRejectedExecutionHandler(new ThreadPoolExecutor.CallerRunsPolicy());
        executor.initialize();
        return executor;
    }
}

@Service
public class NotificationService {
    private final EmailClient emailClient;

    public NotificationService(EmailClient emailClient) {
        this.emailClient = emailClient;
    }

    @Async
    public CompletableFuture<Void> sendWelcomeEmail(String email) {
        emailClient.send(email, "Welcome!", "Thank you for signing up.");
        return CompletableFuture.completedFuture(null);
    }

    @Async
    public CompletableFuture<Report> generateReport(ReportCriteria criteria) {
        // Long-running computation
        Report report = buildReport(criteria);
        return CompletableFuture.completedFuture(report);
    }
}
```

### Custom Validation Annotations

```java
@Documented
@Constraint(validatedBy = UniqueEmailValidator.class)
@Target({ElementType.FIELD})
@Retention(RetentionPolicy.RUNTIME)
public @interface UniqueEmail {
    String message() default "Email already exists";
    Class<?>[] groups() default {};
    Class<? extends Payload>[] payload() default {};
}

public class UniqueEmailValidator implements ConstraintValidator<UniqueEmail, String> {
    private final UserRepository userRepository;

    public UniqueEmailValidator(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    @Override
    public boolean isValid(String email, ConstraintValidatorContext context) {
        if (email == null) return true; // Let @NotNull handle null
        return !userRepository.existsByEmail(email);
    }
}

public record CreateUserRequest(
    @NotBlank String name,
    @NotBlank @Email @UniqueEmail String email
) {}
```

---

## Reactive WebFlux

```java
@RestController
@RequestMapping("/api/v1/orders")
public class OrderController {
    private final OrderService orderService;

    public OrderController(OrderService orderService) {
        this.orderService = orderService;
    }

    @GetMapping("/{id}")
    public Mono<ResponseEntity<OrderDto>> getOrder(@PathVariable UUID id) {
        return orderService.findById(id)
                .map(ResponseEntity::ok)
                .defaultIfEmpty(ResponseEntity.notFound().build());
    }

    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    public Mono<OrderDto> createOrder(@Valid @RequestBody CreateOrderRequest request) {
        return orderService.create(request);
    }
}
```

---

## Spring Security 7 — JWT

```java
@Configuration
@EnableMethodSecurity
public class SecurityConfig {
    @Bean
    public SecurityFilterChain filterChain(HttpSecurity http) throws Exception {
        return http
                .csrf(AbstractHttpConfigurer::disable)
                .sessionManagement(s -> s.sessionCreationPolicy(STATELESS))
                .authorizeHttpRequests(auth -> auth
                        .requestMatchers("/actuator/health").permitAll()
                        .anyRequest().authenticated())
                .oauth2ResourceServer(oauth2 -> oauth2.jwt(Customizer.withDefaults()))
                .build();
    }
}
```

---

## Kotlin Quick Start

### Entity (`kotlin-jpa` Plugin)

```kotlin
@Entity
@Table(name = "products")
class Product(
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    val id: Long? = null,

    @field:NotBlank
    var name: String,

    @field:DecimalMin("0.0")
    var price: BigDecimal
)
```

### Service (Coroutines for WebFlux)

```kotlin
@Service
@Transactional(readOnly = true)
class ProductService(private val repo: ProductRepository) {

    fun search(name: String): List<Product> =
        repo.findByNameContainingIgnoreCase(name)

    @Transactional
    fun create(request: ProductRequest): Product =
        repo.save(Product(name = request.name, price = request.price))
}
```

### DTO

```kotlin
data class ProductRequest(
    @field:NotBlank val name: String,
    @field:DecimalMin("0.0") val price: BigDecimal
)
```

### Kotlin Best Practices

1. Use **Gradle with Kotlin DSL** (`build.gradle.kts`) as the build tool
2. Use `data class` for DTOs and domain entities
3. Use `@field:` annotation use-site targets for validation annotations (e.g., `@field:NotBlank`)
4. Use Kotlin coroutines with Spring WebFlux for reactive APIs
5. Apply the `kotlin-spring` (all-open) and `kotlin-jpa` (no-arg) compiler plugins
6. Prefer `val` over `var` for immutability
7. Use Kotlin-idiomatic null safety instead of `Optional<T>` — use `findByIdOrNull()` extension
8. Use `@MockkBean` / MockK instead of Mockito for testing (or configure Mockito with `mockito-kotlin`)

---

## Constraints

### MUST DO

- Constructor injection (no field injection)
- `@Valid` on all request bodies
- `@Transactional` for multi-step writes
- `@Transactional(readOnly = true)` for reads
- Type-safe config with `@ConfigurationProperties`
- Global exception handling with `@RestControllerAdvice`
- Externalize secrets (use env vars, not properties files)
- Use `@MockitoBean` / `@MockitoSpyBean` (NOT `@MockBean` / `@SpyBean`)
- Use JSpecify `@NullMarked` / `@Nullable` for null safety
- Use `JsonMapper.builder()` (immutable) for Jackson 3 configuration
- Use `@EnableResilientMethods` + `@Retryable` for resilience (no external library)
- Use Declarative HTTP Interface Clients for service-to-service calls
- Use `.properties` files (not YAML), externalize secrets via environment variables
- Set up foundational dotfiles: `.gitignore`, `.env.sample`, `.editorconfig`, `.gitattributes`, `.dockerignore`
- The user must review changes before they are committed to git

### MUST NOT DO

- Field injection (`@Autowired` on fields)
- Skip input validation on endpoints
- Mix blocking and reactive code
- Store secrets in `application.properties`
- Use deprecated `@MockBean` / `@SpyBean` annotations
- Use mutable `ObjectMapper` (use `JsonMapper` builder instead)
- Use `spring-retry` or external resilience libraries (built-in now)
- Use OpenFeign for HTTP clients (use HTTP Interface Clients)
- Use legacy `javax.*` packages (must be `jakarta.*`)
- Hardcode URLs, credentials, environment values
- Create anemic domain models with only getters/setters (add behavior to entities)
- Create god services with too many responsibilities (split into focused services)
- Access repositories directly from controllers (always go through the service layer)
- Introduce circular dependencies between components (use events, DTOs, or restructure)
- Ignore `@Transactional` propagation semantics (understand `REQUIRED` vs `REQUIRES_NEW`)

---

## Anti-Patterns

| Anti-Pattern | Problem | Fix |
|---|---|---|
| Field injection with `@Autowired` | Hides dependencies, untestable | Use constructor injection |
| Exposing entities directly | Tight coupling, security risk | Use DTOs for API boundaries |
| Missing `@Transactional` on writes | Data inconsistency | Add `@Transactional` |
| Anemic domain model | Business logic scattered across services | Put domain logic in entities |
| God service class | Violates SRP, hard to test | Split into focused services |
| Repository in controller | Bypasses business logic, hard to test | Use service layer |
| Synchronous blocking in reactive code | Thread exhaustion | Use `@Async` or reactive operators |
| Hardcoded configuration | Can't change per environment | Use `@ConfigurationProperties` |
| No global exception handling | Inconsistent error responses | Implement `@RestControllerAdvice` |
| Missing Bean Validation | Invalid data reaches business logic | Add `@Valid` + annotations |
| Circular dependencies | Tight coupling, startup failure | Refactor, use events or DTOs |
| N+1 query problem | Excessive database queries | Use `@EntityGraph` or `JOIN FETCH` |
| Not using `readOnly = true` | Unnecessary write locks | Add to read-only methods |
| High-cardinality metric tags | Prometheus OOM / slow queries | Use bounded label values |

---

## Best Practices

1. Use the latest Spring Boot version (currently 4.x) — the `create-project-latest.mjs` script automatically fetches it
2. Review the **Critical Changes** table above for all breaking changes
3. Include Spring Boot Actuator for production-ready features
4. Use Spring Data JPA for database access
5. Use PostgreSQL for database — see [data.md](references/data.md)
6. Use properties files for configuration — see [CONFIGURATION.md](references/CONFIGURATION.md)
7. Use `spring-boot-docker-compose` for automatic database startup — see [DOCKER.md](references/DOCKER.md)
8. Follow RESTful API design principles
9. Configure proper logging with Logback — see [LOGGING.md](references/LOGGING.md)
10. Use Maven for Java projects and Gradle (Kotlin DSL) for Kotlin projects
11. Include Spring Boot DevTools for development productivity
12. Add Spring Security only when needed — see [security.md](references/security.md)
13. Configure Docker for containerized deployments — see [DOCKER.md](references/DOCKER.md)
14. Enable GraalVM native image support for faster startup — see [GRAALVM.md](references/GRAALVM.md)
15. Use `@Cacheable` for frequently accessed, expensive-to-compute data
16. Configure `@Async` with a bounded `ThreadPoolTaskExecutor` for long-running operations
17. Create custom validation annotations (`ConstraintValidator`) for domain-specific constraints
18. Organize code by business feature (package-by-feature), not by technical layer
19. Track business KPIs with custom Micrometer metrics (counters, timers, gauges)
20. Define SLIs/SLOs and set up alerting for production services

---

## Architecture Patterns

### Layer-Based Structure (Simple Projects)

#### Java

```
src/main/java/com/example/app/
├── Application.java
├── config/            # @Configuration, @ConfigurationProperties
├── controller/        # REST endpoints
├── service/           # Business logic (only if needed — simple CRUD can skip)
├── repository/        # Data access (Spring Data JPA)
├── domain/            # JPA entities
├── dto/               # Request/Response records
├── exception/         # Custom exceptions + @RestControllerAdvice handler
├── client/            # HTTP Interface Client interfaces
└── resilience/        # @EnableResilientMethods config
```

#### Kotlin

```
src/main/kotlin/com/example/app/
├── Application.kt
├── config/
├── controller/
├── service/           # Only if needed
├── repository/
├── domain/
├── dto/
├── exception/
├── client/
└── resilience/
```

### Package-by-Feature Structure (DDD — Medium/Large Projects)

For larger applications, organize code by business feature rather than technical layers:

```
src/main/java/com/example/
├── users/
│   ├── domain/              # User entity, UserRepository
│   ├── application/         # UserService, CreateUserCommand
│   └── interfaces/          # UserController, UserDto, UserDtoMapper
├── products/
│   ├── domain/              # Product entity, ProductRepository
│   ├── application/         # ProductService
│   └── interfaces/          # ProductController, ProductDto
├── orders/
│   ├── domain/
│   ├── application/
│   └── interfaces/
└── shared/
    ├── config/              # SecurityConfig, DatabaseConfig, AsyncConfig
    ├── exception/           # GlobalExceptionHandler
    └── infrastructure/      # Cross-cutting utilities
```

#### When to Use Package-by-Feature

- Application has 3+ distinct business domains
- Multiple developers/teams working on the codebase
- Planning to extract microservices in the future (Spring Modulith)
- Need clear module boundaries and encapsulation

### Full Project Layout

```plaintext
my-spring-boot-app/
├── .gitignore                 # Java + secrets (see references/PROJECT-SETUP.md)
├── .env.sample                # Template for local env vars; .env is gitignored
├── .editorconfig              # Consistent formatting across IDEs
├── .gitattributes             # Normalize line endings, better diffs
├── .dockerignore              # Slim Docker build contexts
├── .vscode/                   # Optional editor recommendations
│   ├── extensions.json
│   └── settings.json
├── .devcontainer/             # Optional Dev Container (Java 25 + PostgreSQL)
│   ├── devcontainer.json
│   └── docker-compose.yml
├── src/
│   ├── main/
│   │   ├── java/
│   │   │   └── com/example/app/
│   │   │       ├── Application.java
│   │   │       ├── config/
│   │   │       ├── controller/
│   │   │       ├── service/
│   │   │       ├── repository/
│   │   │       ├── domain/
│   │   │       ├── dto/
│   │   │       ├── exception/
│   │   │       ├── client/
│   │   │       └── resilience/
│   │   └── resources/
│   │       ├── application.properties
│   │       ├── application-dev.properties
│   │       ├── application-prod.properties
│   │       └── db/
│   │           └── migration/         # Flyway/Liquibase migrations
│   └── test/
│       └── java/
│           └── com/example/app/
│               ├── config/
│               ├── controller/        # @WebMvcTest slices
│               ├── service/           # Unit tests with Mockito
│               ├── repository/        # @DataJpaTest slices
│               ├── domain/
│               └── integration/       # @SpringBootTest + Testcontainers
├── Dockerfile                   # Standard JVM Docker build
├── Dockerfile-native            # GraalVM native image build
├── compose.yaml                 # Dev database (spring-boot-docker-compose)
├── docker-compose.yml           # Full stack with PostgreSQL
├── docker-compose-native.yml    # Native image with PostgreSQL
├── pom.xml
└── README.md
```

### Layering Rules

- Controller → Service → Repository
- Controller handles HTTP, validation
- Service handles business logic, transactions
- Repository handles data persistence
- Client interfaces for external service calls

---

## Common Annotations

| Annotation | Purpose |
|---|---|
| `@RestController` | REST controller (combines `@Controller` + `@ResponseBody`) |
| `@Service` | Business logic component |
| `@Repository` | Data access component |
| `@Transactional` | Transaction management |
| `@Valid` | Trigger validation |
| `@ConfigurationProperties` | Bind properties to class |
| `@EnableMethodSecurity` | Enable method security |
| `@MockitoBean` | ⚡ Mock bean in tests (replaces `@MockBean`) |
| `@MockitoSpyBean` | ⚡ Spy bean in tests (replaces `@SpyBean`) |
| `@EnableResilientMethods` | ⚡ Enable built-in resilience |
| `@Retryable` | ⚡ Built-in retry (no `spring-retry`) |
| `@ConcurrencyLimit` | ⚡ Built-in concurrency control |
| `@ApiVersion` | ⚡ First-class REST API versioning |
| `@NullMarked` | ⚡ JSpecify — all non-null by default |
| `@GetExchange` / `@PostExchange` | ⚡ Declarative HTTP client methods |
| `@Cacheable` | Cache method results |
| `@CacheEvict` | Evict cache entries |
| `@CachePut` | Update cache |
| `@EnableAsync` / `@Async` | Asynchronous method execution |
| `@EnableCaching` | Enable Spring Cache abstraction |

---

## Dependencies

Generated projects include: Spring Web, Spring Data JPA, Spring Boot Actuator, DevTools, PostgreSQL Driver, Bean Validation, Docker Compose support, Test Starter with JUnit 5, and TestContainers.

Kotlin projects additionally include: `kotlin-stdlib`, `kotlin-reflect`, `jackson-module-kotlin`, `kotlin-spring` plugin, `kotlin-jpa` plugin.

---

## Validation

| # | What | Command |
|---|---|---|
| 1 | Build backend | `./mvnw clean install` / `./gradlew build` |
| 2 | Unit tests | `./mvnw test` / `./gradlew test` |
| 3 | Integration tests | `./mvnw verify` / `./gradlew integrationTest` (Testcontainers + `@ServiceConnection`) |
| 4 | Run application | `./mvnw spring-boot:run` / `./gradlew bootRun` |
| 5 | Native build | `./mvnw native:compile` / `./gradlew nativeCompile` |
| 6 | Health check | `curl http://localhost:8080/actuator/health` → must return `{"status":"UP"}` |

> Run validation steps first. If anything fails, fix before proceeding.

---

## Reference Guides

Load detailed patterns based on context:

| Topic | Reference | When to Load |
|---|---|---|
| Web/REST | [web.md](references/web.md) | Controllers, validation, ProblemDetail exception handling, API versioning, Value Object converters, Jackson 3 binding, HTTP Interface Client |
| Data Access | [data.md](references/data.md) | JPA 3.2, DDD patterns (TSID, Value Objects, BaseEntity), repositories, transactions, PostgreSQL, HikariCP, performance |
| Security | [security.md](references/security.md) | Spring Security 7, OAuth2, JWT, CSRF, rate limiting, password reset, auth patterns |
| Cloud/Config | [cloud.md](references/cloud.md) | Config server, service discovery, resilience, gateway |
| Testing | [testing.md](references/testing.md) | Unit, integration, slice tests, MockMvc, RestTestClient, `@MockitoBean`, Testcontainers, MockMvcTester, security scanning |
| Configuration | [CONFIGURATION.md](references/CONFIGURATION.md) | Properties files, profiles, secrets management, CORS, Jackson, logging, actuator |
| Logging | [LOGGING.md](references/LOGGING.md) | Logback configuration and patterns |
| Project Setup | [PROJECT-SETUP.md](references/PROJECT-SETUP.md) | `.gitignore`, `.env.sample`, `.editorconfig`, `.devcontainer/` |
| Docker | [DOCKER.md](references/DOCKER.md) | Docker, Docker Compose, development automation, Redis/Grafana/Mailpit services |
| GraalVM | [GRAALVM.md](references/GRAALVM.md) | Docker-based native builds, optimization |
| Azure | [AZURE.md](references/AZURE.md) | Azure Container Apps, Azure Database for PostgreSQL |
| Spring Boot 4 Migration | [SPRING-BOOT-4.md](references/SPRING-BOOT-4.md) | Key changes from Spring Boot 3, Jackson 3 annotations |
| Maven Config | [spring-boot-maven-config.md](references/spring-boot-maven-config.md) | Maven plugins, code quality, build configuration |
| Spring Modulith | [spring-modulith.md](references/spring-modulith.md) | Modular monolith, module boundaries, events |
| Thymeleaf | [thymeleaf.md](references/thymeleaf.md) | Server-side view templates, fragments, layouts |
| Web App Testing | [spring-boot-webapp-testing-with-mockmvctester.md](references/spring-boot-webapp-testing-with-mockmvctester.md) | MockMvcTester, view controller tests |
| ArchUnit | [archunit.md](references/archunit.md) | Architecture tests, dependency rules, layer checks |
| Taskfile | [taskfile.md](references/taskfile.md) | Task runner commands, development workflow shortcuts |

| Microservices Patterns | [spring-microservices-patterns.md](references/spring-microservices-patterns.md) | API Gateway, Service Discovery, Circuit Breaker, CQRS, Saga, Event-Driven, Service Mesh |
| Observability Patterns | [spring-observability-patterns.md](references/spring-observability-patterns.md) | Micrometer metrics, Prometheus, Grafana, SLI/SLO, alerting, structured logging, distributed tracing |
| Kotlin Patterns | [spring-boot-kotlin-patterns.md](references/spring-boot-kotlin-patterns.md) | Coroutine controllers, WebFlux + Flow, Kotlin DSL, MockK testing, functional routing |

---

## Knowledge Base

- Spring Boot 4.x, Spring Framework 7.0, Java 25, Kotlin 2.2
- Jakarta EE 11, Jackson 3.0, JSpecify
- Spring WebFlux, Project Reactor, Spring Data JPA 3.2
- Spring Security 7, OAuth2/JWT, Hibernate 7, R2DBC
- Spring Cloud 2025, Declarative HTTP Interface Clients
- Built-in Resilience (`@Retryable`), API Versioning
- OpenTelemetry, Micrometer, Virtual Threads, GraalVM Native Images
- JUnit 5, TestContainers, Mockito (`@MockitoBean`), Maven/Gradle 9
- Spring Cache (`@Cacheable` / `@CacheEvict`), Caffeine, Redis Cache
- `@EnableAsync` / `@Async`, `CompletableFuture`, `ThreadPoolTaskExecutor`
- Custom Validators (`ConstraintValidator`), Package-by-Feature (DDD)
- Microservices: API Gateway (Spring Cloud Gateway), Circuit Breaker (Resilience4j), Service Discovery (Eureka)
- Distributed Tracing (Zipkin/Jaeger), Event-Driven (Kafka/RabbitMQ), CQRS/Saga
- Prometheus, Grafana, SLI/SLO, Structured Logging (Logstash Encoder)
- Kotlin Coroutines, MockK, Functional Routing