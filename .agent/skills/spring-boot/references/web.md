# Web Layer — Controllers & REST APIs (Spring Boot 4.x)

## REST Controller Pattern

```java
@RestController
@RequestMapping("/api/users")
@Validated
public class UserController {
    private final UserService userService;
    
    public UserController(UserService userService) {
        this.userService = userService;
    }
    
    @GetMapping
    public ResponseEntity<Page<UserResponse>> getUsers(
            @PageableDefault(size = 20, sort = "createdAt") Pageable pageable) {
        Page<UserResponse> users = userService.findAll(pageable);
        return ResponseEntity.ok(users);
    }

    @GetMapping("/{id}")
    public ResponseEntity<UserResponse> getUser(@PathVariable Long id) {
        UserResponse user = userService.findById(id);
        return ResponseEntity.ok(user);
    }

    @PostMapping
    public ResponseEntity<UserResponse> createUser(
            @Valid @RequestBody UserCreateRequest request) {
        UserResponse user = userService.create(request);
        URI location = ServletUriComponentsBuilder
                .fromCurrentRequest()
                .path("/{id}")
                .buildAndExpand(user.id())
                .toUri();
        return ResponseEntity.created(location).body(user);
    }

    @PutMapping("/{id}")
    public ResponseEntity<UserResponse> updateUser(
            @PathVariable Long id,
            @Valid @RequestBody UserUpdateRequest request) {
        UserResponse user = userService.update(id, request);
        return ResponseEntity.ok(user);
    }

    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void deleteUser(@PathVariable Long id) {
        userService.delete(id);
    }
}
```

## First-Class API Versioning (Spring Boot 4.x)

```java
// ⚡ Spring Boot 4.x — built-in API versioning
@RestController
@RequestMapping("/api/users")
public class UserController {

    @GetMapping("/{id}")
    @ApiVersion("1")
    public UserV1Response getUserV1(@PathVariable Long id) {
        // V1 — basic user data
        return userService.findByIdV1(id);
    }

    @GetMapping("/{id}")
    @ApiVersion("2")
    public UserV2Response getUserV2(@PathVariable Long id) {
        // V2 — extended fields (e.g., preferences, activity stats)
        return userService.findByIdV2(id);
    }
}
```

```properties
# application.properties — choose versioning strategy
spring.mvc.api-versioning.strategy=path
# Options: path (/v1/users), header (X-API-Version: 1), query (?version=1), media-type
spring.mvc.api-versioning.header-name=X-API-Version
spring.mvc.api-versioning.query-param-name=version
```

## Request DTOs with Validation (Bean Validation 3.1)

```java
public record UserCreateRequest(
    @NotBlank(message = "Email is required")
    @Email(message = "Email must be valid")
    String email,

    @NotBlank(message = "Password is required")
    @Size(min = 8, max = 100, message = "Password must be 8-100 characters")
    @Pattern(regexp = "^(?=.*[A-Z])(?=.*[a-z])(?=.*\\d).*$",
             message = "Password must contain uppercase, lowercase, and digit")
    String password,

    @NotBlank(message = "Username is required")
    @Size(min = 3, max = 50)
    @Pattern(regexp = "^[a-zA-Z0-9_]+$", message = "Username must be alphanumeric")
    String username,

    @Min(value = 18, message = "Must be at least 18")
    @Max(value = 120, message = "Must be at most 120")
    Integer age
) {}

public record UserUpdateRequest(
    @Email(message = "Email must be valid")
    String email,

    @Size(min = 3, max = 50)
    String username
) {}
```

## Response DTOs

```java
public record UserResponse(
    Long id,
    String email,
    String username,
    Integer age,
    Boolean active,
    LocalDateTime createdAt,
    LocalDateTime updatedAt
) {
    public static UserResponse from(User user) {
        return new UserResponse(
            user.getId(),
            user.getEmail(),
            user.getUsername(),
            user.getAge(),
            user.getActive(),
            user.getCreatedAt(),
            user.getUpdatedAt()
        );
    }
}
```

## Converter for Value Objects

> Enables Spring MVC to automatically convert `@PathVariable` and `@RequestParam` from String to domain Value Objects.

```java
import org.springframework.core.convert.converter.Converter;
import org.springframework.stereotype.Component;

@Component
public class StringToUserIdConverter implements Converter<String, UserId> {

    @Override
    public UserId convert(String source) {
        return new UserId(source);
    }
}
```

Usage in controllers:

```java
@GetMapping("/{userId}")
ResponseEntity<UserVM> findUserById(@PathVariable UserId userId) {
    // userId is already a UserId object, not a String
}
```

## Value Object Binding with Jackson 3

> Use `@JsonValue` and `@JsonCreator` to bind Value Objects in `@RequestBody` payloads.

```java
import tools.jackson.annotation.JsonCreator;  // ⚡ Jackson 3
import tools.jackson.annotation.JsonValue;    // ⚡ Jackson 3
import jakarta.validation.constraints.NotBlank;

public record UserId(
        @JsonValue
        @NotBlank(message = "User id cannot be null or empty")
        String id
) {
    @JsonCreator
    public UserId {
        if (id == null || id.trim().isEmpty()) {
            throw new IllegalArgumentException("User id cannot be null");
        }
    }

    public static UserId of(String id) {
        return new UserId(id);
    }
}
```

Request payload example:

```json
{
  "userId": "ABSHDJFSD",
  "property-1": "value-1"
}
```

Spring MVC automatically binds `"userId"` to a `UserId` object via `@JsonCreator`.

## Global Exception Handling (ProblemDetail — RFC 7807)

> Spring Boot 4.x returns `ProblemDetail` responses by default (RFC 7807).
> Extend `ResponseEntityExceptionHandler` for consistent error formatting.

```java
@RestControllerAdvice
class GlobalExceptionHandler extends ResponseEntityExceptionHandler {
    private static final Logger log = LoggerFactory.getLogger(GlobalExceptionHandler.class);
    private final Environment environment;

    GlobalExceptionHandler(Environment environment) {
        this.environment = environment;
    }

    @Override
    public ResponseEntity<Object> handleMethodArgumentNotValid(
            MethodArgumentNotValidException ex, HttpHeaders headers,
            HttpStatusCode status, WebRequest request) {
        var errors = ex.getAllErrors().stream()
                .map(DefaultMessageSourceResolvable::getDefaultMessage)
                .toList();

        ProblemDetail problemDetail = ProblemDetail.forStatusAndDetail(
                HttpStatus.BAD_REQUEST, ex.getMessage());
        problemDetail.setTitle("Validation Error");
        problemDetail.setProperty("errors", errors);
        problemDetail.setProperty("timestamp", Instant.now());
        return ResponseEntity.unprocessableEntity().body(problemDetail);
    }

    @ExceptionHandler(ResourceNotFoundException.class)
    public ProblemDetail handleNotFound(ResourceNotFoundException ex) {
        log.error("Resource not found: {}", ex.getMessage());
        ProblemDetail problemDetail = ProblemDetail.forStatusAndDetail(
                HttpStatus.NOT_FOUND, ex.getMessage());
        problemDetail.setTitle("Resource Not Found");
        problemDetail.setProperty("timestamp", Instant.now());
        return problemDetail;
    }

    @ExceptionHandler(DataIntegrityViolationException.class)
    public ProblemDetail handleDataIntegrity(DataIntegrityViolationException ex) {
        log.error("Data integrity violation", ex);
        ProblemDetail problemDetail = ProblemDetail.forStatusAndDetail(
                HttpStatus.CONFLICT,
                "Data integrity violation — resource may already exist");
        problemDetail.setProperty("timestamp", Instant.now());
        return problemDetail;
    }

    @ExceptionHandler(Exception.class)
    ProblemDetail handleUnexpected(Exception ex) {
        log.error("Unexpected exception occurred", ex);
        String message = isDevelopmentMode()
                ? ex.getMessage()
                : "An unexpected error occurred";
        ProblemDetail problemDetail = ProblemDetail.forStatusAndDetail(
                HttpStatus.INTERNAL_SERVER_ERROR, message);
        problemDetail.setProperty("timestamp", Instant.now());
        return problemDetail;
    }

    private boolean isDevelopmentMode() {
        return Arrays.asList(environment.getActiveProfiles()).contains("dev");
    }
}
```

### Error Response Examples (RFC 7807)

**Validation Error (422):**

```json
{
  "type": "about:blank",
  "title": "Validation Error",
  "status": 422,
  "detail": "Validation failed for argument...",
  "errors": ["Email is required", "Password must be 8-100 characters"],
  "timestamp": "2025-01-15T10:30:00Z"
}
```

**Not Found (404):**

```json
{
  "type": "about:blank",
  "title": "Resource Not Found",
  "status": 404,
  "detail": "User not found with id: ABC123",
  "timestamp": "2025-01-15T10:30:00Z"
}
```

**Conflict (409):**

```json
{
  "type": "about:blank",
  "title": "Conflict",
  "status": 409,
  "detail": "Data integrity violation — resource may already exist",
  "timestamp": "2025-01-15T10:30:00Z"
}
```

**Internal Server Error (500):**

```json
{
  "type": "about:blank",
  "title": "Internal Server Error",
  "status": 500,
  "detail": "An unexpected error occurred",
  "timestamp": "2025-01-15T10:30:00Z"
}
```

## Declarative HTTP Interface Client (Spring Boot 4.x — replaces OpenFeign)

```java
// ⚡ Define interface — Spring generates proxy at runtime
public interface PaymentServiceClient {

    @GetExchange("/payments/{id}")
    PaymentDto getPayment(@PathVariable String id);

    @PostExchange("/payments")
    PaymentDto createPayment(@RequestBody CreatePaymentRequest request);

    @GetExchange("/payments")
    List<PaymentDto> listPayments(
        @RequestParam String status,
        @RequestParam(defaultValue = "0") int page
    );
}

// Configuration — wire up the client
@Configuration
public class HttpClientConfig {
    @Bean
    public PaymentServiceClient paymentServiceClient(RestClient.Builder builder) {
        RestClient restClient = builder
            .baseUrl("http://payment-service:8080/api/v1")
            .defaultHeader(HttpHeaders.CONTENT_TYPE, MediaType.APPLICATION_JSON_VALUE)
            .build();

        return HttpServiceProxyFactory
            .builderFor(RestClientAdapter.create(restClient))
            .build()
            .createClient(PaymentServiceClient.class);
    }
}
```

## Custom Validation

```java
@Target({ElementType.FIELD, ElementType.PARAMETER})
@Retention(RetentionPolicy.RUNTIME)
@Constraint(validatedBy = UniqueEmailValidator.class)
public @interface UniqueEmail {
    String message() default "Email already exists";
    Class<?>[] groups() default {};
    Class<? extends Payload>[] payload() default {};
}

@Component
public class UniqueEmailValidator implements ConstraintValidator<UniqueEmail, String> {
    private final UserRepository userRepository;

    public UniqueEmailValidator(UserRepository userRepository) {
        this.userRepository = userRepository;
    }
    
    @Override
    public boolean isValid(String email, ConstraintValidatorContext context) {
        if (email == null) return true;
        return !userRepository.existsByEmail(email);
    }
}
```

## CORS Configuration

> See [CONFIGURATION.md](CONFIGURATION.md) § "CORS Configuration" for both properties-based and Java-based CORS setup.

## Quick Reference

| Annotation | Purpose |
|------------|---------|
| `@RestController` | Marks class as REST controller (combines @Controller + @ResponseBody) |
| `@RequestMapping` | Maps HTTP requests to handler methods |
| `@GetMapping/@PostMapping` | HTTP method-specific mappings |
| `@PathVariable` | Extracts values from URI path |
| `@RequestParam` | Extracts query parameters |
| `@RequestBody` | Binds request body to method parameter |
| `@Valid` | Triggers validation on request body |
| `@RestControllerAdvice` | Global exception handling for REST controllers |
| `@ResponseStatus` | Sets HTTP status code for method |
| `@ApiVersion` | ⚡ First-class API versioning (Spring Boot 4.x) |
| `@GetExchange` / `@PostExchange` | ⚡ Declarative HTTP Interface Client methods |
| `ProblemDetail` | ⚡ RFC 7807 error response (Spring Boot 4.x default) |
| `@JsonValue` / `@JsonCreator` | Jackson 3 Value Object binding |
| `Converter<S, T>` | Spring MVC type conversion for `@PathVariable`/`@RequestParam` |
