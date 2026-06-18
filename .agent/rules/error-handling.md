---
trigger: model_decision
description: "Error handling rules covering RFC 7807 ProblemDetail, exception hierarchy, @ControllerAdvice, and error response standards for base-core."
---

# Error Handling Rules

> Standardized error handling for Java/Kotlin Spring Boot applications in base-core.

## Principles

- Consistent error format across all APIs
- Rich error details for debugging, safe details for clients
- Fail fast — validate early, throw specific exceptions
- Never leak stack traces or internal details to API consumers

## Rules

### Error Response Format (RFC 7807)

- ALL API errors **must** return RFC 7807 `ProblemDetail` format
- Use Spring's built-in `ProblemDetail` class (not custom error objects)
- Required fields: `type`, `title`, `status`, `detail`, `instance`

```java
// ✅ Correct — Spring Boot 4.x ProblemDetail
ProblemDetail problem = ProblemDetail.forStatusAndDetail(
    HttpStatus.NOT_FOUND,
    "User with id 123 not found"
);
problem.setType(URI.create("https://api.example.com/errors/user-not-found"));
problem.setTitle("User Not Found");
problem.setProperty("userId", 123);
```

### Exception Hierarchy

- Create a base `BusinessException` extending `RuntimeException`
- Domain-specific exceptions extend `BusinessException`
- Use specific exception types — not generic `RuntimeException`

```
RuntimeException
└── BusinessException (base, abstract)
    ├── EntityNotFoundException
    ├── DuplicateEntityException
    ├── BusinessRuleViolationException
    └── InsufficientPermissionException
```

### Global Exception Handling

- Use `@ControllerAdvice` + `@ExceptionHandler` for all error handling
- ONE global exception handler per application (no scattered handlers)
- Map exceptions to appropriate HTTP status codes
- Log full exception details server-side, return safe message client-side

| Exception Type | HTTP Status |
|---------------|------------|
| `EntityNotFoundException` | 404 |
| `DuplicateEntityException` | 409 |
| `BusinessRuleViolationException` | 422 |
| `InsufficientPermissionException` | 403 |
| `MethodArgumentNotValidException` | 400 |
| `Unexpected exceptions` | 500 |

### Validation Errors

- Use Bean Validation (`@Valid`, `@NotNull`, `@Size`, etc.) on DTOs
- Return **field-level** error details in 400 responses
- Include `field`, `rejectedValue`, and `message` for each violation

### Logging on Errors

- Log `WARN` for client errors (4xx)
- Log `ERROR` for server errors (5xx) with full stack trace
- Include correlation ID / trace ID in error logs
- Never log sensitive data (passwords, tokens, PII)

## Anti-Patterns

- ❌ Returning raw exception messages to clients
- ❌ Catching generic `Exception` without re-throwing or proper handling
- ❌ Using `@ResponseStatus` on exceptions (use `@ControllerAdvice` instead)
- ❌ Multiple `@ControllerAdvice` classes without clear ordering
- ❌ Empty catch blocks that swallow exceptions silently
- ❌ Returning `gin.H{}` / `Map<String, Object>` instead of `ProblemDetail`
- ❌ Mixing error formats across different APIs

## References

- Spring Boot rules: [rules/spring-boot.md](spring-boot.md)
- API design: [skills/api-design-principles/](../skills/api-design-principles/)
- Logging rules: [rules/logging.md](logging.md)
