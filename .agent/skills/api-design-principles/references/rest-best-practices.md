# REST API Best Practices

## URL Structure

### Resource Naming

```
# Good - Plural nouns
GET /api/users
GET /api/orders
GET /api/products

# Bad - Verbs or mixed conventions
GET /api/getUser
GET /api/user  (inconsistent singular)
POST /api/createOrder
```

### Nested Resources

```
# Shallow nesting (preferred)
GET /api/users/{id}/orders
GET /api/orders/{id}

# Deep nesting (avoid)
GET /api/users/{id}/orders/{orderId}/items/{itemId}/reviews
# Better:
GET /api/order-items/{id}/reviews
```

## HTTP Methods and Status Codes

### GET - Retrieve Resources

```
GET /api/users              → 200 OK (with list)
GET /api/users/{id}         → 200 OK or 404 Not Found
GET /api/users?page=2       → 200 OK (paginated)
```

### POST - Create Resources

```
POST /api/users
  Body: {"name": "John", "email": "john@example.com"}
  → 201 Created
  Location: /api/users/123
  Body: {"id": "123", "name": "John", ...}

POST /api/users (validation error)
  → 422 Unprocessable Entity
  Body: {"errors": [...]}
```

### PUT - Replace Resources

```
PUT /api/users/{id}
  Body: {complete user object}
  → 200 OK (updated)
  → 404 Not Found (doesn't exist)

# Must include ALL fields
```

### PATCH - Partial Update

```
PATCH /api/users/{id}
  Body: {"name": "Jane"}  (only changed fields)
  → 200 OK
  → 404 Not Found
```

### DELETE - Remove Resources

```
DELETE /api/users/{id}
  → 204 No Content (deleted)
  → 404 Not Found
  → 409 Conflict (can't delete due to references)
```

## Filtering, Sorting, and Searching

### Query Parameters

```
# Filtering
GET /api/users?status=active
GET /api/users?role=admin&status=active

# Sorting
GET /api/users?sort=created_at
GET /api/users?sort=-created_at  (descending)
GET /api/users?sort=name,created_at

# Searching
GET /api/users?search=john
GET /api/users?q=john

# Field selection (sparse fieldsets)
GET /api/users?fields=id,name,email
```

## Pagination Patterns

### Offset-Based Pagination

```json
GET /api/users?page=2&size=20

Response:
{
  "content": [...],
  "totalElements": 150,
  "totalPages": 8,
  "number": 2,
  "size": 20
}
```

### Cursor-Based Pagination (for large datasets)

```json
GET /api/users?limit=20&cursor=eyJpZCI6MTIzfQ

Response:
{
  "items": [...],
  "nextCursor": "eyJpZCI6MTQzfQ",
  "hasMore": true
}
```

### Link Header Pagination (RESTful)

```
GET /api/users?page=2

Response Headers:
Link: <https://api.example.com/users?page=3>; rel="next",
      <https://api.example.com/users?page=1>; rel="prev",
      <https://api.example.com/users?page=1>; rel="first",
      <https://api.example.com/users?page=8>; rel="last"
```

## Versioning Strategies

### URL Versioning (Recommended)

```
/api/v1/users
/api/v2/users

Pros: Clear, easy to route
Cons: Multiple URLs for same resource
```

### Header Versioning

```
GET /api/users
Accept: application/vnd.api+json; version=2

Pros: Clean URLs
Cons: Less visible, harder to test
```

### Query Parameter

```
GET /api/users?version=2

Pros: Easy to test
Cons: Optional parameter can be forgotten
```

## Rate Limiting

### Headers

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 742
X-RateLimit-Reset: 1640000000

Response when limited:
429 Too Many Requests
Retry-After: 3600
```

### Implementation Pattern

```java
// Using Bucket4j for rate limiting in Spring Boot
@Configuration
public class RateLimitConfig {

    @Bean
    public FilterRegistrationBean<RateLimitFilter> rateLimitFilter() {
        var registration = new FilterRegistrationBean<>(new RateLimitFilter());
        registration.addUrlPatterns("/api/*");
        return registration;
    }
}

public class RateLimitFilter extends OncePerRequestFilter {

    private final Map<String, Bucket> buckets = new ConcurrentHashMap<>();

    @Override
    protected void doFilterInternal(HttpServletRequest request,
                                     HttpServletResponse response,
                                     FilterChain chain) throws ServletException, IOException {
        String clientIp = request.getRemoteAddr();
        Bucket bucket = buckets.computeIfAbsent(clientIp, k -> createBucket());

        if (bucket.tryConsume(1)) {
            response.setHeader("X-RateLimit-Remaining",
                    String.valueOf(bucket.getAvailableTokens()));
            chain.doFilter(request, response);
        } else {
            response.setStatus(HttpStatus.TOO_MANY_REQUESTS.value());
            response.setHeader("Retry-After", "60");
        }
    }

    private Bucket createBucket() {
        return Bucket.builder()
                .addLimit(Bandwidth.classic(100, Refill.intervally(100, Duration.ofMinutes(1))))
                .build();
    }
}
```

## Authentication and Authorization

### Bearer Token

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...

401 Unauthorized - Missing/invalid token
403 Forbidden - Valid token, insufficient permissions
```

### API Keys

```
X-API-Key: your-api-key-here
```

## Error Response Format

### Consistent Structure

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": [
      {
        "field": "email",
        "message": "Invalid email format",
        "value": "not-an-email"
      }
    ],
    "timestamp": "2025-10-16T12:00:00Z",
    "path": "/api/users"
  }
}
```

### Status Code Guidelines

- `200 OK`: Successful GET, PATCH, PUT
- `201 Created`: Successful POST
- `204 No Content`: Successful DELETE
- `400 Bad Request`: Malformed request
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Authenticated but not authorized
- `404 Not Found`: Resource doesn't exist
- `409 Conflict`: State conflict (duplicate email, etc.)
- `422 Unprocessable Entity`: Validation errors
- `429 Too Many Requests`: Rate limited
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: Temporary downtime

## Caching

### Cache Headers

```
# Client caching
Cache-Control: public, max-age=3600

# No caching
Cache-Control: no-cache, no-store, must-revalidate

# Conditional requests
ETag: "33a64df551425fcc55e4d42a148795d9f25f89d4"
If-None-Match: "33a64df551425fcc55e4d42a148795d9f25f89d4"
→ 304 Not Modified
```

## Bulk Operations

### Batch Endpoints

```json
POST /api/users/batch
{
  "items": [
    {"name": "User1", "email": "user1@example.com"},
    {"name": "User2", "email": "user2@example.com"}
  ]
}

Response:
{
  "results": [
    {"id": 1, "status": "created"},
    {"id": null, "status": "failed", "error": "Email already exists"}
  ]
}
```

## Idempotency

### Idempotency Keys

```
POST /api/orders
Idempotency-Key: unique-key-123

If duplicate request:
→ 200 OK (return cached response)
```

## CORS Configuration

```java
@Configuration
public class CorsConfig implements WebMvcConfigurer {

    @Override
    public void addCorsMappings(CorsRegistry registry) {
        registry.addMapping("/api/**")
                .allowedOrigins("https://example.com")
                .allowedMethods("GET", "POST", "PUT", "PATCH", "DELETE")
                .allowedHeaders("*")
                .allowCredentials(true)
                .maxAge(3600);
    }
}
```

## Documentation with OpenAPI

```java
// SpringDoc OpenAPI auto-generates docs from annotations
// Available at /swagger-ui.html and /v3/api-docs

@RestController
@RequestMapping("/api/users")
@Tag(name = "Users", description = "User management API")
public class UserController {

    @Operation(
            summary = "Get user by ID",
            description = "Retrieve full user profile including basic information, contact details, and account status"
    )
    @ApiResponses({
            @ApiResponse(responseCode = "200", description = "User details"),
            @ApiResponse(responseCode = "404", description = "User not found")
    })
    @GetMapping("/{id}")
    public ResponseEntity<UserResponse> getUser(
            @Parameter(description = "The user ID") @PathVariable Long id) {
        return ResponseEntity.ok(userService.findById(id));
    }
}
```

## Health and Monitoring Endpoints

```yaml
# Spring Boot Actuator provides health endpoints out-of-the-box
# application.yml
management:
  endpoints:
    web:
      exposure:
        include: health, info, metrics, prometheus
  endpoint:
    health:
      show-details: when-authorized
      show-components: when-authorized
  health:
    db:
      enabled: true
    redis:
      enabled: true
    diskspace:
      enabled: true

# Endpoints available:
# GET /actuator/health           → {"status": "UP"}
# GET /actuator/health/db        → Database health
# GET /actuator/health/redis     → Redis health
# GET /actuator/info             → Application info
```
