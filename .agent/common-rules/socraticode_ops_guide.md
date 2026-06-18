# SocratiCode — Operations Guide

## Re-indexing Policy

### When to re-index (explicit calls)

| Event                                             | Action                                               | Tool                     |
|---------------------------------------------------|------------------------------------------------------|--------------------------|
| After `/wf_learn_base_code or /wf_learn_old_code` | Full index + context artifacts                       | Step 15 (automatic)      |
| After `/opsx-archive` or `/ctm-archive`           | Re-index context + rebuild graph if needed           | Archive step (automatic) |
| After**branch switch** (`git checkout`)           | Run `codebase_update` to ensure correct branch index | `codebase_update`        |
| After**git pull** with ≥5 files changed           | Incremental update                                   | `codebase_update`        |

### When NOT to re-index

| Event                        | Why skip                                     |
|------------------------------|----------------------------------------------|
| After each task completion   | FILE_WATCHER auto-syncs source code changes  |
| During task implementation   | Read-only queries only (search, graph_query) |
| When another dev pushes code | Each dev has local Qdrant — no shared state  |

### Branch switch awareness

SocratiCode index reflects the **current working directory state**, not a specific git branch.
After switching branches:

1. FILE_WATCHER will detect file changes automatically (add/remove/modify)
2. **Always run `codebase_update`** to ensure index reflects the new branch correctly
3. Run `codebase_status` to verify index is green before searching
4. If ≥5 files changed between branches → also run `codebase_graph_build` to update dependency graph

> **CAUTION**: Without explicit `codebase_update` after branch switch, search results may include stale data from the
> previous branch. Always update before relying on search results.

---

## Pre-flight Check (trước khi search lần đầu trong session)

> ⚠️ **Mục đích:** Tránh false-negative (search trả rỗng vì chưa index) → AI nghĩ logic NEW → viết duplicate code.

> See `GEMINI.md` → Pre-flight (1x per session) for the canonical procedure (SC `codebase_status` + GKG verification).
>
> **Key rules (summary):**
> - SC: `codebase_status(projectPath)` → verify indexed. If not → SC = UNAVAILABLE (do NOT call `codebase_index`)
> - GKG: Try any GKG tool → verify responding. If fails → fallback SC + grep
> - **DO NOT** use `codebase_list_projects()` — wastes ~40 tok/project

---

## Team Setup (Plan B: Shared Knowledge, Local Index)

### Architecture

```
Shared via Git (commit + push):
  .socraticodecontextartifacts.json   ← artifact registry
  base_knowledge/                     ← architectural knowledge files
  GEMINI.md                           ← search strategy + rules

Local per dev (Qdrant localhost):
  codebase index                      ← source code (auto via FILE_WATCHER)
  context artifacts index             ← knowledge (auto on first search)
  code graph                          ← dependencies (auto on first query)
```

### First-time setup (new dev joining team)

1. Clone project + `git pull` (get latest knowledge files)
2. SocratiCode auto-manages Qdrant Docker container (`QDRANT_MODE: managed`)
3. First `codebase_search` → triggers automatic source code indexing (~5-10 min)
4. First `codebase_context_search` → triggers automatic knowledge indexing (~1 min)
5. First `codebase_graph_query` → triggers automatic graph build (~2 min)
6. Sau đó: tất cả search sẵn sàng ✓

### After `git pull` (ongoing)

- Source code: FILE_WATCHER auto-detects changes ✓
- Knowledge files: if `base_knowledge/` changed → `codebase_context_search` auto-detects stale ✓
- No manual action needed in most cases

### Who re-indexes?

| Action                             | Who                | When                               |
|------------------------------------|--------------------|------------------------------------|
| Full learn (`/wf_learn_base_code`) | 1 người (lead)     | Đầu sprint hoặc thay đổi kiến trúc |
| Commit knowledge files             | Lead sau khi learn | Sau `/wf_learn_base_code`          |
| Source code index                  | Mỗi dev tự động    | FILE_WATCHER (local)               |
| Context artifacts                  | Mỗi dev tự động    | Auto-detect stale (local)          |
