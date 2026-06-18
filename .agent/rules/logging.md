---
trigger: model_decision
description: "Logging rules covering structured JSON logging, SLF4J standards, MDC context propagation, log levels, and sensitive data masking for base-core."
---

# Logging Rules

> Logging standards for Java/Kotlin Spring Boot applications in base-core.

## Principles

- Structured logging (JSON) by default — human-readable on demand
- Every request must be traceable end-to-end
- Log at the right level — not too much, not too little
- Never log sensitive data

## Rules

### Framework

- Use **SLF4J** as logging facade — never import `java.util.logging` or Log4j directly
- Use `@Slf4j` (Lombok) or `LoggerFactory.getLogger()` for logger creation
- Use **Logback** as default implementation (Spring Boot default)

### Structured Logging (JSON)

- Use Spring Boot's built-in structured logging:

```yaml
# application.yml
logging:
  structured:
    format:
      console: logstash   # JSON output for production
```

- Include standard fields: `timestamp`, `level`, `logger`, `message`, `traceId`, `spanId`
- Add business context with MDC (Mapped Diagnostic Context)

### Log Levels

| Level | Use When | Example |
|-------|----------|---------|
| `ERROR` | System failure, requires immediate attention | DB connection lost, external API 500 |
| `WARN` | Unexpected but recoverable | Cache miss, retry triggered, 4xx client error |
| `INFO` | Business events, state changes | User created, order placed, payment processed |
| `DEBUG` | Technical details for troubleshooting | SQL queries, request/response payloads |
| `TRACE` | Very detailed flow tracing | Method entry/exit, variable values |

- Production: `INFO` level by default
- Debug: Enable `DEBUG` per-package, never globally

### MDC Context Propagation

- Set `userId`, `requestId`, `correlationId` in MDC at request entry
- Clear MDC at request completion (use `Filter` or `Interceptor`)
- Propagate MDC across async boundaries (`@Async`, `CompletableFuture`)
- Include MDC fields in structured log output

### What to Log

- ✅ Request received (method, path, user)
- ✅ Business events (entity created/updated/deleted)
- ✅ External service calls (URL, duration, status)
- ✅ Errors and exceptions (full stack trace at ERROR level)
- ✅ Performance metrics (slow queries > threshold)

### What NOT to Log

- ❌ Passwords, tokens, API keys, secrets
- ❌ PII (email, phone, national ID) — mask if required
- ❌ Credit card numbers (PCI-DSS violation)
- ❌ Full request/response bodies in production (use DEBUG level)
- ❌ Health check requests (noise)

### Performance

- Use parameterized logging: `log.info("User {} created", userId)` — NOT string concatenation
- Guard expensive log computations: `if (log.isDebugEnabled())`
- Avoid logging in tight loops
- Set appropriate log rotation and retention

## Anti-Patterns

- ❌ Using `System.out.println()` for logging
- ❌ String concatenation in log messages: `log.info("User " + id + " created")`
- ❌ Logging sensitive data (passwords, tokens, PII)
- ❌ Setting `DEBUG` or `TRACE` globally in production
- ❌ Missing correlation IDs in distributed systems
- ❌ Swallowing exceptions: `catch (Exception e) { log.error("error") }` — log the exception!
- ❌ Logging at wrong level (INFO for debug details, ERROR for business validation)

## References

- Spring Boot rules: [rules/spring-boot.md](spring-boot.md)