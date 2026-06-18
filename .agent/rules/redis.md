---
trigger: always_on
description: "Redis caching strategies, data structure selection, and operational best practices."
---

# Redis Rules

> Rules for Redis usage, caching strategies, and data structure selection in base-core.

## Principles

- Cache for read-heavy workloads
- Handle cache invalidation correctly
- Use appropriate data structures for each use case
- Ensure high availability
- Monitor memory usage continuously

## Rules

### Data Structure Selection

| Structure | Use Case | Example |
|-----------|----------|---------|
| Strings | Simple key-value, counters | Session tokens, feature flags |
| Lists | Queues, recent items | Task queues, activity feeds |
| Sets | Unique items, tags | User roles, online users |
| Sorted Sets | Leaderboards, priority queues | Rankings, scheduled tasks |
| Hashes | Objects, profiles | User profiles, configs |
| Streams | Event logs, messaging | Audit logs, real-time events |
| HyperLogLog | Cardinality estimation | Unique visitor counts |

### Caching Patterns

- **Cache-Aside (Lazy Loading)**: App checks cache first, falls back to DB
- **Write-Through**: App writes to cache and DB synchronously
- **Write-Behind**: App writes to cache, async flush to DB
- **Cache Stampede Prevention**: Use locking or probabilistic early expiration

### Persistence & Durability

- **RDB** (Snapshots): Point-in-time backups for disaster recovery
- **AOF** (Append Only File): Log every write for durability
- **Hybrid** (RDB + AOF): Best of both worlds for production
- Disable persistence entirely for pure cache workloads

### High Availability

- Use Redis Sentinel for automatic failover
- Use Redis Cluster for horizontal sharding
- Configure appropriate eviction policies (`allkeys-lru`, `volatile-ttl`)

### Operations

- Set TTL on **all** cache keys (no orphaned data)
- Use key namespacing with colons (`user:123:profile`)
- Monitor cache hit/miss ratio continuously
- Never use `KEYS` command in production (use `SCAN` instead)
- Pipeline commands for improved throughput
- Use Lua scripting for atomic multi-step operations
- Secure with password and ACL rules

## Anti-Patterns

- ❌ Keys without TTL (memory leak risk)
- ❌ Using `KEYS *` in production (blocks Redis)
- ❌ Storing large objects without compression
- ❌ Using Redis as primary database (it's a cache/store, not RDBMS)
- ❌ Ignoring eviction policies until OOM