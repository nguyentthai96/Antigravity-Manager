# Impact Analysis Template

> Dùng bởi `wf_openspec.md` Step 6.5 — Output 1
> Tối ưu cho AI coding agent: đọc xong → mở file → code ngay.

Write to: `openspec/changes/<name>/impact_analysis.md`

---

## Template

```markdown
# Impact Analysis: <feature-name>

_Generated: <date>_

---

## 1. Core Files — NƠI SỬA

> Chỉ liệt kê files CẦN MODIFY code. BẮT BUỘC `file:///` link + line range.

| # | File | Line Range | Chức năng |
|---|------|-----------|-----------|
| 1 | [{ClassName}.{ext}](file:///absolute/path/to/{ClassName}.{ext}) | L{start}-{end} | {mô tả ngắn} |

---

## 2. Call Tree — LOGIC CẦN SỬA

> BẮT BUỘC ASCII tree. KHÔNG dùng Mermaid (AI parse text nhanh hơn).
> Ghi annotation `// ←` ở điểm quan trọng. Ghi exact params cho utility calls.

#### `{ClassName}.{methodName}()` (L{start}-{end})

⟶ {entryMethod}(request)
├── {condition}? → {methodA}()
│   └── {Util}.{method}({param1}, {param2})  // ← {annotation}
├── !{condition}? → {methodB}()
│   └── {Service}.{action}()
└── {onFailed}()
    ├── {CASE_A} → {handler1}()
    ├── {CASE_B} → {handler2}()
    └── default → {cleanup} + throw {ErrorType}

---

## 3. Blast Radius

> BẮT BUỘC 4 layers. BẮT BUỘC `file:///` links cho MỌI file.
> BẮT BUỘC trace gRPC đến client stub. BẮT BUỘC liệt kê REST endpoints.

### 🔴 Direct Impact — {service-name} ({N} files)

| # | File | Link | Cách sử dụng |
|---|------|------|-------------|
| 1 | `{ClassName}.{ext}` | [{ClassName}](file:///path) | {mô tả: Inject X, gọi .handle() khi Y} |

### 🟡 Indirect Impact — {service-name} ({N} files)

| # | File | Link | Cách sử dụng |
|---|------|------|-------------|

### 🟠 Cross-service Impact ({N} files)

| # | File | Link | Protocol | Cách sử dụng |
|---|------|------|----------|-------------|
| 1 | `{Client}.{ext}` | [{Client}](file:///path) | gRPC / REST / Kafka | {mô tả} |

### 🟢 Shared Utilities ({N} files)

| # | File | Link | Methods dùng |
|---|------|------|-------------|

---

## 4. Reuse Map

> Follows `reuse_rules_compact.md` Step 2 Decision Framework.

| Logic Block | Existing Location | Match % | Decision | Impact | Action |
|---|---|---|---|---|---|
| {logic1} | [{Class}:{Line}](file:///path#L{line}) | {N}% | EXTRACT / REUSE / NEW | {🟢/🟡/🔴} ({N} callers) | → {target} |

### EXTRACT Details (nếu có)

#### E1: {logic block name}

**Source:** [{SourceClass.method}](file:///path#L{start}-L{end}) — {mô tả logic}
**Target:** `I{ServiceName}.{method}` — `{module}/common/`
**Callers found:** {N} files
- [{file1}:{method1}](file:///path) → {sẽ update / không ảnh hưởng}

**Impact level:** {🟢 Low / 🟡 Medium / 🔴 High}
**Breaking changes:** {có/không}
**Migration plan:** {backward-compatible / direct update / need wrapper}

---

## 5. Context Snapshot — ĐỦ ĐỂ CODE

> Agent đọc section này → đủ info bắt tay code, KHÔNG cần search thêm.

### Dependencies

| Dependency | Type | Key Methods | Ghi chú |
|-----------|------|-------------|---------|
| `{IService}` | Interface (injected) | `{method1}()`, `{method2}()` | {context} |
| `{Util}` | Static utility | `{method}({params})` | {algorithm/lib} |
| `{Client}` | gRPC/REST client | `{method}({RequestType})` | {target service} |

### Config Keys

| Key | Source | Ví dụ value | Nơi dùng |
|-----|--------|------------|---------|
| `{CONFIG_KEY}` | `IConfigFactory` | `{value}` | `{method()}` |

### Error Codes Thrown

| Error Code | Condition | Nơi throw |
|-----------|-----------|-----------|
| `{ErrorType.CODE}` | {condition} | [{method}():L{line}](file:///path#L{line}) |

### DTO Reuse Check

| DTO cần | Existing DTO | Match % | Decision |
|---|---|---|---|
| {context fields} | {ExistingRequest} | {N}% | REUSE / NEW |

### Base API Verification

| API Call | Verified Method | Source | Status |
|---|---|---|---|
| {intended call} | {actual method signature} | {tool used} | ✅ / ❌ |
```

---

## Validation Checklist (trước khi save)

- [ ] Mọi file đều có `file:///` link (không path rút gọn)
- [ ] Core files có line range (`L{start}-{end}`)
- [ ] Call tree có branching conditions + annotations
- [ ] Blast radius có đủ 4 layers
- [ ] Cross-service trace đến client stub
- [ ] Context Snapshot có: dependencies, config keys, error codes
- [ ] Reuse Map entries có link đến source code

---

# Tasks.md Refactoring Layer Template

> Dùng bởi `wf_openspec.md` Step 6.5 — Output 2
> ⚠️ tasks.md chỉ chứa **actionable tasks** — KHÔNG chứa analysis report.

```markdown
### Layer N: Refactoring — {mô tả} `[EXTRACT]`

> **Ref:** `impact_analysis.md` — {🟢/🟡/🔴} impact, {N} callers

- [ ] **R01: Create I{ServiceName}** `[NEW][EXTRACT]`
  - File: `{module}/common/I{ServiceName}.{PROJECT_EXT}`
  - Methods: {extracted methods list}
  - Source: Extracted from `{SourceClass.method1}`, `{SourceClass.method2}`

- [ ] **R02: Create {ServiceName} impl** `[NEW][EXTRACT]`
  - File: `{module}/common/impl/{ServiceName}.{PROJECT_EXT}`
  - Implements: `I{ServiceName}`
  - Dependencies: {injected interfaces}

- [ ] **R03: Refactor {OriginalClass}** `[MODIFY][REUSE]`
  - Remove: {methods removed}
  - Add dependency: `I{ServiceName}`
  - Delegate: {method calls}

#### Post-extract Checklist (BẮT BUỘC sau khi hoàn thành EXTRACT)

- [ ] Tất cả callers cũ đã được update → inject interface, delegate
- [ ] Không còn duplicate logic ở source gốc (grep confirm)
- [ ] Build/compile thành công — không breaking change
- [ ] Graph query confirm: file mới có đúng dependents
- [ ] tasks.md đã sync: dependencies, method names, signatures khớp code thực tế
```
