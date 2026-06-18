---
description: Load context from artifact_context.yml, check schema/state, then implement tasks with knowledge-aware approach
---

## EXECUTION CONTRACT

Steps 1→8 sequential. Each task: validate → if fail → retry 2x → STOP with error.

**DO NOT:** skip knowledge load (1), skip profile lock (3), skip per-task validation (4b), skip tasks.md sync (5), skip file-back (6), implement without knowledge in memory, create duplicates when existing files can be modified.

---

## SHARED RULES

> 📖 Base rules: See [`_shared_rules.md`](./_shared_rules.md) for: **CONTEXT PRIORITY**, **ANTI-HALLUCINATION**, **TOOL PRIORITY**, **DIFF AWARENESS**.
> Workflow-specific overrides below.

### Apply-specific Overrides

- **CONTEXT PRIORITY** (6 levels — adds propose artifacts + brainstorm):
  | 1.5 | `brainstorm_notes.md` (if exists) | Design direction — supplements #1 |
  | 3 | Propose artifacts (`design.md`, `srs.md`) | Architecture decisions, class mappings |
  Conflict → higher number loses. If design.md conflicts with pre_openspec.md → follow pre_openspec.md.
  Brainstorm supplements pre_openspec (refines, does NOT override). If brainstorm contradicts pre_openspec → IGNORE brainstorm.
  **Override semantics:** P3 overrides P4 for architecture; P4 overrides P3 for coding conventions.
- **ANTI-HALLUCINATION**: Same base. Missing entity fields → `// TODO: cần bổ sung`.
- **DIFF AWARENESS**: Same base. Modify existing files first. Create new ONLY with justification.

---

**Pipeline**: `/wf_pre_openspec` → `/wf_brainstorm_openspec` → `/wf_openspec` → **`/wf_openspec_apply`** → `/wf_integ_test` ⟂ `/wf_client_doc` → `/wf_archive`

**Input**: Change name (kebab-case).

**Fluid Workflow Integration** (from opsx-apply):
- **Can be invoked anytime**: Before all artifacts are done (if tasks exist), after partial implementation, interleaved with other actions
- **Allows artifact updates**: If implementation reveals design issues, suggest updating artifacts — not phase-locked, work fluidly
- **Resumable**: Can pause and continue across sessions

---

## STEP 1 — Load Knowledge

### STEP 1.5 — Schema & State Check (from opsx-apply)

```bash
openspec status --change "<name>" --json
```

Parse JSON to understand:
- `schemaName`: The workflow schema being used (e.g., "spec-driven")
- `artifacts`: completion status of all artifacts

```bash
openspec instructions apply --change "<name>" --json
```

Returns:
- `contextFiles`: artifact ID → array of concrete file paths (varies by schema)
- Progress (total, complete, remaining)
- Task list with status
- Dynamic instruction based on current state

**Handle states:**
- If `state: "blocked"` (missing artifacts): show message, suggest running `/wf_openspec`
- If `state: "all_done"`: congratulate, suggest `/wf_archive`
- Otherwise: proceed to knowledge load

### STEP 1a — Static Knowledge

Read `artifact_context.yml` → `apply` section. Resolve `context`/`rules`/`skills` paths.

**1a. Static Knowledge (from `apply.context`):**
- `base_knowledge/structures/apply/**` — code practices (logging, exception, validation)
- ~~`base_knowledge/structures/propose/**`~~ — **DO NOT load upfront.** Architecture decisions already embedded in `design.md` + `tasks.md`. Load individual propose files ONLY when a task requires a pattern not covered in self-contained tasks.md.

> ⚠️ **DO NOT load `base_knowledge/concepts/**`** — concepts (class relationship map) are for design phase only. At apply time, `design.md` + `tasks.md` + dynamic context already specify exactly what to build. Loading 25 concept files wastes ~50KB tokens and adds noise.

**1b. Dynamic Context (LAZY LOAD):**
`openspec/context/generated/<name>/` → if exists, **DO NOT** read all `*.md` upfront.
Instead: note available files → load on-demand when specific task requires context.
Alternative: `SC codebase_context_search("<query>")` for project conventions.
If dir not exists → WARN, continue.

**1c. Rules + Skills:**
Load from `apply.rules` and `apply.skills` in `artifact_context.yml`.

Show: `Context: N | Rules: N | Skills: N | Dynamic: ✓/✗ | Concepts: SKIPPED`

---

## STEP 2 — Read Propose Artifacts

From `openspec/changes/<name>/`:

**2a. Detect tasks.md format:**
- Check for `<!-- self-contained: true -->` marker in tasks.md
- IF self-contained → read **ONLY** `tasks.md` (+ `impact_analysis.md` if exists)
- ELSE (legacy format) → read all: `proposal.md`, `design.md`, `srs.md`, `tasks.md`,
  `impact_analysis.md` (if exists)

**2a-bis. Load brainstorm context (if exists):**
- Check `openspec/changes/<name>/brainstorm_notes.md`
- IF exists AND self-contained tasks.md → read ONLY:
  - `## Selected Direction` → verify alignment with tasks.md approach
  - `## Open Questions for Design Phase` → flag any `[OPEN]` items as ⚠️ WARN
- IF exists AND legacy format → read full file → supplement design context
- IF not exists → SKIP silently (brainstorm is optional)

> Brainstorm context provides design-direction hints and surfaces unresolved questions.
> It NEVER overrides pre_openspec.md or design.md decisions.

**2b. Extract:** FR list, task count, planned files, inline context per task.

**Validate:** tasks.md exists and loaded. If self-contained → verify inline context fields present.

> **Token savings**: Self-contained tasks.md = ~10,000 tokens saved (skip 6 files).

---

## STEP 3 — Lock Feature Profile

**HARD CONSTRAINT** for ALL code generation.

> ⚠️ **Token optimization:** WF2 Step 6 already locked the profile and embedded it in `design.md`.
> Read from `design.md` FIRST — only cascade to raw knowledge files if `design.md` is missing or incomplete.

**Fast-path:** If `design.md` contains `## Locked Profile` section with all 4 fields
(flow, factory, feature_type, transaction_flow) → read from design.md → SKIP cascade below.
Output: `"Profile loaded from design.md (fast-path)"`. Saves ~800-1500 tokens.

**Slow-path (fallback):** Only when design.md missing or profile section incomplete:

### Flow + Factory

> **Priority order (highest → lowest):**
> 1. `design.md` locked profile section (established in WF2 Step 6) — **read this first**
> 2. `base_knowledge/structures/propose/knowledge_transaction_flow.md` — project-specific (fallback)
> 3. `.agent/common-rules/default_profile.md` — baseline defaults (Java)
> 4. If none exists → STOP, ask user for flow/factory configuration

Read Flow/Factory/Base Handlers from the highest-priority source found. Lock as HARD CONSTRAINT.

### Feature Type — READ from `pre_openspec.md`

Read `> **Type**:` and `> **Transaction Flow**:` headers from `pre_openspec.md`.
Lock as HARD CONSTRAINT. DO NOT re-classify (already done in WF1/WF2).

**Override**: ONLY re-classify if:
- User explicitly requests, OR
- Step 4 implementation reveals contradicting evidence (e.g., NEWBUILD classified but existing handler found)
- In that case: STOP → ask user to confirm → update `pre_openspec.md` → relock profile before continuing

---

## STEP 4 — Implement Tasks

Delegate to `openspec-apply-change` skill with loaded context + profile.

**Handoff protocol** — write persistent marker + pass context so skill SKIPS Step 6 (classification):

**4-pre. Write pipeline marker to tasks.md** (MANDATORY before invoking skill):
If `tasks.md` does not already contain a pipeline marker, prepend:
```markdown
<!-- pipeline: wf_openspec_apply -->
<!-- locked_profile: { flow: "<flow>", factory: "<factory>", feature_type: "<type>", transaction_flow: "<tflow>" } -->
<!-- context_loaded: true -->
<!-- reuse_rules_loaded: true -->
```
This ensures the skill can detect pipeline context even in multi-turn conversations.

**Pass to skill:**
- `locked_profile`: `{ flow, factory, feature_type, transaction_flow }` from Step 3
- `change_name`: `<name>`
- `context_loaded`: `true`
- `reuse_rules_loaded`: `true`

> ⚠️ **Scope**: Skill handles ONLY the implementation loop (Step 4). Steps 5-8 (Sync, File-Back, Cleanup, Validate) belong to THIS workflow and run after the skill completes.

### 4-init (ONE TIME):
- Read `common-rules/reuse_rules_compact.md`
- Read dynamic context files if needed

### 4a. Per task — Source Code Reading:

> **Choose tool by query type, prefer cheaper tools first.**
> Stop at first tier providing sufficient context. Token cost: GKG (~50/match) → grep (~30/match) → SC (~500/match) → view_file (~3/line).

**Decision matrix:**

| Task Type | Exact (GKG/grep) | Semantic (SC) | Dependencies (graph/GitNexus) | Fallback (file) |
|-----------|:------:|:------:|:------:|:------:|
| `[NEW]` without Source | Skip | Skip | Skip | Skip |
| `[NEW] + Source: X` | ✅ read methods | ✅ discover (if exact insufficient) | Skip | Fallback |
| `[MODIFY]` | ✅ read methods | ✅ discover (if exact insufficient) | ✅ `gitnexus context` (callers) | Fallback |
| `[EXTRACT]` | ✅ read methods | ✅ discover | ✅ `gitnexus impact` (blast radius) | Fallback |

> **Project-agnostic**: Fallback file types vary by project. DO NOT hardcode — detect at runtime.

### 4a-gitnexus. Pre-task Impact Check (for [EXTRACT] and [MODIFY] tasks)

Before modifying shared code:
1. `gitnexus impact "{symbol}" upstream` → verify blast radius matches `impact_analysis.md`
2. If blast radius > expected → WARN → ask user before proceeding
3. For `[EXTRACT]` tasks: `gitnexus context "{symbol}"` → verify all callers before extracting
4. Plan update order: interfaces → implementations → callers → tests

> If GitNexus unavailable → rely on `impact_analysis.md` from Step 6.5. No block.

### 4b. Per-Task Validation (MANDATORY)

| Check | Condition |
|-------|-----------|
| File exists | Created/modified on disk |
| Package | Matches knowledge_architecture.md |
| Base class | Handler extends correct base from profile |
| Factory | Extends correct base + correct interface |
| Flow phases | Count matches profile |
| Method pattern | getData(filter) if BaseClientDataFactory |
| Naming | Matches conventions |
| No hallucination | No invented APIs/tables/fields |

**Profile violation → REWRITE file immediately** (not just flag).

### 4c. Code Evidence (MANDATORY per task)

```
Task N: <title>
  File:    <path>
  Class:   <Name> extends <Base>
  Methods: <list>
  Status:  ✓ VALIDATED
```

Cannot provide evidence → task NOT complete.

---

## STEP 4.5 — Extended Tracking Artifacts (Optional, Auto-Run)

> Merged from `openspec-apply-change` skill. Generates deployment-review artifacts.
> Runs after ALL tasks in Step 4 complete. No user prompt needed.

**Skip gate:** If `files_created = 0` AND `files_modified ≤ 2` → SKIP this step.
Output: `"Extended tracking skipped — trivial change."` Saves ~500 tokens.

### 4.5a. Generate/update in `openspec/changes/<name>/`:

| Artifact | When to Create | Content |
|----------|---------------|---------|
| `todo-uncover.md` | If any `TODO`, `FIXME`, or uncovered edge cases found during implementation | Unresolved items list |
| `new-apis.md` | If new endpoints were added | Path, HTTP Method, Request/Response payload, purpose |
| `delta-spec.md` | If impact scope exists (from `impact_analysis.md`) | Delta between original behavior and new implementation |

### 4.5b. Rules:
- Generate ONLY artifacts that have content — do NOT create empty files
- `todo-uncover.md`: scan all created/modified files for `TODO`, `FIXME`, `HACK` comments
- `new-apis.md`: extract from tasks.md entries with Action: `[NEW]` that contain controller/endpoint files
- `delta-spec.md`: only if `impact_analysis.md` exists AND classification ≠ NEWBUILD

Show: `Extended: todo-uncover={✓/✗} | new-apis={✓/✗} | delta-spec={✓/✗}`

---

## STEP 5 — Sync tasks.md

`tasks.md` = source of truth for ALL code changes.

**5a.** Planned task done → mark `[x]` immediately. NO batch updates.

**5b.** User-requested change → add new entry:
```markdown
- [x] **Task N: <description>**
  - File: `<path>` | Action: <what> | Lý do: <why>
```

**5c.** Self-fix: small (import/typo) → no new task. Logic change → add task.

**5d. Consistency check (after ALL tasks):**
- task_count must ≥ file_changes
- Task must reflect actual files (not planned)
- No `- [ ]` remaining

---

## STEP 6 — Knowledge File-Back (Auto-Run)

Runs automatically after ALL tasks. No user prompt.

**Skip gate:** If `classification = MAINTENANCE` AND `files_created = 0` → SKIP this step.
Output: `"Knowledge file-back skipped — MAINTENANCE with no new files."` Saves ~500-1K tokens.

### 6a. Scan all created/modified files.

### 6b. Create concept ONLY if:
- Pattern in ≥2 files, OR affects core business logic
- Can link to ≥2 existing concepts
- DO NOT create for: simple DTOs, config, one-off utils

### 6c. Update/Create
- **Existing:** append `## Updates` section with date + change-name
- **New:** Follow concept format: 4 sections (Overview, Implementation, Integration, Related) + ≥2 wikilinks.
  > **Assessment tools** (before creating):
  > - `SC codebase_context_search("{pattern name}")` → check if similar concept already documented
  > - `gitnexus query "{class name}"` → discover related processes and integration points
  > - `gitnexus context "{symbol}"` → verify callers/callees for accurate Integration section

### 6d. Safe Mode: >3 new concepts → ask user confirmation.

### 6e. Validate each: 4 sections, no placeholder, real class names, ≥2 wikilinks, domain tag. Fail → discard.

### 6f. Update `base_knowledge/concepts/index.md`.

Show: `Scanned: N | Updated: U | Created: W | Discarded: D`

---

## STEP 7 — Cleanup Junk Files (Auto-Run)

> ⚠️ **BẮT BUỘC** — Chạy tự động sau tất cả tasks, TRƯỚC final validation.
> Rule chi tiết: Xem các bước 7a-7c bên dưới.

### 7a. Quét file rác

Scan toàn bộ project (trừ `target/`, `build/`, `.idea/`, `.git/`) tìm:
- `*.log`, `*.log.*` — Log files sinh từ runtime/debug
- `*.orig`, `*.bak`, `*.swp`, `*.tmp` — Editor/merge temp files
- Thư mục `logs/`, `log/`, `test-output/` ở root

**JVM-only patterns** (chỉ khi `{PROJECT_LANG}` = `java`/`kotlin`):
- `hs_err_pid*`, `replay_pid*` — JVM crash logs
- `*.hprof`, `*.jfr` — Heap dump, flight recorder
- `*.class` ngoài `target/`/`build/` — Compiled class rác
- `surefire-reports/` ở root — Maven test reports

### 7b. Xóa tất cả file rác tìm được

> ⚠️ **Cross-platform:** Agent MUST detect OS from system metadata and use appropriate commands.
> Generate cleanup command at runtime from the pattern list in 7a. Use `Remove-Item` (Windows) or `find -delete` (Linux/Mac).
> Exclude directories: `target/`, `build/`, `.idea/`, `.git/`, `node_modules/`.
> Also remove junk directories at root: `logs/`, `log/`, `test-output/`, `surefire-reports/`.
> Also remove stray compiled files (e.g., `*.class` in `src/` for Java projects — detect by `{PROJECT_EXT}`).

### 7c. Báo cáo

```
🧹 Cleanup: Removed N file(s) — [list]
🧹 Cleanup: Clean — no junk files found
```

---

## STEP 8 — Final Validation

| Check | Condition |
|-------|-----------|
| Tasks complete | No `- [ ]` remaining |
| Tasks ↔ code | Every file change has entry |
| Profile | ALL files follow locked profile |
| No placeholders | Zero in code |
| No duplicates | No 2 handlers/factories for same purpose |
| FR coverage | Every FR-xxx addressed |
| Knowledge | New concepts valid, index updated |
| **Junk files** | **Zero log/class/tmp/crash files in source tree** |
| **Walkthrough** | **NO full file diffs** — summary tables + file links only (diffs via VCS) |
| **Impact scope** | **Affected processes ⊆ expected scope** (see 8a-gitnexus) |

### 8a-gitnexus. Post-Implementation Verification (when GitNexus available)

1. `gitnexus detect_changes scope="all"` → map all uncommitted changes to affected processes
2. Compare affected processes vs expected scope from `impact_analysis.md`
3. If unexpected processes affected → WARN → show diff → ask user to review
4. Log verification result in summary

> Especially valuable for `[EXTRACT]` tasks — ensures no unintended side effects.
> If GitNexus unavailable → skip this check (manual review covers it).

**Walkthrough format (HARD RULE):**
- Use `render_diffs(file:///path)` shorthand — NOT inline code blocks with full file content
- Summary table per component: `File | Action | What changed (1 line)`
- Link to file with line range for details: `[ClassName](file:///path#L10-L30)`
- MAX 3 lines of description per file in walkthrough
- If >10 files changed → group by component, not individual file listings

```
═══════════════════════════════════════
APPLY COMPLETE
═══════════════════════════════════════
Change:   <name>
Profile:  <flow> | <factory> | <type>
Tasks:    <planned> + <added> = <total> completed
Files:    created <N> / modified <M>
Knowledge: updated <U> / created <W>
Cleanup:  removed <J> junk file(s)
FRs:      <covered>/<total>
═══════════════════════════════════════
Ready: /wf_integ_test <name>  OR  /wf_client_doc <name>
      /wf_archive <name>  (if skipping test/docs)

💡 If code changed after spec generation, run `sync-task <name>` before archiving.
```

---

## Output Templates (from opsx-apply)

**During Implementation:**
```
## Implementing: <change-name> (schema: <schema-name>)

Working on task 3/7: <task description>
[...implementation happening...]
✓ Task complete

Working on task 4/7: <task description>
[...implementation happening...]
✓ Task complete
```

**On Pause (Issue Encountered):**
```
## Implementation Paused

**Change:** <change-name>
**Schema:** <schema-name>
**Progress:** 4/7 tasks complete

### Issue Encountered
<description of the issue>

**Options:**
1. <option 1>
2. <option 2>
3. Other approach

What would you like to do?
```
