# Reuse & Clone Detection Rules (Compact)

> Merged from: reuse_detection_rules, pre_task_clone_analysis, implementation_guards.
> Applies to: propose + apply phases.
>
> **{PROJECT_EXT} / {PROJECT_LANG}**: See `scan_rules.md` → LANGUAGE DETECTION section for resolution.

---

## Pre-Condition — Change Type Must Be Known

> Classification (MAINTENANCE/EXTEND/NEWBUILD) is assumed to be done in propose phase
> (`/wf_pre_openspec` Step 1d or `/wf_openspec` Step 6c) and saved in `pre_openspec.md`.
> If not yet classified → run `.agent/common-rules/change_type_classification.md` first.

---

## Step 0 — Trigger Detection

**Red Flags (search immediately if ANY present):**

- Task references another class: `"same as X"`, `"follows X"`, `"similar to X"`
- New class for same domain (e.g., `BatchHandler` when `SingleHandler` exists) → **MUST compare line-by-line**
- Task describes common logic: verify, validate, save, update, check duplicate, build model
- Method name collision with existing class in same module → **STOP, search**
- Batch+Single same entity → **MUST EXTRACT per-item logic**

**Auto-trigger:** `[NEW]` handler/service/factory, `[MODIFY]` adding new method, task >30 lines logic.

**Smart Gate — Search Depth:**

```
graph_query("{target file}") → count imports + dependents
  ≥5 dependents → FULL (Phase 1+2+3)
  1-4 dependents → STANDARD (Phase 1 → escalate if needed)
  0 dependents → LIGHTWEIGHT (Phase 1 only)
```

---

## Step 0.5 — Task Logic Decomposition (MANDATORY for [NEW] and [MODIFY])

Decompose task into atomic operations, tag each: `[AUTH]`, `[VALIDATE]`, `[PERSIST]`, `[QUERY]`, `[CACHE]`, `[AUDIT]`,
`[NOTIFY]`, `[CONVERT]`.

Each operation → search query → Step 1.

Skip if task has `[EXTRACT]`/`[REUSE]` instructions already.

---

## Step 1 — Escalation Search (3 phases)

### Phase 1: EXACT MATCH (🟢 always run, parallel)

- `grep_search("{method name}", Includes=["{PROJECT_EXT}"])` — exact text
- GKG `search_definitions(["{name}", "{synonym1}", "{synonym2}"])` — named symbols
- If design mentions class → `grep_search("{class name}", Includes=["{PROJECT_EXT}"])`
- **STOP if ≥1 match with ≥80% logic similarity** → Step 2

### Phase 2: SEMANTIC (🔴 only if Phase 1 insufficient)

- SC `codebase_search("{action} {object} {result}", languageFilter="{PROJECT_LANG}", limit=15)`
- **STOP if match found** → combine with Phase 1 → Step 2

### Phase 3: CONTEXT (🟡 optional, only for convention check)

- SC `codebase_context_search("{pattern question}")` — only when needed

> **FILTER RULE:** `codebase_search` MUST use `languageFilter="{PROJECT_LANG}"`. `grep_search` MUST use
`Includes=["{PROJECT_EXT}"]`.
> See `scan_rules.md` → LANGUAGE DETECTION for `{PROJECT_LANG}` / `{PROJECT_EXT}` resolution.

---

## Step 2 — Decision Framework

### Match-based Decision

| Match %                | Decision    | Action                     |
|------------------------|-------------|----------------------------|
| 100%, in shared class  | **REUSE**   | Import + call directly     |
| ≥80%, in feature class | **EXTRACT** | Move to common → both use  |
| 50-80%, generalizable  | **EXTRACT** | Parameterize differences   |
| <50% or not found      | **NEW**     | Write new, document reason |

> 🔴 **HARD BLOCK:** Match ≥80% → MUST EXTRACT. No subjective overrides. Only exception: user explicitly approves.

### Occurrence-based Decision

| Occurrences (≥50% match) | Decision                                           |
|--------------------------|----------------------------------------------------|
| ≥3 places                | **MUST EXTRACT** — no exception                    |
| 2 places                 | **RECOMMEND EXTRACT** — add to tasks.md for review |
| 1 place                  | Use match-based above                              |

---

## Step 3 — Impact Analysis (for EXTRACT/MODIFY)

1. GKG `get_references("{method}", "{file}")` → exact callers
2. SC `codebase_graph_query("{file}")` → dependents
3. `grep_search("{method name}", Includes=["{PROJECT_EXT}"])` → edge cases

| Impact          | Criteria          | Action                                |
|-----------------|-------------------|---------------------------------------|
| 🟢 Low (1-2)    | Few callers       | Extract safely                        |
| 🟡 Medium (3-5) | Several callers   | Backward-compatible wrapper if needed |
| 🔴 High (>5)    | Many/cross-module | Review, consider interface/abstract   |

---

## Step 4 — Extract Pattern (Architecture-Aware)

> ⚠️ **KHÔNG hardcode đường dẫn.** Mỗi dự án có kiến trúc riêng — PHẢI tra cứu cấu trúc thực tế trước khi tạo file.

### 4a. Discover Project Layout (MANDATORY before creating any file)

**Priority order:**

1. **Knowledge file** — Đọc `base_knowledge/structures/propose/knowledge_architecture.md` (nếu tồn tại):
    - `Directory Convention` → biết package nào chứa loại file nào
    - `Naming Conventions` → biết prefix/suffix bắt buộc (I-prefix, Constants suffix, ...)
    - `Key Patterns` → biết pattern chuẩn (interface+impl, base class, ...)

2. **Peer file scan** — Nếu không có knowledge file, tìm file cùng loại đã tồn tại:
   ```
   grep_search("{logic_type_keyword}", Includes=["{PROJECT_EXT}"])
   # Ví dụ: tìm "extends BaseCommandHandler" để biết handler đặt ở đâu
   # Ví dụ: tìm "static.*util" hoặc "Utils" để biết utils đặt ở đâu
   ```
   → Lấy package path từ file tìm được → đặt file mới cùng vị trí.

3. **Graph query** — Nếu vẫn chưa rõ:
   ```
   codebase_graph_query("{source_file}")
   ```
   → Xem các file liên quan import từ đâu → suy ra convention.

### 4b. Map Logic Type → Project Location

Sau khi discover, áp dụng mapping **theo kiến trúc thực tế của dự án**:

| Logic type        | Cách xác định vị trí                                                          | Pattern                                     |
|-------------------|-------------------------------------------------------------------------------|---------------------------------------------|
| Stateless utility | Tìm package chứa `*Utils.{PROJECT_EXT}` hoặc `*Helper.{PROJECT_EXT}` hiện có  | Static methods                              |
| Service with DI   | Tìm package chứa `I*Service.{PROJECT_EXT}` hoặc interface+impl pair hiện có   | `I{Service}` + `{Service}` (interface+impl) |
| Domain-scoped     | Đặt cùng module/package với feature đang thao tác                             | Domain service cùng feature                 |
| Template method   | Tìm package chứa `Base*.{PROJECT_EXT}` hoặc `Abstract*.{PROJECT_EXT}` hiện có | Abstract class                              |
| Common/shared     | Tìm module `common/` hoặc `shared/` hoặc library dùng chung                   | Tùy convention dự án                        |

### 4c. Validation Checklist (trước khi tạo file)

- [ ] Package path khớp với convention trong `knowledge_architecture.md` hoặc peer files
- [ ] Naming convention đúng (prefix/suffix theo dự án, ví dụ: I-prefix cho interface)
- [ ] Base class/interface đúng (extends/implements theo pattern dự án)
- [ ] Annotations đúng (theo layer tương ứng của dự án)

**MUST create interface+impl pair** when extracting logic with dependencies (trừ khi dự án không dùng pattern này —
verify từ peer files).

---

## Step 5 — Post-Extract Verification

1. GKG `get_references` → confirm both callers use new service
2. `grep_search("{old method}")` → confirm no duplicate remains
3. Build/compile check
4. `codebase_graph_circular()` → no new circular deps
5. Sync tasks.md with actual code (deps, method names, signatures)

---

## Step 6 — Documentation Rules

- EXTRACT → create `impact_analysis.md` + add EXTRACT layer to `tasks.md`
- tasks.md has action items only + 1-line ref to impact_analysis.md
- Post-extract checklist in tasks.md (callers updated, no duplicate, build OK, graph OK)

### DTO Reuse Rule

- Don't create new DTO if existing one has ≥80% matching fields → REUSE
- Batch handler: build existing Request DTO per item, NOT new Context DTO

### Base API Check

1. GKG `search_definitions(["{guessed name}"])` → if name known
2. SC `codebase_search("{description}")` → if name unknown
3. `grep_search("{exact name}")` → confirm
4. External JAR → ask user. **NEVER guess method names.**

---

## Clone Analysis (for new component ← existing component)

**When:** Design references existing component ("similar to X", same base class, batch variant).

### Phase 1 — Structural Scan

- `repo_map(project, ["{source file}"], depth=2)` → method signatures
- `codebase_graph_query("{source file}")` → dependencies

### Phase 2 — Semantic Block Discovery

- GKG `search_definitions` for each method (low cost)
- SC `codebase_search` if GKG insufficient (high cost)

### Phase 3 — Targeted Read + Classify

- `read_definitions(file, [methods])` → read only matched methods
- Classify: EXTRACT (≥80%, ≥5 lines) / PARAMETERIZE (60-80%) / STATIC_UTIL (stateless) / UNIQUE (skip) / ADAPT (>40%
  changes)

### Phase 4 — Output Reuse Map

```
📋 Clone Analysis: {New} ← {Source}
  Method: {name}()
    🔴 Block 1: {desc} → EXTRACT → {Service.method(params)}
    🟡 Block 2: {desc} → PARAMETERIZE → param: {varying}
    🔵 Block 3: {desc} → STATIC_UTIL → {Utils.method()}
    ⚪ Block 4: {desc} → UNIQUE (skip)
```

### Phase 5 — Inject into tasks: Extraction tasks → Refactor source → New component
