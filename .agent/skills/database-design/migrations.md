# Migration Principles (Java / Spring Boot)

> Safe migration strategy for zero-downtime changes using Flyway or Liquibase.

## Safe Migration Strategy

```
For zero-downtime changes:
│
├── Adding column
│   └── Add as nullable → backfill → add NOT NULL
│
├── Removing column
│   └── Stop using → deploy → remove column
│
├── Adding index
│   └── CREATE INDEX CONCURRENTLY (non-blocking, PostgreSQL)
│
└── Renaming column
    └── Add new → migrate data → deploy → drop old
```

## Migration Philosophy

- Never make breaking changes in one step
- Test migrations on data copy first
- Have rollback plan
- Run in transaction when possible

## Flyway (Recommended for SQL-first)

```
src/main/resources/db/migration/
├── V1__create_users_table.sql
├── V2__add_email_index.sql
├── V3__create_orders_table.sql
└── V4__add_status_column.sql
```

```yaml
# application.yml
spring:
  flyway:
    enabled: true
    locations: classpath:db/migration
    baseline-on-migrate: true
```

## Liquibase (For XML/YAML changelogs)

```yaml
# src/main/resources/db/changelog/db.changelog-master.yaml
databaseChangeLog:
  - changeSet:
      id: 1
      author: dev
      changes:
        - createTable:
            tableName: users
            columns:
              - column:
                  name: id
                  type: bigint
                  autoIncrement: true
                  constraints:
                    primaryKey: true
```

## Tool Comparison

| Feature | Flyway | Liquibase |
|---------|--------|-----------|
| Migration format | SQL files | XML/YAML/JSON/SQL |
| Rollback support | Limited (paid) | Built-in |
| Learning curve | Low | Medium |
| Spring Boot support | Excellent | Excellent |
| Best for | SQL-first teams | Complex changelogs |
