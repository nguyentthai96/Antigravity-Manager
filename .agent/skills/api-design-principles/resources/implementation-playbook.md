# API Design Principles Implementation Playbook

This file contains detailed patterns, checklists, and code samples referenced by the skill.
All examples use **Java / Spring Boot 4.x / Spring Framework 7.0** stack.

## Core Concepts

### 1. RESTful Design Principles

**Resource-Oriented Architecture**

- Resources are nouns (users, orders, products), not verbs
- Use HTTP methods for actions (GET, POST, PUT, PATCH, DELETE)
- URLs represent resource hierarchies
- Consistent naming conventions

**HTTP Methods Semantics:**

- `GET`: Retrieve resources (idempotent, safe)
- `POST`: Create new resources
- `PUT`: Replace entire resource (idempotent)
- `PATCH`: Partial resource updates
- `DELETE`: Remove resources (idempotent)

### 2. GraphQL Design Principles

**Schema-First Development**

- Types define your domain model
- Queries for reading data
- Mutations for modifying data
- Subscriptions for real-time updates

**Query Structure:**

- Clients request exactly what they need
- Single endpoint, multiple operations
- Strongly typed schema
- Introspection built-in

### 3. API Versioning Strategies

**URL Versioning (Recommended for REST):**

```
/api/v1/users
/api/v2/users
```

**Spring Boot 4.x First-Class API Versioning:**

```java
@RestController
@RequestMapping("/api/users")
public class UserController {

    @GetMapping
    @ApiVersion("1")
    public ResponseEntity<Page<UserResponseV1>> listUsersV1(
            @PageableDefault(size = 20) Pageable pageable) {
        // V1 implementation
    }

    @GetMapping
    @ApiVersion("2")
    public ResponseEntity<Page<UserResponseV2>> listUsersV2(
            @PageableDefault(size = 20) Pageable pageable) {
        // V2 implementation with enhanced fields
    }
}
```

**Header Versioning:**

```
Accept: application/vnd.api+json; version=1
```

---

## REST API Design Patterns

### Pattern 1: Resource Collection Design

```java
// Good: Resource-oriented endpoints with Spring Boot
// GET    /api/users              → List users (with pagination)
// POST   /api/users              → Create user
// GET    /api/users/{id}         → Get specific user
// PUT    /api/users/{id}         → Replace user
// PATCH  /api/users/{id}         → Update user fields
// DELETE /api/users/{id}         → Delete user

// Nested resources
// GET    /api/users/{id}/orders  → Get user's orders
// POST   /api/users/{id}/orders  → Create order for user

// Bad: Action-oriented endpoints (avoid)
// POST   /api/createUser         ❌
// POST   /api/getUserById        ❌
// POST   /api/deleteUser         ❌

@RestController
@RequestMapping("/api/users")
public class UserController {

    private final UserService userService;

    // Constructor injection (mandatory — no field @Autowired)
    public UserController(UserService userService) {
        this.userService = userService;
    }

    @GetMapping
    public ResponseEntity<Page<UserResponse>> listUsers(
            @PageableDefault(size = 20, sort = "createdAt", direction = Sort.Direction.DESC)
            Pageable pageable) {
        return ResponseEntity.ok(userService.findAll(pageable));
    }

    @PostMapping
    public ResponseEntity<UserResponse> createUser(
            @Valid @RequestBody CreateUserRequest request) {
        UserResponse user = userService.create(request);
        URI location = ServletUriComponentsBuilder.fromCurrentRequest()
                .path("/{id}")
                .buildAndExpand(user.id())
                .toUri();
        return ResponseEntity.created(location).body(user);
    }

    @GetMapping("/{id}")
    public ResponseEntity<UserResponse> getUser(@PathVariable Long id) {
        return ResponseEntity.ok(userService.findById(id));
    }

    @PutMapping("/{id}")
    public ResponseEntity<UserResponse> replaceUser(
            @PathVariable Long id,
            @Valid @RequestBody UpdateUserRequest request) {
        return ResponseEntity.ok(userService.replace(id, request));
    }

    @PatchMapping("/{id}")
    public ResponseEntity<UserResponse> updateUser(
            @PathVariable Long id,
            @Valid @RequestBody PatchUserRequest request) {
        return ResponseEntity.ok(userService.patch(id, request));
    }

    @DeleteMapping("/{id}")
    public ResponseEntity<Void> deleteUser(@PathVariable Long id) {
        userService.delete(id);
        return ResponseEntity.noContent().build();
    }

    // Nested resource
    @GetMapping("/{id}/orders")
    public ResponseEntity<Page<OrderResponse>> getUserOrders(
            @PathVariable Long id,
            @PageableDefault(size = 20) Pageable pageable) {
        return ResponseEntity.ok(orderService.findByUserId(id, pageable));
    }
}
```

### Pattern 2: Pagination and Filtering

```java
// --- DTOs (Java Records — immutable) ---

public record UserFilterRequest(
        @Nullable String status,
        @Nullable @DateTimeFormat(iso = DateTimeFormat.ISO.DATE_TIME) Instant createdAfter,
        @Nullable String search
) {}

public record UserResponse(
        Long id,
        String name,
        String email,
        String status,
        Instant createdAt
) {}

// --- Controller ---

@RestController
@RequestMapping("/api/users")
public class UserController {

    private final UserService userService;

    public UserController(UserService userService) {
        this.userService = userService;
    }

    @GetMapping
    public ResponseEntity<Page<UserResponse>> listUsers(
            @PageableDefault(size = 20, sort = "createdAt", direction = Sort.Direction.DESC)
            Pageable pageable,
            @ModelAttribute UserFilterRequest filter) {
        Page<UserResponse> result = userService.findAll(filter, pageable);
        return ResponseEntity.ok(result);
    }
}

// --- Service ---

@Service
public class UserService {

    private final UserRepository userRepository;

    public UserService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    @Transactional(readOnly = true)
    public Page<UserResponse> findAll(UserFilterRequest filter, Pageable pageable) {
        Specification<User> spec = Specification.where(null);

        if (filter.status() != null) {
            spec = spec.and((root, query, cb) ->
                    cb.equal(root.get("status"), filter.status()));
        }
        if (filter.createdAfter() != null) {
            spec = spec.and((root, query, cb) ->
                    cb.greaterThanOrEqualTo(root.get("createdAt"), filter.createdAfter()));
        }
        if (filter.search() != null) {
            spec = spec.and((root, query, cb) ->
                    cb.or(
                            cb.like(cb.lower(root.get("name")), "%" + filter.search().toLowerCase() + "%"),
                            cb.like(cb.lower(root.get("email")), "%" + filter.search().toLowerCase() + "%")
                    ));
        }

        return userRepository.findAll(spec, pageable)
                .map(this::toResponse);
    }

    private UserResponse toResponse(User user) {
        return new UserResponse(
                user.getId(),
                user.getName(),
                user.getEmail(),
                user.getStatus(),
                user.getCreatedAt()
        );
    }
}

// --- Repository ---

public interface UserRepository extends JpaRepository<User, Long>,
        JpaSpecificationExecutor<User> {
}
```

### Pattern 3: Error Handling and Status Codes (RFC 7807)

```java
// --- Global Exception Handler ---

@RestControllerAdvice
public class GlobalExceptionHandler {

    // 404 Not Found
    @ExceptionHandler(ResourceNotFoundException.class)
    public ProblemDetail handleNotFound(ResourceNotFoundException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.NOT_FOUND, ex.getMessage());
        problem.setTitle("Resource Not Found");
        problem.setProperty("resource", ex.getResourceName());
        problem.setProperty("id", ex.getResourceId());
        problem.setProperty("timestamp", Instant.now());
        return problem;
    }

    // 422 Validation Error
    @ExceptionHandler(MethodArgumentNotValidException.class)
    public ProblemDetail handleValidation(MethodArgumentNotValidException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.UNPROCESSABLE_ENTITY, "Request validation failed");
        problem.setTitle("Validation Error");

        List<Map<String, String>> errors = ex.getBindingResult()
                .getFieldErrors()
                .stream()
                .map(error -> Map.of(
                        "field", error.getField(),
                        "message", Objects.requireNonNullElse(error.getDefaultMessage(), "Invalid value"),
                        "rejected", String.valueOf(error.getRejectedValue())
                ))
                .toList();

        problem.setProperty("errors", errors);
        problem.setProperty("timestamp", Instant.now());
        return problem;
    }

    // 409 Conflict
    @ExceptionHandler(DuplicateResourceException.class)
    public ProblemDetail handleConflict(DuplicateResourceException ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.CONFLICT, ex.getMessage());
        problem.setTitle("Resource Conflict");
        problem.setProperty("timestamp", Instant.now());
        return problem;
    }

    // 500 Internal Error (catch-all)
    @ExceptionHandler(Exception.class)
    public ProblemDetail handleUnexpected(Exception ex) {
        ProblemDetail problem = ProblemDetail.forStatusAndDetail(
                HttpStatus.INTERNAL_SERVER_ERROR, "An unexpected error occurred");
        problem.setTitle("Internal Server Error");
        problem.setProperty("timestamp", Instant.now());
        // Do NOT leak stack trace to client
        return problem;
    }
}

// --- Custom Exceptions ---

public class ResourceNotFoundException extends RuntimeException {

    private final String resourceName;
    private final Object resourceId;

    public ResourceNotFoundException(String resourceName, Object resourceId) {
        super(resourceName + " not found with id: " + resourceId);
        this.resourceName = resourceName;
        this.resourceId = resourceId;
    }

    public String getResourceName() { return resourceName; }
    public Object getResourceId() { return resourceId; }
}

public class DuplicateResourceException extends RuntimeException {

    public DuplicateResourceException(String message) {
        super(message);
    }
}

// --- HTTP Status Code Reference ---

// 200 OK           — Successful GET, PUT, PATCH
// 201 Created      — Successful POST (include Location header)
// 204 No Content   — Successful DELETE
// 400 Bad Request  — Malformed request syntax
// 401 Unauthorized — Missing or invalid authentication
// 403 Forbidden    — Authenticated but insufficient permissions
// 404 Not Found    — Resource does not exist
// 409 Conflict     — Duplicate resource or state conflict
// 422 Unprocessable — Validation failed (semantic errors)
// 500 Internal     — Unexpected server error
```

### Pattern 4: HATEOAS (Spring HATEOAS)

```java
import org.springframework.hateoas.EntityModel;
import org.springframework.hateoas.CollectionModel;
import static org.springframework.hateoas.server.mvc.WebMvcLinkBuilder.*;

@RestController
@RequestMapping("/api/users")
public class UserHateoasController {

    private final UserService userService;
    private final UserModelAssembler assembler;

    public UserHateoasController(UserService userService,
                                  UserModelAssembler assembler) {
        this.userService = userService;
        this.assembler = assembler;
    }

    @GetMapping("/{id}")
    public EntityModel<UserResponse> getUser(@PathVariable Long id) {
        UserResponse user = userService.findById(id);
        return assembler.toModel(user);
    }
}

// --- Model Assembler ---

@Component
public class UserModelAssembler
        implements RepresentationModelAssembler<UserResponse, EntityModel<UserResponse>> {

    @Override
    public EntityModel<UserResponse> toModel(UserResponse user) {
        return EntityModel.of(user,
                linkTo(methodOn(UserHateoasController.class).getUser(user.id())).withSelfRel(),
                linkTo(methodOn(UserHateoasController.class).listUsers(Pageable.unpaged())).withRel("users"),
                linkTo(methodOn(OrderController.class).getUserOrders(user.id(), Pageable.unpaged())).withRel("orders")
        );
    }
}
```

---

## GraphQL Design Patterns (Spring for GraphQL)

### Pattern 1: Schema Design

```graphql
# src/main/resources/graphql/schema.graphqls

# Clear type definitions
type User {
    id: ID!
    email: String!
    name: String!
    createdAt: String!
    orders(first: Int = 20, after: String): OrderConnection!
    profile: UserProfile
}

type Order {
    id: ID!
    status: OrderStatus!
    total: Float!
    items: [OrderItem!]!
    createdAt: String!
    user: User!
}

# Pagination (Relay-style Connection)
type OrderConnection {
    edges: [OrderEdge!]!
    pageInfo: PageInfo!
    totalCount: Int!
}

type OrderEdge {
    node: Order!
    cursor: String!
}

type PageInfo {
    hasNextPage: Boolean!
    hasPreviousPage: Boolean!
    startCursor: String
    endCursor: String
}

enum OrderStatus {
    PENDING
    CONFIRMED
    SHIPPED
    DELIVERED
    CANCELLED
}

# Query root
type Query {
    user(id: ID!): User
    users(first: Int = 20, after: String, search: String): UserConnection!
    order(id: ID!): Order
}

# Mutation root
type Mutation {
    createUser(input: CreateUserInput!): CreateUserPayload!
    updateUser(input: UpdateUserInput!): UpdateUserPayload!
    deleteUser(id: ID!): DeleteUserPayload!
    createOrder(input: CreateOrderInput!): CreateOrderPayload!
}

input CreateUserInput {
    email: String!
    name: String!
    password: String!
}

type CreateUserPayload {
    user: User
    errors: [Error!]
}

type Error {
    field: String
    message: String!
}
```

### Pattern 2: Spring for GraphQL Controller

```java
import org.springframework.graphql.data.method.annotation.Argument;
import org.springframework.graphql.data.method.annotation.MutationMapping;
import org.springframework.graphql.data.method.annotation.QueryMapping;
import org.springframework.graphql.data.method.annotation.SchemaMapping;

@Controller
public class UserGraphqlController {

    private final UserService userService;

    public UserGraphqlController(UserService userService) {
        this.userService = userService;
    }

    @QueryMapping
    public User user(@Argument Long id) {
        return userService.findById(id);
    }

    @QueryMapping
    public Connection<User> users(
            @Argument int first,
            @Argument String after,
            @Argument String search) {
        return userService.findAll(first, after, search);
    }

    @MutationMapping
    public CreateUserPayload createUser(@Argument CreateUserInput input) {
        try {
            User user = userService.create(input);
            return new CreateUserPayload(user, List.of());
        } catch (ValidationException e) {
            return new CreateUserPayload(null,
                    List.of(new Error(e.getField(), e.getMessage())));
        }
    }
}

// --- Schema mapping for nested fields (N+1 prevention) ---

@Controller
public class UserOrdersController {

    private final OrderService orderService;

    public UserOrdersController(OrderService orderService) {
        this.orderService = orderService;
    }

    @SchemaMapping(typeName = "User", field = "orders")
    public Connection<Order> orders(User user,
                                     @Argument int first,
                                     @Argument String after) {
        return orderService.findByUserId(user.getId(), first, after);
    }
}
```

### Pattern 3: DataLoader (N+1 Problem Prevention with Spring for GraphQL)

```java
import org.springframework.graphql.execution.BatchLoaderRegistry;
import org.dataloader.DataLoader;
import org.dataloader.BatchLoaderWithContext;

@Configuration
public class DataLoaderConfig {

    @Bean
    public BatchLoaderRegistry batchLoaderRegistry(
            UserRepository userRepository,
            OrderRepository orderRepository) {

        BatchLoaderRegistry registry = new DefaultBatchLoaderRegistry();

        // Batch load users by ID
        registry.forTypePair(Long.class, User.class)
                .registerMappedBatchLoader((userIds, env) -> {
                    List<User> users = userRepository.findAllById(userIds);
                    Map<Long, User> userMap = users.stream()
                            .collect(Collectors.toMap(User::getId, Function.identity()));
                    return Mono.just(userMap);
                });

        // Batch load orders by user ID
        registry.forTypePair(Long.class, List.class)
                .withName("ordersByUser")
                .registerMappedBatchLoader((userIds, env) -> {
                    List<Order> orders = orderRepository.findAllByUserIdIn(userIds);
                    Map<Long, List<Order>> grouped = orders.stream()
                            .collect(Collectors.groupingBy(Order::getUserId));
                    // Ensure all keys are present
                    Map<Long, List<Order>> result = new HashMap<>();
                    for (Long userId : userIds) {
                        result.put(userId, grouped.getOrDefault(userId, List.of()));
                    }
                    return Mono.just(result);
                });

        return registry;
    }
}

// --- Repository method for batch loading ---

public interface OrderRepository extends JpaRepository<Order, Long> {

    @Query("SELECT o FROM Order o WHERE o.userId IN :userIds")
    List<Order> findAllByUserIdIn(@Param("userIds") Collection<Long> userIds);
}
```

---

## Best Practices

### REST APIs (Spring Boot)

1. **Consistent Naming**: Use plural nouns for collections (`/users`, not `/user`)
2. **Stateless**: Each request contains all necessary information
3. **Use HTTP Status Codes Correctly**: 2xx success, 4xx client errors, 5xx server errors
4. **Version Your API**: Use `@ApiVersion` (Spring Boot 4.x) or URL versioning
5. **Pagination**: Always paginate with `Pageable` and return `Page<T>`
6. **Rate Limiting**: Use Spring Cloud Gateway or Bucket4j
7. **Documentation**: Use SpringDoc OpenAPI for interactive docs
8. **Error Format**: Use RFC 7807 `ProblemDetail` (built into Spring 6+)
9. **Validation**: Use `@Valid` with Jakarta Bean Validation annotations
10. **Constructor Injection**: Never use `@Autowired` on fields

### GraphQL APIs (Spring for GraphQL)

1. **Schema First**: Define `.graphqls` files in `src/main/resources/graphql/`
2. **Avoid N+1**: Use `BatchLoaderRegistry` and `DataLoader`
3. **Input Validation**: Validate at schema level and in service layer
4. **Error Handling**: Return structured errors in mutation payloads
5. **Pagination**: Use Relay-style cursor connections
6. **Deprecation**: Use `@deprecated` directive in schema for gradual migration
7. **Monitoring**: Enable GraphQL observability with Spring Boot Actuator

## Common Pitfalls

- **Over-fetching/Under-fetching (REST)**: Fixed in GraphQL but requires DataLoaders
- **Breaking Changes**: Version APIs or use deprecation strategies
- **Inconsistent Error Formats**: Standardize on RFC 7807 `ProblemDetail`
- **Missing Rate Limits**: APIs without limits are vulnerable to abuse
- **Poor Documentation**: Use SpringDoc OpenAPI for auto-generated docs
- **Ignoring HTTP Semantics**: POST for idempotent operations breaks expectations
- **Tight Coupling**: API structure shouldn't mirror database schema
- **Returning Entities**: Always map to DTOs/records, never expose JPA entities
- **Missing `@Transactional(readOnly = true)`**: Read operations should be read-only
