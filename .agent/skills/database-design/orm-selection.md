# ORM / Data Access Selection (Java / Kotlin)

> Choose data access strategy based on project needs and query complexity.

## Decision Tree

```
What's the context?
│
├── Standard CRUD + Relationships
│   └── Spring Data JPA / Hibernate (most projects)
│
├── Complex SQL / Reporting / Analytics
│   └── jOOQ (type-safe SQL builder)
│
├── Legacy DB / Stored Procedures
│   └── MyBatis (XML/annotation SQL mapping)
│
├── Simple queries + Full control
│   └── Spring JdbcClient (Spring 6.1+)
│
└── Mix of approaches
    └── Spring Data JPA + jOOQ (CQRS pattern)
```

## Comparison

| ORM / Tool | Best For | Trade-offs |
|------------|----------|------------|
| **Spring Data JPA** | CRUD, relationships, rapid dev | N+1 risk, complex queries harder |
| **Hibernate** | Enterprise, complex mappings | Learning curve, performance tuning |
| **jOOQ** | Complex SQL, type-safe queries | Code generation step required |
| **MyBatis** | Legacy DB, stored procedures | Manual SQL mapping |
| **JdbcClient** | Simple queries, full control | No ORM features, manual mapping |

## Migration Tools

| Tool | Best For | Trade-offs |
|------|----------|------------|
| **Flyway** | SQL-based migrations, simple | Less rollback support |
| **Liquibase** | XML/YAML/JSON migrations, complex | More verbose, steeper learning |

## Questions to Ask

1. How complex are the queries?
2. Is rapid development or query control more important?
3. Do you need stored procedure support?
4. Is the schema already designed (code-first vs DB-first)?
5. Do you need type-safe SQL (consider jOOQ)?
