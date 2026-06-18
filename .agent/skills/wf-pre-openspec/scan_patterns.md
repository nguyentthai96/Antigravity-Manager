# Scan Patterns & Context File Templates (Phase B)

> Extracted from `wf-pre-openspec/SKILL.md` — referenced by Step 8 and Step 9.
> DO NOT modify without updating SKILL.md references.

---

## Step 8 — Atomic Scan Patterns

### 8a. Base Class Scan

**grep_search** patterns by language:
| Language | Patterns |
|-------------|-----------------------------------------------------------------------------------|
| Java/Kotlin | `class Base`, `extends Base`, `interface IAgw`, `interface I*Repository`          |
| Python | `class Base`, `(BaseView)`, `(APIView)`, `(ModelSerializer)`, `class *Repository` |
| TypeScript | `extends Base`, `implements I`, `@Injectable`, `abstract class`                   |
| Default | `class Base`, `extends Base`                                                      |

> Resolve `{PROJECT_LANG}` from LANGUAGE DETECTION above → pick matching row.

**grep_search** for: `class Base`, `extends Base`, `interface IAgw`, `interface I*Repository`

**OUTPUT per detected class:**

```
- ClassName
  - path: <relative-file-path>
  - extends: <parent> (or N/A)
  - implements: <interfaces> (or N/A)
```

If none found → `NOT DETECTED`

### 8b. Package Structure Scan

List ALL packages under candidate services. Group by:

| Layer      | Pattern         | Status            |
|------------|-----------------|-------------------|
| Controller | `*/controller/` | FOUND / NOT FOUND |
| Handler    | `*/handler/`    | FOUND / NOT FOUND |
| Factory    | `*/factory/`    | FOUND / NOT FOUND |
| Model      | `*/model/`      | FOUND / NOT FOUND |
| Entity     | `*/entity/`     | FOUND / NOT FOUND |
| Repository | `*/repository/` | FOUND / NOT FOUND |

Also list non-standard packages (e.g., `service/`, `dto/`).

### 8c. Integration Scan

**grep_search** patterns by language:
| Language | Patterns |
|-------------|------------------------------------------------------------------------------------------------------------------------|
| Java/Kotlin | `RestTemplate`, `WebClient`, `FeignClient`, `Gateway`, `Client`, `Redis`, `Cache`, `Kafka`, `RabbitMQ`,
`JdbcTemplate` |
| Python | `requests.`, `httpx`, `aiohttp`, `redis`, `celery`, `kafka`, `SQLAlchemy`,
`django.db`                                 |
| TypeScript | `HttpService`, `axios`, `fetch(`, `Redis`, `Bull`, `TypeORM`,
`Prisma`                                                 |
| Default | `Client`, `Gateway`, `Redis`, `Cache`, `Kafka`,
`http`                                                                 |

**OUTPUT per integration:**

```
- type: <REST/Cache/MQ/DB>
  - class: <ClassName> — <path>
  - protocol: <detected>
```

If none → `NOT DETECTED`

### 8d. DTO & Validation Scan

**grep_search** patterns by language:
| Language | Patterns |
|-------------|----------------------------------------------------------------------------------------|
| Java/Kotlin | `extends Base*Request`, `Response`, `Filter`, `@Getter`, `@Builder`, `@SuperBuilder`   |
| Python | `@dataclass`, `BaseModel`, `Serializer`, `Schema`, `class *Request`, `class *Response` |
| TypeScript | `interface *Request`, `interface *Response`, `class *Dto`, `@IsString`, `@IsNotEmpty`  |
| Default | `Request`, `Response`, `Filter`, `Dto`

**OUTPUT:**

```
- Request: <ClassName> extends <Base> — <path>
- Response: <ClassName> — <path>
- Annotations: <list>
```

If none → `NOT DETECTED`

### 8e. Error Pattern Scan

**grep_search** for: `Exception`, `extends RuntimeException`, `ErrorCode`, `ERROR_`

**OUTPUT:**

```
- Exception: <ClassName> — <path>
- Error code format: <pattern> — <example constant>
```

If none → `NOT DETECTED`

### Step 8 VALIDATION

- [ ] Every listed class has real file path
- [ ] No invented/example class names
- [ ] `NOT DETECTED` used where nothing found
- [ ] All scans within scope (candidate services only)

---

## Step 9 — Context File Templates

Write to: `openspec/context/generated/<feature-name>/`

**OUTPUT RULE:** ONLY include detected items. NO predefined examples. `NOT DETECTED` for empty sections.

### File 1: `base_class_map.md`

```markdown
# Base Class Map

_Generated: <date> | Services: <list>_

## Controller

- <ClassName> — <path>

## Handler

- <ClassName> — <path>

## Factory

- <ClassName> — <path>

## Client / Gateway

- <ClassName> — <path>

## NOT DETECTED

- <categories with no results>
```

### File 2: `service_structure.md`

```markdown
# Service Structure

_Generated: <date>_

## <service-name>

### Detected Packages

- <package>: <description>

### Not Found

- <standard packages not found>

### Naming Convention

- <derived from actual file names>
```

### File 3: `integration_map.md`

```markdown
# Integration Map

_Generated: <date>_

## <Integration Name>

- Client: <ClassName> — <path>
- Protocol: <detected>
- Request: <ClassName> — <path>
- Response: <ClassName> — <path>

## NOT DETECTED

- <types not found>
```

### File 4: `dto_pattern.md`

```markdown
# DTO Pattern

_Generated: <date>_

## Request DTO

- <ClassName> extends <Base> — <path> — annotations: <list>

## Response DTO

- <ClassName> — <path>

## NOT DETECTED

- <types not found>
```

### File 5: `error_pattern.md`

```markdown
# Error Handling Pattern

_Generated: <date>_

## Exception Classes

- <ClassName> — <path>

## Error Code Format

- Pattern: <format> — Example: <real constant>

## NOT DETECTED

- <patterns not found>
```

### Step 9 VALIDATION

- [ ] ALL 5 files created
- [ ] Every class has file path
- [ ] No hardcoded/example values
- [ ] `NOT DETECTED` for empty sections
- [ ] No `{placeholder}` text
