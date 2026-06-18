---
trigger: model_decision
description: "Testing rules covering JUnit 5 standards, Spring Boot test slices, Mockito patterns, Testcontainers, ArchUnit, and coverage targets for base-core."
---

# Testing Rules

> Testing standards for Java/Kotlin Spring Boot applications in base-core.

## Principles

- Test behavior, not implementation details
- Fast feedback — unit tests < 100ms, integration < 10s
- Reliable — no flaky tests in CI
- Comprehensive — cover happy path, edge cases, error paths
- Independent — tests must not depend on execution order

## Rules

### Test Framework Standards

- **JUnit 5** only — no JUnit 4 or TestNG
- Use `@MockitoBean` (NOT deprecated `@MockBean`)
- Use `@MockitoSpyBean` (NOT deprecated `@SpyBean`)
- Prefer `AssertJ` fluent assertions over JUnit `assertEquals`
- Use `@ParameterizedTest` for data-driven tests

### Test Organization

- Test class naming: `<ClassName>Test` for unit, `<ClassName>IT` for integration
- Test method naming: `should_<expected>_when_<condition>` pattern
- One assertion concept per test (multiple asserts OK if same concept)
- Group tests with `@Nested` for readability

### Test Slices (Spring Boot)

| Slice | Use When |
|-------|----------|
| `@WebMvcTest` | Testing controllers only (no DB, no service) |
| `@DataJpaTest` | Testing JPA repositories only |
| `@JsonTest` | Testing JSON serialization/deserialization |
| `@SpringBootTest` | Full integration tests |

- Prefer **test slices** over `@SpringBootTest` for speed
- Use `@SpringBootTest` only when testing cross-layer behavior

### Testcontainers

- Use Testcontainers for database integration tests (PostgreSQL)
- Use Testcontainers for Redis integration tests
- Prefer `@ServiceConnection` for auto-configuration
- Share containers across test classes with `@Container` + `static`

### ArchUnit Architecture Tests

- Enforce layer dependency rules (domain ≠ depend on infrastructure)
- Enforce naming conventions (DTOs end with Request/Response)
- Enforce annotation usage (no `@Autowired` on fields)
- Run ArchUnit tests in CI pipeline

### Coverage Targets

| Layer | Target |
|-------|--------|
| Domain / Business Logic | > 90% |
| Service Layer | > 85% |
| Controller Layer | > 80% |
| Overall Project | > 80% |

- Use **JaCoCo** for coverage measurement
- Coverage gate in CI — fail build if below threshold

### Test Data

- Use **Builder pattern** or **Factory methods** for test data
- Never use production data in tests
- Use `@Sql` or Flyway for database state setup
- Clean up test data with `@Transactional` rollback

## Anti-Patterns

- ❌ Using `@MockBean` (deprecated in Spring Boot 4.x)
- ❌ Testing private methods directly
- ❌ Tests with `Thread.sleep()` for async waiting
- ❌ Tests depending on execution order
- ❌ Integration tests without Testcontainers (H2 != PostgreSQL)
- ❌ Ignoring/disabling failing tests without tracking issue
- ❌ Testing framework code (Spring, Hibernate) instead of your code
- ❌ No assertions in test methods (false green)

## References

- JPA patterns: [skills/jpa-patterns/](../skills/jpa-patterns/)
- Code review: [agents/code-reviewer.md](../agents/code-reviewer.md)