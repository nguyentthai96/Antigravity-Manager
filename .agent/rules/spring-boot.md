---
trigger: always_on
description: "Project-specific Spring Boot coding rules for base-core. Enforces naming conventions, annotation usage, and API design standards."
---

# Spring Boot Rules

> Project-specific coding standards for Spring Boot development in base-core. For comprehensive Spring Boot patterns and architecture guidance.

## Principles

- Convention over Configuration
- Standalone, production-grade applications
- Opinionated starter dependencies
- Constructor-based Dependency Injection (no field `@Autowired`)
- Aspect-Oriented Programming (AOP) where appropriate

## Rules

### Dependency Injection

- **Always** use constructor injection; avoid `@Autowired` on fields
- Use `@RequiredArgsConstructor` (Lombok) or explicit constructors
- Keep constructor parameter count ≤ 5; extract a facade/service if exceeded

### Annotation Usage

- `@SpringBootApplication` — main entry point only
- `@RestController` / `@Controller` — web layer only
- `@Service` — business logic layer only
- `@Repository` — data access layer only
- `@Component` — generic beans only when no specific stereotype applies
- **Do NOT use deprecated annotations** such as `org.springframework.lang.NonNull`; use JSpecify (`@NonNull` / `@Nullable`) instead

### Naming Conventions

- Entity properties using `System.currentTimeMillis()` **must** end with `*At` suffix (e.g., `createdAt`, `updatedAt`, `deletedAt`)
- DTOs follow the pattern: `<Entity><Action>Request` / `<Entity><Action>Response`
- Use clear, intention-revealing names for all beans and methods

### Configuration

- Use `application.yml` over `application.properties`
- Use Spring Profiles (`dev`, `test`, `staging`, `prod`)
- Use `@ConfigurationProperties` for type-safe config (not `@Value` for complex objects)
- Externalize all environment-specific values

### Exception Handling

- Handle exceptions globally with `@ControllerAdvice` + `@ExceptionHandler`
- Return RFC 7807 Problem Details format for all error responses
- Never leak stack traces to API consumers

### Input Validation

- Validate all incoming requests with `@Valid` and Bean Validation annotations
- Use `@NotNull`, `@NotBlank`, `@Size`, `@Pattern` etc. on DTOs
- Validate at controller layer; enforce at service layer

### Testing

- Write integration tests with `@SpringBootTest`
- Use `@MockitoBean` (not deprecated `@MockBean`) for Spring Boot 4.x
- Target test coverage > 80%

### Observability

- Enable Spring Boot Actuator for health/metrics endpoints
- Use Micrometer for metrics export
- Use structured logging with MDC context
- Integrate distributed tracing (OpenTelemetry)

## Anti-Patterns

- ❌ Field injection with `@Autowired`
- ❌ Using `org.springframework.lang.NonNull` (deprecated)
- ❌ Catching generic `Exception` without re-throwing or proper handling
- ❌ Returning raw entity objects from REST controllers (use DTOs)
- ❌ Hardcoding configuration values in source code
- ❌ Using `@GetMapping` for write operations or `@PostMapping` for reads