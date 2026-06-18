---
description: Load context from artifact_context.yml and generate all OpenSpec artifacts (propose phase)
---

## EXECUTION CONTRACT

Steps 1→8 sequential. Each step: validate → if fail → retry 2x → **FALLBACK** (mark `⚠️ PARTIAL`, skip dependents, log in summary).

**DO NOT:** skip profile lock (Step 6), skip template read (7a), skip cross-check (7b), generate out of dependency order, copy context/rules into artifact output.

---

## SHARED RULES

> 📖 Base rules: See [`_shared_rules.md`](./_shared_rules.md) for: **CONTEXT PRIORITY**, **ANTI-HALLUCINATION**, **TOOL PRIORITY**, **DIFF AWARENESS**.
> Workflow-specific overrides below.

### OpenSpec-specific Overrides

- **CONTEXT PRIORITY**: No override — follows base (4 levels).
- **ANTI-HALLUCINATION**: Same base rules. Additionally: ALL assumptions MUST be tagged inline.
- **TOOL PRIORITY**: Same hybrid search. GitNexus GRAPH search added at Phase 1.5 (see Step 5a).
- **DIFF AWARENESS (MAINTENANCE)** 
---



**Pipeline**: `/wf_pre_openspec` → `/wf_brainstorm_openspec` → **`/wf_openspec`** → `/wf_openspec_apply` → `/wf_integ_test` ⟂ `/wf_client_doc` → `/wf_archive`

**Input**: Change name (kebab-case). Must contain `pre_openspec.md`.

**Idempotency:** If artifacts exist → ask: **Overwrite** | **Merge** (default) | **Skip**. No response → Merge.

---

## STEP 1 — Verify pre_openspec.md

Check `openspec/changes/<name>/pre_openspec.md` exists. If missing → STOP → run `/wf_pre_openspec`.
If change dir missing → `openspec new change "<name>"`.

**Validate:** exists, non-empty, has `> **Type**:`, has `## 10. DETECTED SCOPE`.

---

## STEP 2 — Load Dynamic Context

Check `openspec/context/generated/<name>/`. If exists → read all `*.md` (priority 2). If not → WARN user, wait confirmation, continue with static.

**Validate:** status determined. If loaded → ≥1 file.

---

## STEP 3 — Load Context

Read `artifact_context.yml` + `config.yaml`.

> **Path resolution:**
> `artifact_context.yml` is dynamically generated per change when running WF1 (`openspec new change`).
> Primary location: `openspec/changes/<name>/artifact_context.yml` (change-specific — always check first).
> Fallback: `openspec/mapping/artifact_context.yml` (shared/global — only if change-specific not found).

> **Fallback**: If `artifact_context.yml` not found in either location → WARN user, use defaults:
> environment from project root detection (see `scan_rules.md` → LANGUAGE DETECTION),
> no custom rules/skills, static knowledge only. Continue with reduced context.

**3a.** Global: `input.primary`, `environment`, `rules`.
**3b.** Per-artifact: resolve `context`/`rules`/`skills` paths. Merge order = priority table above.
**3c.** Read `pre_openspec.md` → extract: FR list, feature type, flow type, scope.

**3d.** If `brainstorm_notes.md` exists in change dir → read and extract:
  - **Selected Direction** → influences design approach in Step 7
  - **Pre-classifications** → cross-validate with Step 6 profile lock
  - **GitNexus Findings** → supplements Phase 1.5 graph search (Step 6.5a)
  - **Open Questions for Design Phase** → flag as items needing resolution in artifacts
  
  **Validation (MANDATORY when brainstorm_notes loaded):**
  - MUST have `## Selected Direction` section with non-empty content
    → If missing/empty → WARN `"⚠️ Incomplete brainstorm — no direction selected"` → continue without direction hint
  - `## Pre-classifications` MUST NOT conflict with `pre_openspec.md` classifications
    → If conflict → log `⚠️ Brainstorm pre-class conflicts with URD analysis — using URD` → use pre_openspec values
  - `## Open Questions for Design Phase` → each item SHOULD be tagged `[RESOLVED]` or `[OPEN]`
    → `[OPEN]` items → inject into artifact generation as `⚠️ OPEN QUESTION:` markers
    → If no tags → treat all as `[OPEN]` and WARN
  - If YAML frontmatter present (`type: brainstorm_notes`) → validate `status: complete`
    → If `status` ≠ `complete` → WARN `"⚠️ Brainstorm incomplete — proceed with caution"`
  
  > If brainstorm_notes.md not found → SKIP silently (brainstorm is optional).

> ⚠️ **LAZY LOAD for knowledge files:** DO NOT read all `base_knowledge/**` upfront.
> 
> **SC-first strategy per artifact (Step 7):**
> 1. SC `codebase_context_search("{artifact topic}", projectPath)` → find relevant knowledge chunks (~800 tokens vs ~5K full file)
> 2. If SC returns insufficient → read specific `base_knowledge/` file by path from `artifact_context.yml`
> 3. NEVER read ALL knowledge files — only those relevant to current artifact
> 4. Read selected concept files → extract class relationships, wikilinks
> 5. SKIP concepts in unrelated domains
> This saves ~15-25K tokens of unused context.

**Validate:** both configs loaded, FR list extracted, priority established.

---

## STEP 4 — Verify .openspec.yaml

Exists in change dir (DO NOT overwrite). Show: context/rules/skills counts, dynamic status, FR count.

---

## STEP 5 — Artifact Build Order

`openspec status --change "<name>" --json` → parse artifacts, dependencies, status.

Parse JSON to get:
- `applyRequires`: array of artifact IDs needed before implementation (e.g., `["tasks"]`)
- `artifacts`: list with status and dependencies

**Rule:** ALL `applyRequires` MUST be `done` before generating. Circular → STOP.

After generating each artifact, re-run `openspec status --change "<name>" --json` to verify progress.

---

## STEP 6 — Lock Feature Profile

Cross-reference knowledge → lock profile as **HARD CONSTRAINT**.

### 6a. Flow

> **Dynamic override (priority order):**
> 1. `base_knowledge/structures/propose/knowledge_transaction_flow.md` — project-specific (highest)
> 2. `.agent/common-rules/default_profile.md` — baseline defaults (Java/Kotlin only — non-Java projects will fall through to option 3)
> 3. If neither exists → STOP, ask user for flow configuration

Read Flow table from the highest-priority file found. Lock as HARD CONSTRAINT.

### 6b. Factory

> **Dynamic override:** Same priority order as 6a above.

Read Factory table from the same file. Lock as HARD CONSTRAINT.

### 6c. Type — READ from `pre_openspec.md` (DO NOT re-classify)

Read `> **Type**:` header from `pre_openspec.md` → lock as MAINTENANCE / EXTEND / NEWBUILD.
Classification was already performed in `/wf_pre_openspec` (Step 1d).
Re-classifying wastes tokens and risks conflicting results.

**Override:** Only re-classify if user explicitly requests OR new evidence contradicts.

Print locked profile with evidence before proceeding.

### 6d. Verify Base Classes (apply Tool Priority)

> **Skip guard:** If `classification = MAINTENANCE` → SKIP this step entirely.
> Base classes are already verified in the codebase; MAINTENANCE only modifies existing logic.
> Output: `"Base class verification skipped — MAINTENANCE change."` Saves ~800-1.5K tokens.

> Follow **GEMINI.md Search Strategy.** Prefer cheaper tools first.
>
> **Language guard:** If project has `knowledge_transaction_flow.md` → verify items from that file.
> If `{PROJECT_LANG}` = `java`/`kotlin` AND no project-specific file → use table below.
> Otherwise → SKIP this verification, ask user for base class patterns.

| Verify Item | Expected | Primary (P1) | Fallback (P2) |
|---|---|---|---|
| Base handler class | Package + generic params | GKG `search_definitions` | grep |
| Handler lifecycle methods | preHandle → aroundHandle → postHandle | GKG `read_definitions` | grep |
| gRPC interface | IGrpcHandler generic params | GKG `search_definitions` | grep |

---

## STEP 6.5 — Reuse & Impact Analysis (MANDATORY before tasks generation)

> 📐 **Output format:** `.agent/skills/codebase-management/templates/impact_analysis_template.md`
> 🔍 **Execution:** Inline below (sourced from `reuse_rules_compact.md` Steps 1-3). Full file loaded separately in apply phase only.

**Skip guard:**
- If `classification = NEWBUILD` AND `archive_found = false` → SKIP this step entirely.
  Output: `"No reuse candidates — NEWBUILD with no archive."` Saves ~3-5K tokens.
- If `classification = MAINTENANCE` AND `FR_count ≤ 3` AND no `[ENRICHED]` FRs → SKIP this step.
  Output: `"Impact analysis skipped — small MAINTENANCE change."` Saves ~3-5K tokens.

**Input**: Locked profile (Step 6) + pre_openspec.md (FR list + DETECTED SCOPE)

**Process**: For each **major logic block** identified in FRs/scope → run escalation search:

### 6.5a. Escalation Search (4 phases)

| Phase | When | Tool | Stop condition |
|-------|------|------|---------------|
| 🟢 Phase 1: EXACT | Always, parallel | `grep_search("{name}", Includes=["{PROJECT_EXT}"])` + GKG `search_definitions` | ≥1 match with ≥80% similarity |
| 🔵 Phase 1.5: GRAPH | Always, parallel with Phase 1 | `gitnexus impact "{symbol}" upstream` + `gitnexus context "{symbol}"` | Blast radius mapped |
| 🔴 Phase 2: SEMANTIC | Only if Phase 1+1.5 insufficient | SC `codebase_search("{action} {object}", languageFilter="{PROJECT_LANG}", limit=15)` | Match found |
| 🟡 Phase 3: CONTEXT | Optional, convention check | SC `codebase_context_search("{pattern}")` | Only when needed |

> **Phase 1.5 runs in PARALLEL with Phase 1.** No extra latency.
> GitNexus provides: blast radius (d=1,2,3), affected processes, confidence scores.
> If Phase 1 + 1.5 are sufficient → SKIP Phase 2 (saves ~500 tokens/match).
> If GitNexus index unavailable → skip Phase 1.5 silently, proceed with Phase 1 + Phase 2.

> Follow **GEMINI.md Search Strategy** table. Pipeline-specific exceptions:
> - GKG `read_definitions` for method bodies (NOT full class — saves 40-60% tokens)
> - `grep_search` for DI references (SC `codebase_impact` may miss injected refs)
> - Output format: `file:///` absolute links (NOT path abbreviations)

### 6.5b. Decision Framework

| Match % | Decision | Action |
|---------|----------|--------|
| 100%, shared class | **REUSE** | Import + call |
| ≥80%, feature class | **EXTRACT** | Move to common → both use |
| 50-80% | **EXTRACT** | Parameterize differences |
| <50% or not found | **NEW** | Write new, document reason |

> 🔴 **HARD BLOCK:** Match ≥80% → MUST EXTRACT. ≥3 occurrences → MUST EXTRACT (no exception).

### 6.5c. Impact Assessment (for EXTRACT/MODIFY — GitNexus-enhanced)

**Automated assessment** (when GitNexus available):
1. `gitnexus impact "{target}" upstream maxDepth=3` → automated blast radius
2. Map depth levels:

| GitNexus Depth | Impact | Action |
|---------------|--------|--------|
| d=1 ≤2 symbols | 🟢 Low | Extract safely |
| d=1 3-5 symbols | 🟡 Medium | Backward-compatible wrapper |
| d=1 >5 symbols | 🔴 High | Review, consider interface |

3. Cross-validate with manual analysis → resolve conflicts

**Manual fallback** (when GitNexus unavailable):

| Impact | Criteria | Action |
|--------|----------|--------|
| 🟢 Low (1-2 callers) | Few callers | Extract safely |
| 🟡 Medium (3-5) | Several callers | Backward-compatible wrapper |
| 🔴 High (>5) | Many/cross-module | Review, consider interface |

### Output: Generate `impact_analysis.md` + tasks.md refactoring layer

> Follow template in `.agent/skills/codebase-management/templates/impact_analysis_template.md`
> - Output 1: `impact_analysis.md` → write to `openspec/changes/<name>/impact_analysis.md`
> - Output 2: Refactoring tasks → **buffer in memory** (merge into `tasks.md` in Step 7 as `[EXTRACT]` layer)

**CRITICAL**: 
1. `impact_analysis.md` PHẢI được tạo TRƯỚC `tasks.md` → user review analysis trước
2. tasks.md chỉ chứa actionable items — KHÔNG copy analysis report vào
3. 🔴 High impact EXTRACT → PHẢI hỏi user confirm trước khi ghi vào tasks.md
4. Sau khi code xong → tasks.md PHẢI sync với code thực tế
5. `impact_analysis.md` PHẢI có đủ 5 sections: Core Files, Call Tree, Blast Radius, Reuse Map, Context Snapshot


---

## STEP 7 — Generate Artifacts

Loop `applyRequires` done → generate:

### 7a. Per artifact:

1. Load per-artifact context/rules/skills
2. `openspec instructions <id> --change "<name>" --json`
   - `context` and `rules` are **constraints for AI** — do NOT include in output
3. Read template from `schemas/<schema>/templates/`
4. Apply profile constraints (below)
5. Read dependency artifacts
6. Generate following template structure
7. Apply context priority on conflicts

### 7a-enrich. tasks.md Enrichment (MANDATORY for `tasks` artifact)

> **Purpose**: Make tasks.md **self-contained** so apply phase reads ONLY this file — no need to separately load design.md, srs.md, or context files. Saves ~35% apply-phase tokens.

For each task block in tasks.md, embed inline context:

```markdown
- [ ] **Task N: <title>**
  - File: `<path>` | Action: [NEW] / [MODIFY] / [EXTRACT]
  - Base: `<BaseClass>` from `<package>` ← (from design.md)
  - FR: FR-xxx — <description> ← (from srs.md)
  - Error: `<ERROR_CODE>` (nếu có) ← (from srs.md)
  - Pattern: <naming/structure convention> ← (from dynamic context)
  - Dependencies: <imports, factories, clients> ← (from design.md component mapping)
  - Source: `<existing file>` (nếu [EXTRACT]) ← (from impact_analysis.md)
```

**Rules**:
- Embed ONLY info relevant to that specific task — NOT full design/srs dump
- If info not available for a field → omit the field (không dùng N/A)
- Add metadata header: `<!-- self-contained: true -->` ← apply phase detects this marker
- **Project-agnostic**: fields tùy thuộc vào project archetype (old/new basecode, hybrid, library)


### 7b. Template Enforcement (STRICT)

**Structure:**
- MUST preserve ALL template sections in ORDER
- MUST NOT add/duplicate sections
- Remove conditional sections ONLY if profile forbids

**Field-level validation:**
- Required fields within section MUST be non-empty (or explicit `N/A`)
- Field types MUST match template intent:
  - API endpoint fields → must contain HTTP method + path
  - Class reference fields → must contain full package.ClassName
  - Flow description fields → must match locked flow phases
- If section exists but content is semantically wrong type → regenerate section

**Section count** in output = template sections − removed conditionals. Mismatch → regenerate.

### 7c. Flow/Factory Enforcement

> **Language guard:** Class-specific rules below apply when `{PROJECT_LANG}` = `java`/`kotlin`.
> For other languages, enforce flow constraints (Query/Command/Financial/NonFinancial) but skip Java class name checks.

- Query → NO write/transaction. Command → NO multi-phase.
- NonFinancial → NO OTP/AuthMethod. Financial → MUST Init+Auth+Confirm.
- BaseCrudDataFactory → DB refs. BaseClientDataFactory → API client refs. _(Java/Kotlin only)_

### 7d. Post-Validation (MANDATORY per artifact)

| Check | Condition |
|-------|-----------|
| Artifact exists | File written to disk, non-empty |
| Template structure | Sections match, order correct, fields non-empty |
| Flow type | Matches profile |
| Base classes | Match context |
| FR traceability | ALL FR-xxx covered |
| No hallucination | No invented APIs/tables |
| No placeholders | Zero `{placeholder}` |
| Field semantics | Field content matches expected type |

**Traceability:** Count FR in pre_openspec vs artifact. If mismatch → identify missing → regenerate.

Fail → fix + re-validate (max 2). Then confirm: `"✓ Created <id>"`.

### 7e. Cross-Verify Generated Names (MANDATORY — except MAINTENANCE)

> **Skip guard:** If `classification = MAINTENANCE` AND no `[NEW]` artifacts generated → SKIP this step.
> MAINTENANCE changes modify existing names, not introduce new ones.
> Output: `"Cross-verify skipped — MAINTENANCE with no new names."` Saves ~1-2K tokens.

Sau khi generate TẤT CẢ artifacts, apply **Tool Priority P1+P2 parallel** → escalate P3 nếu mismatch:

| Category | Tool (Priority) | Example |
|---|---|---|
| Class/Interface names | GKG `search_definitions` (P1) | `CreateSavingHandler`, `IAgwClient` |
| Constants/Enums | `grep_search` (P2) | `TranType.OPEN_SAVING`, `ERR_001` |
| API endpoints/methods | SC `codebase_search` (P3, only if P1/P2 miss) | `initTransaction()` |

Mismatch → fix artifact immediately → log in change notes.

---

## STEP 8 — Summary

`openspec status --change "<name>"`

```
═══════════════════════════════════════
OPENSPEC COMPLETE
═══════════════════════════════════════
Change:    <name>
Profile:   <flow> | <factory> | <type>
Artifacts: ✓ proposal ✓ specs ✓ design ✓ tasks ✓ srs
Context:   static N / dynamic N / rules N / skills N
FRs:       <covered>/<total> (missing: <list or none>)
Warnings:  <partial artifacts or none>
═══════════════════════════════════════
Ready: /wf_openspec_apply <name>

💡 If design decisions need revisiting, run `/wf_brainstorm_openspec <name>` then re-run `/wf_openspec <name>`.
```