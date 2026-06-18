# Database Selection (Java / Spring Boot)

> Choose database based on context, not default. For Spring Boot projects, PostgreSQL is the standard choice.

## Decision Tree

```
What are your requirements?
│
├── Full relational features needed
│   ├── Standard deployment → PostgreSQL (recommended)
│   ├── Cloud managed → AWS RDS / Azure Database / Cloud SQL
│   └── MySQL required → MySQL 8+ / MariaDB
│
├── AI / Vector search
│   └── PostgreSQL + pgvector
│
├── High-throughput time-series
│   └── TimescaleDB (PostgreSQL extension)
│
├── Document store / flexible schema
│   └── MongoDB (Spring Data MongoDB)
│
├── Caching / Session store
│   └── Redis (Spring Data Redis)
│
└── Global distribution / NewSQL
    └── CockroachDB, YugabyteDB, Google Spanner
```

## Comparison

| Database | Best For | Spring Integration |
|----------|----------|--------------------|
| **PostgreSQL** | Full features, JSONB, extensions | Spring Data JPA, HikariCP |
| **MySQL** | Web apps, read-heavy workloads | Spring Data JPA, HikariCP |
| **MongoDB** | Flexible schema, document data | Spring Data MongoDB |
| **Redis** | Caching, sessions, pub/sub | Spring Data Redis |
| **TimescaleDB** | Time-series, IoT data | JPA compatible (PG extension) |

## Questions to Ask

1. What's the deployment environment?
2. How complex are the queries / relationships?
3. Does the project require ACID transactions?
4. Is full-text search or vector search needed?
5. What's the expected data volume and growth?
