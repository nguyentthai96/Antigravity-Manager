---
trigger: always_on
---

# Base-Core Rules

> Project-specific coding standards enforced across all agents and skills.

## Rule Inventory (10 rules)

### Always Active (loaded for every conversation)
| Rule | Scope | Description |
|------|-------|-------------|
| `first-rules.md` | Meta | Ngôn ngữ giao tiếp AI, reuse-first, architecture compliance, incremental changes |
| `spring-boot.md` | Backend | Spring Boot standards — DI, annotations, naming, config, testing |
| `database.md` | Data | SQL optimization, indexing, normalization, naming conventions |
| `redis.md` | Cache | Data structure selection, caching patterns, TTL, key namespacing |
| `refactoring.md` | Quality | Safe refactoring process, code smells, when NOT to refactor |

### Conditional (activated by trigger)
| Rule | Trigger | Description |
|------|---------|-------------|
| `api-security.md` | Security tasks | OAuth2/JWT, token management, encryption, OWASP |
| `testing.md` | Testing tasks | JUnit 5, `@MockitoBean`, test slices, Testcontainers, ArchUnit, coverage |
| `logging.md` | Logging tasks | Structured JSON, SLF4J, MDC propagation, log levels, data masking |
| `error-handling.md` | Error handling tasks | RFC 7807 ProblemDetail, exception hierarchy, @ControllerAdvice, error response standards |

## Rule Format

Each rule follows this structure:
```markdown
---
trigger: model_decision          # When to activate
description: "One-line summary"  # For AI indexing
---
# Rule Title
## Principles                    # Why these rules exist
## Rules                         # What to do
## Anti-Patterns                 # What NOT to do
## References                    # Links to skills/agents
```

## Related

- **Skills** (`skills/`): Detailed implementation guides and code examples
- **Agents** (`agents/`): Agent personas that reference these rules
