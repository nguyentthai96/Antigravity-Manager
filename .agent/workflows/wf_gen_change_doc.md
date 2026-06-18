---
description: Generate Database-changes.md from archived change — DDL and DML scripts for deployment
---

Generate `Database-changes.md` — tài liệu deploy đầy đủ bao gồm **DB scripts** + **Config file changes** + **Spring Cloud Config changes**, dựa trên code changes trong archived openspec change.

**Pipeline position**: ... → `/wf_openspec_apply` → `/wf_archive` → **`/wf_gen_change_doc`**

> **Generic workflow** — Tự scan JPA entities, config files, @Value annotations. Dùng `knowledge_deploy_schema.md` nếu tồn tại.

---

**Input**: Argument after `/wf_gen_change_doc` is the archive directory name inside `openspec/changes/archive/` (e.g., `/wf_gen_change_doc 2026-03-30-ngung-dich-vu-alias`).

**Output**: `openspec/changes/archive/<name>/Database-changes.md`

**Steps**

## 1. Load archive context

a. **Validate** archive exists at `openspec/changes/archive/<name>/`
b. **Read** files from archive:
   - `tasks.md` — actual code changes list (source of truth)
   - `walkthrough.md` — implementation diffs (if exists)
   - `proposal.md` — change overview
   - `design.md` — technical design
   - `bundle-manifest.md` — bundled artifacts (if exists)

c. **Extract scope**: From tasks.md, build list of:
   - Files modified (exact paths)
   - What changed in each file
   - Routing/MID changes, config keys, error/message codes

## 2. Discover project DB schema (Dynamic)

### 2a. Check existing knowledge
Look for `base_knowledge/structures/apply/knowledge_db_schema.md`.
**If exists** → Load, skip to Step 3.
**If NOT exists** → Scan codebase (Steps 2b-2e), then suggest saving.

### 2b–2e. Auto-scan JPA entities, routing, config, message tables
_(Same as before — scan `@Table`, `@Column`, trace dispatch/config/message patterns)_

Scan all `@Table(name = "...")` annotations → build schema map:
```
TABLE_MAP = { EntityClass → { table, pk, columns[], status_column } }
```

## 3. Scan actual source code for DB impacts

_(Steps 3a–3d same as before — routing, config DB, message DB, DDL)_

## 4. Scan config file changes ⚡ NEW

> Tài liệu deploy PHẢI bao gồm mọi thay đổi config file, không chỉ DB.

### 4a. Discover project config files

**Auto-scan** config file locations:
```bash
# Common locations
find . -name "application*.yml" -o -name "application*.yaml" -o -name "application*.properties"
find . -name "bootstrap*.yml" -o -name "bootstrap*.properties"
find . -path "*/conf/*" -name "*.properties" -o -name "*.yml" -o -name "*.xml"
find . -path "*/resources/*" -name "*.properties" -o -name "*.yml"
```

**Or** from `knowledge_db_schema.md` → Config Files section (if exists).

**Typical project config file map**:
| File | Location | Purpose | Managed by |
|------|----------|---------|------------|
| `application.yml` | `conf/` or `src/main/resources/` | Spring Boot main config | DevOps |
| `config.properties` | `conf/` | Legacy app config, API endpoints | DevOps |
| `config.local.properties` | `conf/` | Secrets, credentials (NOT in Git) | DevOps |
| `logback-spring.xml` | `conf/` | Logging config | DevOps |
| `bootstrap.yml` | `src/main/resources/` | Spring Cloud Bootstrap | DevOps |

### 4b. Check for config file changes via git

```bash
# Get list of changed config files relative to main/develop branch
git diff --name-only HEAD~N -- "*.yml" "*.yaml" "*.properties" "*.xml"

# Or check specific config directories
git diff --name-only HEAD~N -- conf/ src/main/resources/
```

**If git not available** → Scan changed files from `tasks.md` / `walkthrough.md` for config references.

### 4c. Analyze each changed config file

For each changed config file, extract **specific changes** using git diff or by comparing with known state:

**For YAML files** (`application.yml`, etc.):
```bash
git diff HEAD~N -- conf/application.yml
```

Extract:
- **New keys added** (new YAML paths)
- **Keys modified** (value changed)
- **Keys removed** (deleted lines)
- **Sections restructured**

**For Properties files** (`config.properties`, etc.):
```bash
git diff HEAD~N -- conf/config.properties
```

Extract:
- **New properties** (new key=value lines)
- **Modified properties** (value changed)
- **Commented out** (# prefix added)
- **Removed properties**

**For XML files** (`logback-spring.xml`, etc.):
- Element added/removed
- Attribute changes

### 4d. Analyze `@Value` annotation changes

Scan changed Java files for new or modified `@Value` annotations:
```java
@Value("${some.new.property}")        // New property needed
@Value("${existing.prop:new-default}")  // Default changed
```

Cross-reference:
- Each `@Value("${key}")` → does `key` exist in config files?
- If NO → flag as **"Missing config — must add before deploy"**
- If YES → check if value is appropriate for each environment

### 4e. Analyze Spring Cloud Config changes

If project uses Spring Cloud Config (`spring.config.import` or `spring.cloud.config`):

```yaml
spring:
  cloud:
    config:
      name: sgb-mb-2345-redis, sgb-mb-database, ...  # Config profiles
```

Check if:
- New config profile names added to `spring.cloud.config.name`
- Properties that should come from Config Server vs local file
- Environment-specific overrides needed

## 5. Generate Database-changes.md

Create the file at `openspec/changes/archive/<name>/Database-changes.md`.

**Full template** (dynamic table names + config file changes):

````markdown
# Deployment Changes — <change-name>

_Generated on YYYY-MM-DD based on archived change `<name>`_
_DB Type: <Oracle|PostgreSQL|MySQL>_

## Summary

| Category | Detail | Count | Action |
|----------|--------|-------|--------|
| DB — Routing | <ROUTING_TABLE> | N | INSERT/UPDATE/DELETE |
| DB — Config | <CONFIG_TABLE> | N | INSERT/UPDATE/DELETE |
| DB — Messages | <MESSAGE_TABLE> | N | INSERT/UPDATE/DELETE |
| DB — DDL | (schema) | N | CREATE/ALTER/DROP |
| Config File | application.yml | N | ADD/MODIFY/REMOVE keys |
| Config File | config.properties | N | ADD/MODIFY/REMOVE keys |
| Spring Cloud | Config Server | N | Profile/override changes |

---

## Part A: Database Changes

### A1. <ROUTING_TABLE> — Routing Changes

```sql
-- Disable/Enable routes
UPDATE <TABLE> SET <STATUS_COL> = '<value>'
WHERE <PK> IN ('...');

-- Rollback
UPDATE <TABLE> SET <STATUS_COL> = '<original>'
WHERE <PK> IN ('...');
```

### A2. <CONFIG_TABLE> — DB Config Changes

```sql
-- INSERT / UPDATE / Rollback
```

### A3. <MESSAGE_TABLE> — Message Changes

```sql
-- INSERT / UPDATE / Rollback
```

### A4. DDL — Schema Changes

```sql
-- CREATE TABLE / ALTER TABLE / Rollback
```

---

## Part B: Config File Changes

### B1. `conf/application.yml`

**Changes**:
| Action | YAML Path | Old Value | New Value | Environment Notes |
|--------|-----------|-----------|-----------|-------------------|
| ADD | `credit-card.api.paths.get-cvv` | — | `/api/v1/cards/get-cvv` | All environments |
| MODIFY | `credit-card.api.host` | `http://old-host` | `http://new-host` | DEV only, UAT/PROD uses Config Server |
| REMOVE | `alias.api.host` | `http://alias-gw` | — | Removed per TT30/2025 |

**Diff** (for DevOps to apply manually):
```yaml
# ADD the following under credit-card.api.paths:
credit-card:
  api:
    paths:
+     get-cvv: /api/v1/cards/get-cvv

# REMOVE the following section:
- alias:
-   api:
-     host: http://alias-gateway
```

### B2. `conf/config.properties`

**Changes**:
| Action | Property Key | Old Value | New Value | Notes |
|--------|-------------|-----------|-----------|-------|
| ADD | `new_api_url` | — | `http://...` | New integration endpoint |
| MODIFY | `qr_bank_code` | `970428` | `970428` | No change (verify) |

**Diff**:
```properties
# ADD
+ new_api_url = http://new-endpoint:port/api
```

### B3. `conf/config.local.properties` (Secrets)

> ⚠️ File này KHÔNG commit Git — DevOps phải thêm thủ công trên từng môi trường.

| Action | Property Key | Description | Who to get value from |
|--------|-------------|-------------|----------------------|
| ADD | `new_api_secret_key` | API key cho service mới | Team Lead / SA |

### B4. Spring Cloud Config Server Changes

| Config Profile | Property | Value | Notes |
|---------------|----------|-------|-------|
| `sgb-mb-corebank-alias` | — | — | Profile có thể xóa/disable vì alias đã ngưng |

### B5. `@Value` Properties Verification

> Kiểm tra tất cả `@Value` mới/thay đổi đã có trong config file chưa.

| `@Value` Expression | Found In File | Status |
|---------------------|---------------|--------|
| `${credit-card.api.paths.get-cvv:}` | `conf/application.yml` ✅ | OK |
| `${new.property.key}` | ❌ MISSING | 🔴 Must add before deploy |

---

## Part C: Execution Order

> ⚠️ Execute in this exact order on each environment.

| # | Type | Target | Action | Notes |
|---|------|--------|--------|-------|
| 1 | DDL | Database | Run schema scripts | Before anything |
| 2 | DML | <CONFIG_TABLE> | Insert/Update config rows | Before code deploy |
| 3 | DML | <MESSAGE_TABLE> | Insert/Update messages | Before code deploy |
| 4 | Config | `application.yml` | Apply YAML changes | On app server |
| 5 | Config | `config.properties` | Apply property changes | On app server |
| 6 | Config | `config.local.properties` | Add secrets manually | On app server |
| 7 | Config | Spring Cloud Config | Update config profiles | On Config Server |
| 8 | Deploy | Application | Deploy new JAR/WAR | Restart required |
| 9 | DML | <ROUTING_TABLE> | Enable/disable routes | After successful deploy |
| 10 | Post | Cache | Clear application cache | If cached tables changed |

---

## Part D: Verification

### D1. Database Verification

```sql
SELECT <PK>, <KEY_COLS> FROM <TABLE> WHERE <PK> IN (...);
```

### D2. Config File Verification

```bash
# Verify YAML values on target server
grep -A1 "get-cvv" /path/to/conf/application.yml

# Verify properties
grep "new_api_url" /path/to/conf/config.properties
```

### D3. Application Health Check

```bash
# After deploy — verify app starts without error
curl -s http://localhost:PORT/actuator/health
# Check for @Value resolution errors in log
grep "Could not resolve placeholder" /path/to/logs/app.log
```

---

## Affected Environments

| Environment | DB Scripts | Config Files | Config Server | Deploy | Status |
|-------------|------------|-------------|---------------|--------|--------|
| DEV | [ ] | [ ] | [ ] | [ ] | ⬜ |
| SIT | [ ] | [ ] | [ ] | [ ] | ⬜ |
| UAT | [ ] | [ ] | [ ] | [ ] | ⬜ |
| PRODUCTION | [ ] | [ ] | [ ] | [ ] | ⬜ |

## Notes

- DBA review required before UAT/PROD
- DevOps must apply config file changes BEFORE deploying new code
- Secrets in `config.local.properties` must be added manually — NOT in Git
- Cache clear required after: <list cached tables changed>
- Spring Cloud Config Server changes require Config Server restart/refresh
- All DB scripts include rollback per section
- Rollback procedure: Revert config files → Deploy old code → Run DB rollback scripts
````

## 6. Verify & save

a. **Verify** SQL syntax matches project DB type
b. **Verify** all `@Value` properties have corresponding config entries
c. **Save** to `openspec/changes/archive/<name>/Database-changes.md`
d. **Show summary** with counts per category

---

**Output On Success**

```
## Deployment Changes Generated

**Archive:** <name>
**File:** openspec/changes/archive/<name>/Database-changes.md

### Summary
| Category | Items | Rollback |
|----------|-------|----------|
| DB — Routing | 4 updates | ✓ |
| DB — Config | 1 insert | ✓ |
| DB — Messages | 0 | — |
| DB — DDL | 0 | — |
| Config — application.yml | 3 keys changed | ✓ (diff provided) |
| Config — config.properties | 1 key added | ✓ |
| Config — Secrets | 0 | — |
| Config — Spring Cloud | 0 | — |
| @Value verification | 26 checked, 0 missing | ✅ |

Execution order (10 steps) and verification included.
```

---

**Guardrails**

- MUST read `tasks.md` first — source of truth for actual code changes
- MUST scan actual source code for DB impacts — do NOT guess
- MUST discover table schemas dynamically (JPA scan) or from `knowledge_db_schema.md`
- MUST detect and document ALL config file changes — not just DB
- MUST scan `@Value` annotations in changed files and cross-check with config files
- MUST flag missing config entries as 🔴 blockers
- MUST include clear diff snippets for each config file change
- MUST separate secrets (`config.local.properties`) from regular config — mark as manual
- MUST include Spring Cloud Config profile changes if project uses Config Server
- MUST include complete execution order with all 3 types (DB → Config → Deploy → Post)
- MUST include verification steps for DB, config files, AND application health
- MUST include environment checklist with checkboxes for each type
- MUST include rollback procedure covering all 3 types
- DO NOT hardcode table names — derive from scan or knowledge
- DO NOT include changes with no deployment impact (code-only refactors)
- If uncertain about config values per environment → add note for DevOps to verify
- If `knowledge_db_schema.md` missing → suggest saving after first scan
