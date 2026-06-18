## Implementation Tasks

<!-- 
Tasks MUST be ordered by dependency layer:
1. Entity + Repository (data layer first)
2. Model (domain model)
3. Factory interface + implementation
4. Request/Response DTOs
5. Handler (business logic)
6. Controller (API surface)
7. Configuration (cache, constants, tests)

Each task format:
- [ ] **Task title**
  - File: `path/to/File.java`
  - Extends: `BaseClass<generics>`
  - Package: `vn.thaint.application.{service}.module.{feature}.{layer}`
  - Notes: specific implementation details
-->

### Layer 1: Entity + Repository

- [ ] **Create {Feature}Entity**
  - File: `{service}/module/{feature}/entity/{Feature}Entity.java`
  - Implements: `IBaseEntity<Long>`
  - Annotations: `@Entity`, `@Table`, `@SuperBuilder(toBuilder=true)`, `@NoArgsConstructor(PROTECTED)`
  - Table: `TABLE_NAME`

- [ ] **Create {Feature}Repository**
  - File: `{service}/module/{feature}/repository/{Feature}Repository.java`
  - Extends: `ListCrudRepository<{Feature}Entity, Long>`, `ListPagingAndSortingRepository<{Feature}Entity, Long>`

### Layer 2: Model

- [ ] **Create {Feature}Model**
  - File: `{service}/module/{feature}/model/{Feature}Model.java`
  - Annotations: `@Getter`, `@SuperBuilder(toBuilder=true)`, `@NoArgsConstructor(PROTECTED)`

### Layer 3: Factory

- [ ] **Create I{Feature}Factory interface**
  - File: `{service}/module/{feature}/factory/I{Feature}Factory.java`

- [ ] **Create {Feature}Factory implementation**
  - File: `{service}/module/{feature}/factory/impl/{Feature}Factory.java`
  - Extends: `BaseCrudDataFactory<Long, {Feature}Model, Long, {Feature}Entity, {Feature}Repository>`
  - Override: `convertToModel`, `createConvertToEntity`, `updateConvertToEntity`, `getCacheClass`
  - Cache: cacheModel=true/false, cacheCollection=true/false

### Layer 4: Request/Response DTOs

- [ ] **Create {Feature}Request**
  - File: `{service}/module/{feature}/model/request/{Feature}Request.java`
  - Extends: `BaseSessionRequest implements QueryMessage`

- [ ] **Create {Feature}Response**
  - File: `{service}/module/{feature}/model/response/{Feature}Response.java`

### Layer 5: Handler

- [ ] **Create {Feature}Handler**
  - File: `{service}/module/{feature}/handler/{Feature}Handler.java`
  - Extends: `BaseQueryHandler<{Feature}Request, {Feature}Response>`
  - Inject: `I{Feature}Factory` via `@RequiredArgsConstructor`
  - Override: `aroundHandle(request)`
  - Logging: `LogContext.push(LogType.TRACING, ...)` at each checkpoint

### Layer 6: Controller

- [ ] **Create App{Feature}Controller**
  - File: `{service}/module/{feature}/controller/app/App{Feature}Controller.java`
  - Extends: `BaseAppController`
  - Annotation: `@LoggingInsert`, `@RestController`
  - Pattern: thin controller, delegate to `execute(request)`

### Layer 7: Configuration

- [ ] **Register constants (if applicable)**
- [ ] **Add cache reload constant (if applicable)**
- [ ] **Update module configuration**
