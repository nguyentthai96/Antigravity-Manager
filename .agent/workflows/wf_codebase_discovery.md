---
description: Phân tích source code cũ bằng SocratiCode + GitNexus + brainstorming → sinh knowledge files cho downstream workflows
---

**[WORKFLOW]** Standalone — chạy theo nhu cầu, không nằm trong pipeline chính.

- **Purpose**: Index, phân tích, và tổng hợp kiến thức từ source code → sinh knowledge files
- **Scope**: Multi-project/service — hỗ trợ chạy trên bất kỳ service nào
- **Output**: `base_knowledge/` folder chứa các knowledge files + index chung

**Dependencies**: SocratiCode MCP, GitNexus MCP, Brainstorming skill

> ℹ️ Workflow này **không block** pipeline chính (`wf_pre_openspec` → `wf_openspec` → ...).
> Nếu knowledge files chưa có, các workflow khác vẫn chạy được (fallback rules trong `scan_rules.md`).

---

## Input

| Param | Required | Description |
|-------|----------|-------------|
| `PROJECT_PATH` | Có | Absolute path đến project cần phân tích (VD: `/home/user/project/api-service`) |
| `SERVICE_NAME` | Không | Tên service (default: extract từ folder name) |
| `FORCE_REINDEX` | Không | `true` để force re-index (default: `false` — skip nếu đã indexed) |

---

## Step 1 — Check & Index Codebase (SocratiCode)

### 1a. Check existing index
```
codebase_status(projectPath="{PROJECT_PATH}")
```
- Nếu đã indexed + `FORCE_REINDEX=false` → skip to Step 1b
- Nếu chưa indexed hoặc `FORCE_REINDEX=true`:

### 1b. Index codebase
```
codebase_index(projectPath="{PROJECT_PATH}")
```
- Poll `codebase_status` until 100% complete

### 1c. Build dependency graph
```
codebase_graph_build(projectPath="{PROJECT_PATH}")
```
- Poll `codebase_graph_status` until complete

### 1d. Graph stats overview
```
codebase_graph_stats(projectPath="{PROJECT_PATH}")
```
- Ghi nhận: total files, edges, most connected files, orphans

**VALIDATION:**
- [ ] Index status = 100%
- [ ] Graph built successfully
- [ ] Stats captured

---

## Step 2 — Index Code Knowledge Graph (GitNexus)

### 2a. Check GitNexus repos
```
gitnexus list_repos()
```
- Nếu repo đã indexed → skip re-index
- Nếu chưa → cần user chạy `gitnexus index` trước (ngoài scope workflow)

### 2b. Explore communities & processes
```
gitnexus cypher("MATCH (c:Community) RETURN c.heuristicLabel, c.symbolCount, c.keywords ORDER BY c.symbolCount DESC LIMIT 20")
```
```
gitnexus cypher("MATCH (p:Process) RETURN p.heuristicLabel, p.processType, p.stepCount ORDER BY p.stepCount DESC LIMIT 20")
```

### 2c. Map routes (nếu có API)
```
gitnexus route_map()
```

**VALIDATION:**
- [ ] Communities listed
- [ ] Processes listed
- [ ] Routes mapped (hoặc N/A nếu không phải API service)

---

## Step 3 — Extract Features from Endpoints, Handlers & Packages

> **Mục tiêu**: Dựa trên API endpoints, handler classes, và package structure để tạo danh sách tính năng business.

### 3a. Extract ALL API Endpoints

**Cách 1 — GitNexus route_map** (ưu tiên nếu có kết quả):
```
gitnexus route_map()
```
→ Thu thập: route path, HTTP method, handler file, middleware/wrapper

**Cách 2 — Grep endpoints** (fallback hoặc bổ sung):
```
grep_search("@GetMapping|@PostMapping|@PutMapping|@DeleteMapping|@RequestMapping", 
  SearchPath="{PROJECT_PATH}/src", IsRegex=true, MatchPerLine=true, Includes=["*.java"])
```

**Cách 3 — SocratiCode search** (semantic):
```
codebase_search(query="REST API endpoint controller mapping", projectPath="{PROJECT_PATH}", limit=20)
```

**Output 3a**: Bảng endpoint master list:

| # | HTTP Method | Path | Handler Class | Handler Method | Package |
|---|-------------|------|---------------|----------------|---------|
| 1 | POST | /api/v1/accounts/register | AccountController | registerAccount | presentation.controller |
| ... | | | | | |

### 3b. Extract Handler/UseCase/Service Chain

Với **mỗi handler/controller** phát hiện từ 3a:
```
gitnexus context("{ControllerName}", include_content=false)
```
→ Thu thập: callees (UseCase → Repository → Entity chain)

**Fallback** nếu GitNexus không có kết quả:
```
codebase_symbol(name="{ControllerName}", projectPath="{PROJECT_PATH}")
codebase_flow(entrypoint="{controllerMethodName}", projectPath="{PROJECT_PATH}", depth=4)
```

**Output 3b**: Handler-to-chain mapping:

```
Controller.method() → UseCase.execute() → Repository.method() → JpaEntity
```

### 3c. Extract Package/Module Grouping

```
codebase_symbols(projectPath="{PROJECT_PATH}", query="UseCase")
codebase_symbols(projectPath="{PROJECT_PATH}", query="Controller")
codebase_symbols(projectPath="{PROJECT_PATH}", query="Handler")
codebase_symbols(projectPath="{PROJECT_PATH}", query="Service")
codebase_symbols(projectPath="{PROJECT_PATH}", query="Factory")
```

**Bổ sung — Folder-level grouping:**
```
# Liệt kê tất cả sub-packages dưới root source
list_dir("{PROJECT_PATH}/src/main/java/{BASE_PACKAGE_PATH}")
```

→ Nhóm symbols theo `package path` → Mỗi package = 1 potential functional group

### 3d. Consolidate — Feature List

**Merge logic** (ưu tiên theo thứ tự):
1. **Endpoint → Handler → UseCase chain** (Step 3a + 3b) — 1 endpoint = 1 feature candidate
2. **Package grouping** (Step 3c) — Nhiều endpoints cùng package = 1 feature group
3. **Community labels** (Step 2b) — Validate/enrich keywords

**Template cho mỗi feature:**
```markdown
### <Feature Name> [<Feature-ID>]
- **Module**: <package/module path>
- **Endpoint(s)**: <HTTP Method> <Path> (× số endpoint)
- **Handler(s)**: <controller/handler class.method>
- **UseCase(s)**: <use case class>
- **Service(s)**: <service/factory/repository involved>
- **Entity(s)**: <domain model/JPA entity>
- **Type**: <Financial | Non-Financial | Command | Query>
- **Auth**: <JWT-Required | Public | Partner-Signature>
- **MID(s)**: <MID numbers — nếu project dùng MID pattern>
- **Keywords**: <business domain keywords>
```

**VALIDATION:**
- [ ] ≥1 feature detected
- [ ] Mỗi feature có ≥1 endpoint
- [ ] Không duplicate features (merge nếu trùng)
- [ ] Mỗi feature gắn đúng package/module

---

## Step 4 — Classify Code Patterns & Styles

> **Mục tiêu**: Phân nhóm các features/handlers theo pattern code giống nhau → giúp khi tạo feature mới biết follow pattern nào.

### 4a. Phân loại theo Auth Pattern

```
grep_search("@PreAuthorize|@Secured|@RolesAllowed|permitAll|authenticated",
  SearchPath="{PROJECT_PATH}/src", IsRegex=true, MatchPerLine=true, Includes=["*.java"])
```
```
codebase_search(query="security configuration permit all authenticated", projectPath="{PROJECT_PATH}", limit=10)
```

**Phân nhóm:**
| Auth Pattern | Description | Example Endpoints |
|-------------|-------------|-------------------|
| `JWT-Required` | Yêu cầu đăng nhập, có JWT token | ... |
| `Public` | Không cần auth | ... |
| `Partner-Signature` | Verify ECDSA/RSA signature từ partner | ... |
| `Admin-Only` | Cần role admin | ... |

### 4b. Phân loại theo Transaction Flow Style

Dựa trên kết quả Step 3b (handler chain), phân loại:

| Flow Style | Pattern | Description |
|-----------|---------|-------------|
| `Init-Confirm` | Init → (Auth) → Confirm | 2-step transaction có xác thực |
| `Init-AuthMethod-Confirm` | Init → SelectAuth → (OTP/Biometric) → Confirm | Financial transaction |
| `Single-Step-Command` | Controller → UseCase → save | 1-step write (settings, config) |
| `Single-Step-Query` | Controller → UseCase → find/get | 1-step read-only |
| `CRUD` | Standard Create/Read/Update/Delete | Basic entity management |
| `Report/Export` | Query → Aggregate → Format (PDF/Excel/CSV) | Data export |
| `Partner-API` | Receive → Verify Signature → Process → Response | External partner integration |
| `Async/Event` | Receive → Queue → Background Process | Event-driven |

**Cách detect pattern:**
```
gitnexus query("init confirm handler transaction", limit=10)
gitnexus query("report export download file", limit=5)
gitnexus query("partner gateway external api", limit=5)
codebase_search(query="event handler async listener", projectPath="{PROJECT_PATH}", limit=5)
```

### 4c. Phân loại theo Common Utilities/Base Classes

```
gitnexus cypher("MATCH (c:Class)-[:CodeRelation {type: 'EXTENDS'}]->(p) RETURN c.name, p.name, c.filePath ORDER BY p.name LIMIT 50")
```
```
gitnexus cypher("MATCH (c:Class)-[:CodeRelation {type: 'IMPLEMENTS'}]->(i:Interface) RETURN c.name, i.name, c.filePath ORDER BY i.name LIMIT 50")
```

**Output 4c — Base Class Registry:**
```markdown
| Base Class/Interface | Extending/Implementing Classes | Purpose |
|---------------------|-------------------------------|---------|
| BaseHandler | Handler1, Handler2, ... | Common handler logic |
| SignableRequest | Request1, Request2, ... | ECDSA signature verification |
```

### 4d. Consolidate — Code Pattern Matrix

Tổng hợp 4a + 4b + 4c thành **Pattern Matrix**:

```markdown
| Feature | Auth Pattern | Flow Style | Base Class | Key Utilities |
|---------|-------------|------------|------------|---------------|
| RegisterAccount | Partner-Signature | Single-Step-Command | - | DataIntegrityChecker, IdempotencyCheck |
| UpdateStatus | Partner-Signature | Single-Step-Command | - | DataIntegrityChecker, IdempotencyCheck |
| ... | ... | ... | ... | ... |
```

**VALIDATION:**
- [ ] ≥1 auth pattern identified
- [ ] ≥1 flow style identified
- [ ] Base classes mapped
- [ ] Pattern matrix complete (mỗi feature có đủ 4 cột)

---

## Step 5 — Analyze Architecture

### 5a. SocratiCode — Graph statistics
```
codebase_graph_stats(projectPath="{PROJECT_PATH}")
```

### 5b. SocratiCode — Search patterns
```
codebase_search(query="base class handler controller", projectPath="{PROJECT_PATH}", limit=15)
codebase_search(query="interface service factory", projectPath="{PROJECT_PATH}", limit=15)
codebase_search(query="configuration bean setup", projectPath="{PROJECT_PATH}", limit=10)
```

### 5c. Consolidate — Architecture Map
Tổng hợp thành architecture knowledge:
- Base classes + inheritance hierarchy
- Naming conventions (prefix/suffix patterns)
- Package/layer structure
- Key design patterns (Interface+Impl, Factory, Strategy, etc.)
- Configuration patterns (application.yml, @ConfigurationProperties)
- Cross-cutting concerns (logging, security, caching, error handling)

**VALIDATION:**
- [ ] Base classes identified
- [ ] Package structure mapped
- [ ] Naming conventions documented

---

## Step 6 — Analyze Transaction Flows (Deep)

### 6a. GitNexus — Query transaction patterns
```
gitnexus query("transaction flow init confirm", limit=10)
gitnexus query("financial transfer payment", limit=10)
gitnexus query("OTP authentication verify", limit=10)
```

### 6b. GitNexus — Trace handler flows
Với mỗi handler phát hiện:
```
gitnexus context("{handler_name}", include_content=true)
```
→ Xác định flow pattern: Init→Confirm, Init→AuthMethod→Confirm, Single-step

### 6c. SocratiCode — Deep search specific patterns
```
codebase_search(query="auth method OTP biometric", projectPath="{PROJECT_PATH}", limit=10)
codebase_search(query="error code response exception", projectPath="{PROJECT_PATH}", limit=10)
```

### 6d. Consolidate — Transaction Flow Knowledge
Phân loại flows theo types (từ Step 4b):

Với mỗi type, liệt kê:
- Example handlers
- Common base class
- Auth pattern
- Error handling pattern
- Sequence diagram (text-based)

**VALIDATION:**
- [ ] ≥1 flow type identified
- [ ] Mỗi flow type có ≥1 example handler
- [ ] Pattern documentation complete

---

## Step 7 — Brainstorm & Validate

### 7a. Invoke brainstorming skill
Read `.agent/skills/brainstorming/SKILL.md` (nếu tồn tại) hoặc thực hiện review:

**Review Checklist:**
- [ ] Danh sách features có đầy đủ không? Có miss feature nào rõ ràng?
- [ ] Architecture map có chính xác không? Có base class nào chưa phát hiện?
- [ ] Transaction flow classification có đúng pattern không?
- [ ] Naming conventions có nhất quán không?
- [ ] Có integration/external system nào chưa được map?
- [ ] Code pattern matrix đã cover hết chưa?

### 7b. Cross-validate with source code
Với mỗi finding cần verify:
```
grep_search("{pattern}", SearchPath="{PROJECT_PATH}", Includes=["{ext}"])
```

### 7c. Agent tự bổ sung insights
Agent PHẢI tự tổng hợp thêm:
- **Missing features**: Các feature có thể tồn tại nhưng chưa phát hiện qua tool
- **Undocumented patterns**: Code style/convention chưa có trong danh sách
- **Technical debt observations**: Inconsistencies, anti-patterns phát hiện
- **Suggestions for improvement**: Đề xuất cải thiện cấu trúc code

### 7d. Ask user (interactive)
Nếu có unknowns sau brainstorm:
- Liệt kê danh sách questions
- Đợi user trả lời
- Update findings theo user feedback

**VALIDATION:**
- [ ] Brainstorm review completed
- [ ] Agent insights added
- [ ] Unknowns resolved (hoặc marked as assumption)
- [ ] User feedback incorporated

---

## Step 8 — Generate Knowledge Files

### 8a. Create knowledge directory
```bash
mkdir -p base_knowledge/structures/propose
mkdir -p base_knowledge/structures/overview
```

### 8b. Generate `features.md` — Master Feature List
File: `base_knowledge/structures/propose/features.md`

Nội dung: Danh sách TẤT CẢ features phát hiện, phân nhóm theo functional area:

```markdown
# Feature Index — <service-name>

> Auto-generated by wf_codebase_discovery
> Date: <date>
> Project: <project-path>

## Summary
- Total Features: <count>
- Total Endpoints: <count>
- Total Handlers/UseCases: <count>
- Functional Groups: <count>

---

## Group: <functional-area-1>

### <feature-name-1> [F-001]
- **Keywords**: <keyword1, keyword2>
- **Module**: <package path>
- **Endpoint(s)**: `POST /api/v1/accounts/register`
- **Handler(s)**: AccountController.registerAccount()
- **UseCase(s)**: RegisterCustomerUseCase
- **Service(s)**: <service class list>
- **Entity(s)**: Customer, CustomerJpaEntity
- **Type**: Command
- **Auth**: Partner-Signature
- **Flow Style**: Single-Step-Command
- **MID(s)**: N/A
- **Base Class**: N/A
- **Utilities**: DataIntegrityChecker, IdempotencyCheck

### <feature-name-2> [F-002]
...

---

## Group: <functional-area-2>
...
```

### 8c. Generate `knowledge_code_patterns.md` — Pattern Classification
File: `base_knowledge/structures/propose/knowledge_code_patterns.md`

```markdown
# Code Pattern Classification — <service-name>

> Auto-generated by wf_codebase_discovery
> Date: <date>

## Auth Patterns
| Pattern | Description | Endpoints Using | Count |
|---------|-------------|-----------------|-------|
| Partner-Signature | ECDSA verify via DataIntegrityChecker | /api/v1/accounts/* | 3 |
| JWT-Required | Spring Security JWT filter | ... | ... |
| Public | No auth required | ... | ... |

## Flow Styles
| Style | Pattern | Example Features | Count |
|-------|---------|-----------------|-------|
| Single-Step-Command | Controller → UseCase → save | RegisterAccount, UpdateStatus | 3 |
| Init-Confirm | Init → (Auth) → Confirm | ... | ... |
| Report/Export | Query → Aggregate → Format | ... | ... |

## Base Class Registry
| Base Class/Interface | Extending Classes | Purpose | Package |
|---------------------|-------------------|---------|---------|
| SignableRequest | RegisterAccountRequest, ... | ECDSA signature data | presentation.dto |

## Common Utilities
| Utility | Usage Count | Used By Features | Description |
|---------|-------------|-----------------|-------------|
| DataIntegrityChecker | 3 | RegisterAccount, UpdateStatus, ChangePackage | ECDSA signature verification |
| IdempotencyCheck | 3 | RegisterAccount, UpdateStatus, ChangePackage | requestId dedup |

## Pattern Matrix (Feature × Pattern)
| Feature | Auth | Flow Style | Base Class | Key Utilities |
|---------|------|------------|------------|---------------|
| RegisterAccount | Partner-Signature | Single-Step-Command | - | DataIntegrityChecker, Idempotency |
| UpdateStatus | Partner-Signature | Single-Step-Command | - | DataIntegrityChecker, Idempotency |
| ChangePackage | Partner-Signature | Single-Step-Command | - | DataIntegrityChecker, Idempotency |
```

### 8d. Generate `knowledge_transaction_flow.md`
File: `base_knowledge/structures/propose/knowledge_transaction_flow.md`

Nội dung: Transaction flow patterns, phân loại theo types với examples thực tế.
Bao gồm:
- Sequence diagram text cho mỗi flow type
- Example handlers/controllers
- Error handling pattern
- Auth chain

### 8e. Generate `knowledge_architecture.md`
File: `base_knowledge/structures/propose/knowledge_architecture.md`

Nội dung: Architecture map — base classes, layers, conventions, patterns.

### 8f. Generate `overview_system.md`
File: `base_knowledge/structures/overview/overview_system.md`

Nội dung: Tech stack, system overview, dependencies, external integrations.

### 8g. Generate `knowledge_index.md` (Master Index)
File: `base_knowledge/knowledge_index.md`

```markdown
# Knowledge Index — <service-name>

> Master index of all knowledge files generated by wf_codebase_discovery.
> Last updated: <date>
> Project: <project-path>

## Quick Stats
- Total Features: <count>
- Total Endpoints: <count>
- Functional Groups: <count>
- Flow Styles: <count>
- Auth Patterns: <count>

## Knowledge Files

| File | Category | Description | Status |
|------|----------|-------------|--------|
| `structures/propose/features.md` | Features | Master feature list với keywords, endpoints, handlers | ✅ Generated |
| `structures/propose/knowledge_code_patterns.md` | Patterns | Auth patterns, flow styles, base classes, utilities | ✅ Generated |
| `structures/propose/knowledge_transaction_flow.md` | Flows | Transaction flow patterns (Financial/Non-Financial/Command/Query) | ✅ Generated |
| `structures/propose/knowledge_architecture.md` | Architecture | Base classes, layers, conventions | ✅ Generated |
| `structures/overview/overview_system.md` | Overview | Tech stack, integrations | ✅ Generated |

## Cross-References

### Feature → Pattern Lookup
> Khi cần biết feature X dùng pattern gì:
> 1. Tìm feature trong `features.md` → lấy `Flow Style` + `Auth`
> 2. Xem chi tiết pattern trong `knowledge_code_patterns.md`
> 3. Xem flow diagram trong `knowledge_transaction_flow.md`

### Pattern → Feature Lookup
> Khi cần tạo feature mới theo pattern Y:
> 1. Tìm pattern trong `knowledge_code_patterns.md` → lấy example features
> 2. Xem chi tiết feature trong `features.md` → lấy handler/usecase chain
> 3. Follow code convention trong `knowledge_architecture.md`

## Usage Notes
- Các downstream workflow (`wf_pre_openspec`, `wf_openspec`) nên đọc `knowledge_index.md` trước
- `features.md` là file tham chiếu chính để detect scope khi tạo feature mới
- `knowledge_code_patterns.md` giúp xác định pattern nào nên follow khi implement
```

**VALIDATION:**
- [ ] Tất cả files tạo thành công
- [ ] `knowledge_index.md` liệt kê đủ files
- [ ] Mỗi file có nội dung thực tế (không phải placeholder)
- [ ] `features.md` có ≥1 feature với đầy đủ fields
- [ ] `knowledge_code_patterns.md` có ≥1 auth pattern + ≥1 flow style
- [ ] `knowledge_transaction_flow.md` có ≥1 flow type
- [ ] Cross-references trong index chính xác

---

## Step 9 — Generate Standards Documents

> **Mục tiêu**: Trích xuất conventions và standards đang tồn tại trong source code → sinh ra 4 core documents mà `openspec-apply-change` (Step 4.5) yêu cầu bắt buộc.
> **Tham khảo template**: `.agent/skills/wf-codebase-discovery/SKILL.md` → section "Standards Generation Rules"

### 9a. Generate `coding_standard.md`
File: `base_knowledge/standards/coding_standard.md`

**Cách trích xuất:**
1. **Naming conventions** — Dùng SocratiCode `codebase_symbols(query="Handler")` + `codebase_symbols(query="Service")` → phân tích naming patterns
2. **Package structure** — Dùng `codebase_graph_stats()` → top-level package organization
3. **Code patterns** — Từ `knowledge_code_patterns.md` (Step 8) → tóm tắt Init/Confirm flow, Factory pattern
4. **DTO rules** — Dùng `codebase_search("Request Response DTO")` → extract DTO naming/structure
5. **Annotation usage** — Dùng `grep_search` hoặc GitNexus cypher tìm custom annotations

Nội dung bao gồm:
- Naming conventions (class, method, field, constant)
- Package structure conventions
- DTO/Request/Response patterns
- Code flow patterns (Init-Confirm, CRUD, Query)
- Annotation usage rules
- Agent tự tổng hợp thêm thông tin bổ sung từ codebase nếu phát hiện conventions khác

### 9b. Generate `logging_standard.md`
File: `base_knowledge/standards/logging_standard.md`

**Cách trích xuất:**
1. **Log framework** — Dùng `codebase_search("logger LoggerFactory")` → framework config
2. **MDC fields** — Dùng `grep_search("MDC.put")` → liệt kê MDC fields đang dùng
3. **Log levels** — Dùng `grep_search("log.debug\\|log.info\\|log.warn\\|log.error")` → phân tích usage patterns
4. **Data masking** — Dùng `codebase_search("mask sensitive data log")` → masking patterns
5. **Log format** — Check `logback.xml` hoặc `application.yml` → structured format

Nội dung bao gồm:
- Log framework và configuration
- MDC context fields (list + purpose)
- Log level guidelines (khi nào dùng DEBUG/INFO/WARN/ERROR)
- Sensitive data masking rules
- Log message format conventions
- Agent tự tổng hợp thêm thông tin bổ sung từ codebase nếu phát hiện patterns khác

### 9c. Generate `error_handling_standard.md`
File: `base_knowledge/standards/error_handling_standard.md`

**Cách trích xuất:**
1. **Exception hierarchy** — Dùng GitNexus `cypher("MATCH (c:Class)-[:CodeRelation {type: 'EXTENDS'}]->(p) WHERE p.name CONTAINS 'Exception' RETURN c.name, p.name, c.filePath")` → exception tree
2. **Error codes** — Dùng `grep_search("ErrorCode\\|error_code\\|errorCode")` → error code patterns
3. **Error response** — Dùng `codebase_search("ControllerAdvice ExceptionHandler")` → global handler
4. **HTTP status mapping** — Dùng `codebase_search("HttpStatus ResponseEntity")` → status mapping

Nội dung bao gồm:
- Exception class hierarchy (diagram)
- Error code format và registry
- Error response structure
- @ControllerAdvice / @ExceptionHandler patterns
- HTTP status code mapping rules
- Retry/fallback patterns (nếu có)
- Agent tự tổng hợp thêm thông tin bổ sung từ codebase nếu phát hiện patterns khác

### 9d. Generate `security_standard.md`
File: `base_knowledge/standards/security_standard.md`

**Cách trích xuất:**
1. **Authentication** — Dùng `codebase_search("JWT token authentication filter")` → auth mechanism
2. **Authorization** — Dùng `codebase_search("@PreAuthorize @Secured Role")` → authz patterns
3. **Encryption** — Dùng `grep_search("ECDSA\\|AES\\|RSA\\|Cipher\\|encrypt\\|decrypt")` → crypto algorithms
4. **Input validation** — Dùng `codebase_search("@Valid @NotNull validation")` → validation chain
5. **Audit** — Dùng `codebase_search("audit log security event")` → audit patterns

Nội dung bao gồm:
- Authentication mechanism (JWT/Session/OAuth2)
- Authorization patterns (roles, permissions)
- Encryption algorithms đang dùng
- Input validation chain
- Security audit logging
- Agent tự tổng hợp thêm thông tin bổ sung từ codebase nếu phát hiện patterns khác

### 9e. Update `knowledge_index.md` — thêm Standards section

Bổ sung vào `knowledge_index.md` (đã tạo ở Step 8g):

```markdown
### Standards Documents (standards/)

| # | File | Description | Last Updated |
|---|------|-------------|-------------|
| S1 | [coding_standard.md](standards/coding_standard.md) | Naming, structure, patterns, annotations | <date> |
| S2 | [logging_standard.md](standards/logging_standard.md) | MDC, log levels, format, masking | <date> |
| S3 | [error_handling_standard.md](standards/error_handling_standard.md) | Exceptions, error codes, response format | <date> |
| S4 | [security_standard.md](standards/security_standard.md) | Auth, encryption, validation, audit | <date> |
```

**VALIDATION:**
- [ ] `base_knowledge/standards/` directory tồn tại
- [ ] 4 files tạo thành công (coding, logging, error_handling, security)
- [ ] Mỗi file có nội dung thực tế trích xuất từ source code
- [ ] `knowledge_index.md` đã cập nhật Standards section
- [ ] Nội dung phản ánh đúng codebase hiện tại (không copy generic template)

---

## Output Summary


```
═══════════════════════════════════════
CODEBASE DISCOVERY COMPLETE
═══════════════════════════════════════

Project:     <project-path>
Service:     <service-name>

Indexing:
  SocratiCode: <status> (<chunk-count> chunks)
  GitNexus:    <status> (<community-count> communities, <process-count> processes)
  Graph:       <node-count> nodes, <edge-count> edges

Discovery:
  Features:      <count> (in <group-count> groups)
  Endpoints:     <count>
  Handlers:      <count>
  UseCases:      <count>
  Services:      <count>
  Flow Styles:   <count> (<list>)
  Auth Patterns: <count> (<list>)
  Base Classes:  <count>
  Utilities:     <count>

Generated Files:
  ✅ base_knowledge/knowledge_index.md (Master Index)
  ✅ base_knowledge/structures/propose/features.md
  ✅ base_knowledge/structures/propose/knowledge_code_patterns.md
  ✅ base_knowledge/structures/propose/knowledge_transaction_flow.md
  ✅ base_knowledge/structures/propose/knowledge_architecture.md
  ✅ base_knowledge/structures/overview/overview_system.md
  ✅ base_knowledge/standards/coding_standard.md
  ✅ base_knowledge/standards/logging_standard.md
  ✅ base_knowledge/standards/error_handling_standard.md
  ✅ base_knowledge/standards/security_standard.md

═══════════════════════════════════════
```
