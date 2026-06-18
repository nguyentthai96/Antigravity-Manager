---
trigger: always_on
description: "SQL query optimization, database design, and data modeling rules. Covers indexing, execution plans, normalization, and naming conventions."
---

# Database Rules

> Rules for SQL query optimization, database design, and data modeling in base-core.

## Principles

- Fetch only what you need
- Index effectively
- Understand the execution plan
- Minimize database round-trips
- Design for data integrity first
- Normalize to reduce redundancy; denormalize consciously for performance

## Rules

### Query Optimization

- SELECT specific columns, never `SELECT *`
- Use WHERE clauses to filter early
- Use LIMIT for large result sets
- Avoid functions on indexed columns in WHERE clauses
- Use JOINs appropriately (prefer INNER over OUTER when possible)
- Use EXISTS instead of IN for correlated subqueries
- Batch inserts and updates for bulk operations
- Use Prepared Statements for all parameterized queries

### Indexing

- Index columns used in WHERE, JOIN, ORDER BY
- Use Composite Indexes following the Leftmost Prefix Rule
- Use Covering Indexes to avoid heap lookup
- Remove unused indexes regularly
- Monitor index fragmentation

### Execution Plans

- Use `EXPLAIN` / `EXPLAIN ANALYZE` before optimizing
- Identify and eliminate Full Table Scans (Seq Scan)
- Check for high-cost operations (Sort, Hash Join)
- Verify actual vs estimated row counts

### Normalization

- **1NF**: Atomic values, unique rows
- **2NF**: No partial dependencies (composite keys)
- **3NF**: No transitive dependencies
- **BCNF**: Stricter 3NF for complex schemas
- Apply 4NF/5NF when dealing with multi-valued dependencies

### Denormalization (When Justified)

- Pre-computed aggregates for dashboards
- Materialized Views for expensive joins
- Redundant columns for critical read paths
- JSON columns for flexible/schema-less data
- Always document the reason for denormalization

### Naming Conventions

- Tables: use consistent casing (prefer `snake_case`, e.g., `user_profiles`)
- Columns: `snake_case` (e.g., `user_id`, `created_at`)
- Primary keys: `pk_<table>`
- Foreign keys: `fk_<table>_<column>`
- Indexes: `idx_<table>_<column>`

### Data Integrity

- Use standard ISO 8601 for dates
- Use UTC for all timestamps
- Avoid reserved words for table/column names
- Validate data at both application AND database level
- Consider GDPR/Privacy requirements in schema design
- Plan for schema evolution (use migration tools)

## Anti-Patterns

- ❌ N+1 Queries (looping queries in application code)
- ❌ Implicit type conversion in WHERE clauses
- ❌ Leading wildcard in LIKE (`'%value'` defeats indexes)
- ❌ OR conditions that prevent index usage
- ❌ Large transactions holding locks for extended periods
- ❌ Using `SELECT *` in production queries
- ❌ Missing indexes on foreign key columns

## References

- Database architecture: [skills/database-architect/](../skills/database-architect/)
- Database design: [skills/database-design/](../skills/database-design/)
- JPA patterns: [skills/jpa-patterns/](../skills/jpa-patterns/)