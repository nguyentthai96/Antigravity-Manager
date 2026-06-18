---
name: wf-pre-openspec
description: Pre-process URD or user idea into structured specification AND scan source code for dynamic context. Supports both URD (Confluence/file) and direct idea input via source_type parameter. Output identical for downstream pipeline compatibility.
metadata:
  author: Agent
  version: "5.0"
---

## EXECUTION CONTRACT (MANDATORY)

You MUST execute ALL steps in order: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11.

**Phase A (Steps 1–7):** Input Analysis → `pre_openspec.md`
**Phase B (Steps 8–11):** Code Scan → `openspec/context/generated/<name>/`

For EACH step:
- MUST produce required output
- MUST validate output against step's VALIDATION rules
- If validation fails → retry (max 2 times)
- If still fails → STOP and report error

DO NOT:
- Skip archive scan (Step 1c)
- Skip classification (Step 1d)
- Skip Confluence comments if source is Confluence (URD mode)
- Proceed without reading URD content or understanding user's idea
- Invent modules not found in `features.md` (`base_knowledge/structures/propose/features.md`)
  > **Fallback if `features.md` missing:** See `scan_rules.md` → CANDIDATE SERVICE DETECTION → Fallback rule (line 71): `list_dir` project root → detect service/module directories by name.
- Invent API names or bank systems
- Skip Phase B (code scan)
- Scan outside candidate service directories

If cannot find evidence for a claim → write `N/A` or `⚠️ Assumption`.

---

## SOURCE TYPE (determines Step 1a behavior)

| `source_type` | When | Step 1a Behavior |
|---------------|------|-----------------|
| **`urd`** (default) | User provides Confluence URL, page ID, file, or raw text | Read URD content → extract requirements |
| **`idea`** | User describes feature directly in chat | Clarify idea → build Synthetic Requirement Summary |

> The calling workflow sets `source_type`. If not specified, default to `urd`.
> ALL steps after 1a are SHARED — only the input collection differs.

---

## EVIDENCE REQUIREMENT (APPLIES TO ALL STEPS)

**Phase A (Input):**
- For EACH classification, integration, or module reference:
  - MUST include: keyword match, module match, archive path, or class found
  - If no evidence → mark as: `⚠️ Assumption`

**Phase B (Code Scan):**
- For EVERY detected class, pattern, or convention:
  - MUST include **real class name** + **file path**
  - If not found → mark as `NOT DETECTED`
  - DO NOT guess. DO NOT use examples from training data.

---

## ANTI-HALLUCINATION RULES

> **Single source:** `.agent/common-rules/scan_rules.md` — loaded in Phase B.
> Core rule: DO NOT invent APIs, modules, patterns. Missing → `N/A`. Not found → `NOT DETECTED`.
> Additional for idea mode: DO NOT fabricate requirements user did not express or confirm.

---

## INPUT

### IF `source_type = urd` (default):

| Source Type | How to Identify | How to Collect |
|-------------|----------------|----------------|
| **Confluence page** | URL containing `atlassian.net/wiki` or page ID | `confluence_get_page(page_id)` + `confluence_get_comments(page_id)` |
| **Plain text / notes** | User pastes text or provides file path | Read from message or file system |
| **External URL** | Non-Confluence URL | `read_url_content(url)` |

### IF `source_type = idea`:

| Source Type | How to Identify | How to Collect |
|-------------|----------------|----------------|
| **Direct idea** | User describes feature/requirement in chat | Read from user message |
| **Idea + file** | User references a local requirement doc | Read file from filesystem |
| **Short prompt** | User gives a feature name or brief phrase | Ask clarifying questions |

> **KEY DIFFERENCE:** In `idea` mode, no URD/Confluence required.
> Agent acts as co-creator, asking clarifying questions to build requirements.

## OUTPUT

| Property | Value |
|----------|-------|
| **File 1** | `openspec/changes/<feature-name>/pre_openspec.md` |
| **Dir** | `openspec/context/generated/<feature-name>/` |
| **Language** | Vietnamese (except IDs and tech terms) |

**Context files (Phase B output):**

| # | File | Content |
|---|------|---------|
| 1 | `base_class_map.md` | Base class hierarchy (controller, handler, factory, client) |
| 2 | `service_structure.md` | Package structure, naming conventions |
| 3 | `integration_map.md` | External integrations (protocol, client, request/response) |
| 4 | `dto_pattern.md` | DTO patterns (request, response, filter, annotations) |
| 5 | `error_pattern.md` | Exception classes, error codes, error response formats |

---

# ═══════════════════════════════════════
# PHASE A — INPUT ANALYSIS (Steps 1–7)
# ═══════════════════════════════════════

## STEP 1 — Identify, Clarify, Classify, and Scaffold

### 1a. Collect Input (BRANCHES by source_type)

---

#### IF `source_type = urd`:

| Source | Action |
|--------|--------|
| Confluence | Extract page ID → `confluence_get_page(page_id)` + `confluence_get_comments(page_id)` |
| File / text | Read from file system or user message |
| URL | `read_url_content(url)` |

**CONFLUENCE RULE (MANDATORY):**
If source is Confluence:
- MUST read page content AND comments
- If comments fail → retry up to 2 times
- If still fails → continue with `⚠️ WARNING: comments unavailable` flag
- Log warning in output header

If URD content (page body) cannot be retrieved → STOP and ask user for alternative source.

---

#### IF `source_type = idea`:

Read the user's input and evaluate its completeness.

**SUFFICIENCY CHECK — Evaluate if the idea covers these 5 dimensions:**

| # | Dimension | Question to ask if MISSING |
|---|-----------|---------------------------|
| 1 | **Actors** | "Ai sẽ sử dụng tính năng này? (user, admin, system...)" |
| 2 | **Core Action** | "Hành động chính của tính năng là gì? Mô tả flow từng bước." |
| 3 | **Auth/OTP** | "Tính năng có cần xác thực OTP/biometric không?" |
| 4 | **External System** | "Có cần tích hợp hệ thống bên ngoài nào? (ngân hàng, NAPAS, VBG...)" |
| 5 | **Constraints** | "Có ràng buộc đặc biệt nào? (hạn mức, thời gian, quyền hạn...)" |

**CLARIFICATION RULES:**
- Evaluate ALL 5 dimensions against user's input
- If a dimension is clearly answered → SKIP that question
- If a dimension is missing or ambiguous → ASK (one at a time)
- Maximum 5 clarifying questions total
- Ask ONE QUESTION AT A TIME — wait for user response before next question
- If ALL 5 dimensions are sufficiently covered → SKIP clarification entirely
- After each answer, re-evaluate remaining gaps before asking next question

**SUFFICIENCY FORMULA:**
```
sufficient_count = count of dimensions clearly covered by user input
IF sufficient_count >= 4 → SKIP clarification (proceed directly)
IF sufficient_count < 4 → ASK missing dimensions (one at a time)
```

After clarification completes, synthesize all user input into a **Synthetic Requirement Summary**:
```markdown
## Synthetic Requirement Summary
- **Feature**: <short name>
- **Actors**: <list>
- **Core Flow**: <step-by-step description>
- **Auth Required**: Yes/No — <method>
- **External Systems**: <list or None>
- **Constraints**: <list or None>
- **Source**: User idea (conversation)
```

---

### 1b. Derive Feature Name

Extract from URD Use Case Name (urd mode) or user's idea (idea mode) → convert to kebab-case.
Example: "Mở tài khoản HKD" → `mo-tai-khoan-hkd`

### 1c. Scan Archive (MANDATORY — never skip)

1. `list_dir` on `openspec/changes/archive/`
2. Search for directory containing derived feature name
3. If match found:
   - Read `pre_openspec.md` from archive (PRIMARY BASE)
   - Read `tasks.md` if exists
   - Read `design.md` if exists
   - Note archive path in output header
4. If no match → proceed as fresh (FULL mode)

**Rules:**
- Archive = validated decisions → DO NOT discard unless new input explicitly contradicts
- Only read `pre_openspec.md`, `tasks.md`, `design.md` — NOT spec files

### 1d. Classify Feature Type (STRICT)

> 📌 **DELEGATES TO**: `.agent/common-rules/change_type_classification.md` — **Read Steps 0a-0d ONLY**
> Execute classification logic (Q0-Q3 matrix + code evidence verification).
> **STOP reading at `## Step 1`** — Steps 1a-1e (DTO Overlap Strategy) are for apply phase, loaded by WF2/WF3 when needed.

**Input for classification:**
- `feature_name` (from Step 1b)
- `candidate_services` (from `base_knowledge/structures/propose/features.md` keyword match)
- `archive_classification` (from Step 1c, if exists)

**Output required:** `classification` (MAINTENANCE|EXTEND|NEWBUILD) + `classification_evidence`

**Phase-specific guard:** Classification at pre-openspec is PRELIMINARY — may be refined at apply phase if new evidence found.

### 1e. Classify Transaction Flow

Read `knowledge_transaction_flow.md` section "Phân biệt từ URD / Mockup UI":

| Input Signal | Flow Type |
|-------------|-----------|
| Màn xác nhận thông tin + OTP, chuyển tiền, thanh toán | **Financial** (Init → AuthMethod → Confirm) |
| OTP trực tiếp, thay đổi cài đặt, cập nhật thông tin | **Non-Financial** (Init → Confirm) |
| Không cần OTP, hành động đơn giản | **Command** (single step) |
| Chỉ hiển thị/tra cứu data | **Query** (single step) |

### 1f. Create Change Directory

```bash
openspec new change "<feature-name>"
```
If already exists → skip, continue.

### STEP 1 OUTPUT REQUIREMENTS

| Variable | Constraint |
|----------|-----------|
| `feature_name` | NOT empty, kebab-case |
| `source_type` | `urd` / `idea` |
| `archive_found` | `true` / `false` |
| `archive_path` | valid path or `N/A` |
| `classification` | `MAINTENANCE` / `EXTEND` / `NEWBUILD` |
| `transaction_flow` | `Financial` / `Non-Financial` / `Command` / `Query` |
| `classification_evidence` | keyword + module + file (NOT empty) |
| `clarification_skipped` | `true` / `false` (idea mode only) |

### STEP 1 VALIDATION
- [ ] `feature_name` NOT empty, kebab-case
- [ ] `classification` is exactly `MAINTENANCE`, `EXTEND`, or `NEWBUILD`
- [ ] `transaction_flow` is one of 4 values
- [ ] `classification_evidence` contains ≥1 real evidence
- [ ] If `source_type = urd` AND Confluence → comments were read
- [ ] If `source_type = idea` → Synthetic Requirement Summary is complete
- [ ] Archive scan was performed

---

## STEP 2 — Extract Raw Data (NO Interpretation)

Parse input content and extract WITHOUT modifying meaning:

a. **Feature name** → kebab-case
b. **Actors** → ALL participants
c. **Raw requirements** → sentence-level from:
   - URD mode: basic flow, alternative flows, exception flows, business rules
   - Idea mode: core flow steps, constraints, auth requirements, edge cases user mentioned
d. **Integrations** → ALL mentioned external systems

**Rules:**
- DO NOT infer missing logic
- DO NOT rewrite original meaning
- If source mentioned it → include it
- If source didn't mention it → DO NOT assume
- URD mode: Preserve strikethrough (`~~text~~`) → deprecated

### STEP 2 VALIDATION
- [ ] ≥1 actor identified
- [ ] ≥1 raw requirement extracted (idea mode) or ≥3 (urd mode)
- [ ] URD mode: Strikethrough items preserved as-is
- [ ] Idea mode: No requirements invented beyond user's statements

---

## STEP 3 — Normalize into Structured Requirements

Transform to: `Actor → Action → Object → Condition → Result`
Convert to Vietnamese: `"Hệ thống phải <action> khi <condition>."`

a. Assign IDs: `FR-001`, `FR-002`, ...
b. Vietnamese title (3–6 words)
c. Split compound → multiple FRs
d. Extract validation rules into FR body
e. Tag: `[URD]` or `[IDEA]` or `[ENRICHED]`

**TAGGING RULES:**
- `[URD]` — Requirement directly from URD document (urd mode)
- `[IDEA]` — Requirement directly from user's idea/conversation (idea mode)
- `[ENRICHED]` — Requirement added by agent for domain completeness (both modes)

### STEP 3 VALIDATION
- [ ] Every FR has unique ID
- [ ] Every FR has Vietnamese title (3–6 words)
- [ ] Every FR has testable action
- [ ] Every FR tagged `[URD]`/`[IDEA]` or `[ENRICHED]`
- [ ] No compound FRs

---

## STEP 4 — Deduplicate and Consolidate

a. Merge duplicates → 1 FR
b. Normalize wording
c. Detect conflicts → keep latest, add to Issues

### STEP 4 VALIDATION
- [ ] No duplicate FRs
- [ ] Conflicts documented in Issues (not silently removed)

---

## STEP 5 — Enrich with Domain Requirements

**ENRICHMENT LIMIT: `min(5, ceil(FR_count × 0.20))`**

Example: 30 FRs → 20% = 6, min(5, 6) = 5. 15 FRs → 20% = 3, min(5, 3) = 3.

| Priority | Enrichment | Tag |
|----------|-----------|-----|
| 1 | Idempotency cho create/update | `[ENRICHED]` |
| 2 | Full transaction logging | `[ENRICHED]` |
| 3 | Timeout handling cho external API | `[ENRICHED]` |
| 4 | Retry mechanism cho failed calls | `[ENRICHED]` |
| 5 | Request authentication / encryption | `[ENRICHED]` |

### STEP 5 VALIDATION
- [ ] Enriched FRs ≤ calculated limit
- [ ] All tagged `[ENRICHED]`
- [ ] No existing FR overridden

---

## STEP 6 — Score Quality with Evidence-Based Deductions

### 6a. Quality Scoring (4 × 25 = 100)

| Tiêu chí | Max |
|----------|-----|
| Rõ ràng (Clarity) | 25 |
| Đầy đủ (Completeness) | 25 |
| Nhất quán (Consistency) | 25 |
| Kiểm thử được (Testability) | 25 |

**SCORING RULE:** Every deduction MUST reference specific `FR-XXX` + quote exact source phrase (URD text or user idea).

- Score < 70 → populate Open Questions
- Score < 50 → ask user BEFORE continuing

### 6b. Issue Detection

| Type | Detect |
|------|--------|
| Conflict | Same concept, different values |
| Missing | No error handling / edge case |
| Ambiguity | Vague words without criteria |
| Risk | Banking gaps: no idempotency/logging/retry |
| Incomplete | Source didn't specify critical detail |

### STEP 6 VALIDATION
- [ ] Every deduction references real FR-XXX
- [ ] Score is 0–100
- [ ] Issues have severity emoji (🔴🟡🟢)

---

## STEP 7 — Write pre_openspec.md

### MODE SELECTION

| Archive? | Input changed? | Mode |
|----------|---------------|------|
| No | — | **FULL** |
| Yes | Yes | **DELTA** |
| Yes | No | **REUSE** |

---

### Template — Load from skill

**Read template**: `.agent/skills/wf-pre-openspec/pre_openspec_template.md`

**Fill rules:**
- **Sections 1–10**: STRICT — fill all sections using data from Steps 1–6. Do NOT modify section names or order — downstream workflows depend on `STRUCTURED_MARKER`.
- **Section 11 (Transaction Flow Detail)**: SEMI-STRICT — fill based on Step 1e transaction flow classification.
- **Section 12 (Traceability Matrix)**: SEMI-STRICT — map FR → URD → Spec → Class.
- **Section 13 (Agent Notes)**: FREE-FORM — agent tự tổng hợp insights, observations, related features, integration notes, suggested approach. Phần này không bị ràng buộc template — agent bổ sung bất kỳ thông tin hữu ích nào.

**Mode behavior:**
- **FULL MODE**: Fill template hoàn toàn từ data Steps 1–6
- **DELTA MODE**: Follow 2-phase diff algorithm — chỉ update sections thay đổi so với archive
    **2-PHASE DIFF (MANDATORY):**
    **Phase 1:** Extract FR list from archive + new URD
    **Phase 2:** Compare deterministically:
    ```
    For EACH new_FR: if NOT in archive → NEW; if differs → MODIFIED
    For EACH archive_FR: if NOT in new → REMOVED
    ```

- **REUSE MODE**: Copy archived + add previous version header + update Section 13 (Agent Notes) nếu có insight mới

**Source-type specific overrides (idea mode only):**
- Use `[IDEA]` tag instead of `[URD]` for user-sourced FRs
- Add `> **Source**: User Idea (no URD)` in header
- FR count label: `Idea: <x>` instead of `URD: <x>`

---

### STEP 7 VALIDATION
- [ ] File exists at `openspec/changes/<name>/pre_openspec.md`
- [ ] Contains `> **Type**:` header
- [ ] If `source_type = idea` → contains `> **Source**: User Idea` header
- [ ] Contains `## 10. DETECTED SCOPE` with candidate services
- [ ] All FRs have unique IDs and tags
- [ ] Enriched FRs ≤ 5
- [ ] Quality deductions reference real FR-IDs
- [ ] All Vietnamese (except IDs/tech terms)

---

# ═══════════════════════════════════════
# PHASE B — CODE SCAN (Steps 8–11)
# ═══════════════════════════════════════

> Phase B runs IMMEDIATELY after Phase A writes `pre_openspec.md`.
> Uses the DETECTED SCOPE (Section 10, marked with `STRUCTURED_MARKER`) to scan ONLY relevant code.
> This replaces the separate `/wf_openspec_explore` workflow.

## STEP 8–10 — Code Scan, Context Files, Confidence

> **MANDATORY:** Follow ALL rules in `.agent/common-rules/scan_rules.md`
>
> This includes:
> - Anti-hallucination rules
> - Evidence requirement
> - Candidate Service Detection
> - Scope Enforcement

Execute Steps 8→9→10 exactly as defined in the shared rules file.
---

## STEP 8 — Scan Code Patterns

> **Tool Strategy:**
> - **SC available** → use `codebase_search()` with `languageFilter` for semantic discovery, then `grep_search` for exact pattern matching.
> - **GKG available** → use `gitnexus query` for process discovery, `gitnexus context` for symbol deep-dive.
> - **Neither available** → execute all scans using `grep_search` as defined below (GREP_FALLBACK mode).
> - All modes: `grep_search` results are the **ground truth**. SC/GKG results are supplementary evidence.

Execute these atomic scans within candidate service scope:

> **📄 SCAN PATTERNS**: Read `.agent/skills/wf-pre-openspec/scan_patterns.md` → Steps 8a–8e for all scan patterns, grep patterns per language, and output formats.

### STEP 8 VALIDATION

- [ ] Every listed class has real file path
- [ ] No invented/example class names
- [ ] `NOT DETECTED` used where nothing found
- [ ] All scans within scope (candidate services only)

---

## STEP 9 — Generate Context Files

Write to: `openspec/context/generated/<feature-name>/`

**OUTPUT RULE:** ONLY include detected items. NO predefined examples. `NOT DETECTED` for empty sections.

> **📄 TEMPLATES**: Read `.agent/skills/wf-pre-openspec/scan_patterns.md` → Step 9 for all 5 context file templates (`base_class_map.md`, `service_structure.md`, `integration_map.md`, `dto_pattern.md`, `error_pattern.md`).

### STEP 9 VALIDATION

- [ ] ALL 5 files created
- [ ] Every class has file path
- [ ] No hardcoded/example values
- [ ] `NOT DETECTED` for empty sections
- [ ] No `{placeholder}` text

---

## STEP 10 — Calculate Confidence + Change Impact Map

### 10a. Confidence Score (WEIGHTED FORMULA)

```
Confidence = (Service match × 0.30)
           + (Base classes × 0.25)
           + (Integration × 0.20)
           + (DTO pattern × 0.15)
           + (Error handling × 0.10)
```

Each signal scores 0 (NOT DETECTED) or 1 (DETECTED).

| Signal                    | Weight   | Detected? | Weighted Score |
|---------------------------|----------|-----------|----------------|
| Service directories exist | 0.30     | Yes/No    |                |
| Base classes found        | 0.25     | Yes/No    |                |
| Integration detected      | 0.20     | Yes/No    |                |
| DTO pattern found         | 0.15     | Yes/No    |                |
| Error handling found      | 0.10     | Yes/No    |                |
| **Total**                 | **1.00** |           | **X.XX**       |

If < 0.40 → WARN: `"⚠️ Insufficient code signal — context may be incomplete"`
If < 0.20 AND classification ≠ NEWBUILD → STOP: `"❌ Cannot proceed — no meaningful code detected. Verify candidate services."`
If < 0.20 AND classification = NEWBUILD → WARN: `"⚠️ NEWBUILD — no existing code expected. Context files may be mostly NOT DETECTED. Proceeding with infrastructure-only scan."`

### 10b. Change Impact Map (MAINTENANCE / EXTEND)

If classification = `MAINTENANCE` or `EXTEND`:

For EACH FR in `pre_openspec.md`:

- Map FR → affected classes found in Phase B scan
- Map FR → affected APIs (from integration_map.md)
- Tag each FR: `[MODIFY]` (existing class changes) / `[ADD]` (new class needed) / `[REUSE]` (no changes)

**OUTPUT (append to summary):**

```
Change Impact:
  FR-001 → [MODIFY] <ClassName> (<path>) → <API endpoint>
  FR-002 → [MODIFY] <ClassName> (<path>) → internal only
  FR-003 → [ADD] NEW (no existing class)
  FR-004 → [REUSE] <ClassName> (<path>) → no changes needed
```

If classification = `NEWBUILD` → skip this sub-step.

### STEP 10 VALIDATION

- [ ] Confidence calculated with weighted formula
- [ ] Warning/STOP shown at threshold
- [ ] If MAINTENANCE → Change Impact Map generated

---

## STEP 11 — Final Summary

Output EXACTLY:

```
═══════════════════════════════════════
PRE-OPENSPEC + EXPLORE COMPLETE
═══════════════════════════════════════

PHASE A — Input Analysis:
  Feature:        <name>
  File:           openspec/changes/<name>/pre_openspec.md
  Mode:           FULL / DELTA / REUSE
  Source:         <URD (Confluence/file/URL) / User Idea (no URD)>
  Type:           MAINTENANCE / EXTEND / NEWBUILD
  Evidence:       <keyword> → <module> → <file>
  Flow:           <Financial / Non-Financial / Command / Query>
  FRs:            <total> (URD/Idea: <x>, Enriched: <y>)
  Issues:         <total> (🔴 <a>, 🟡 <b>)
  Quality Score:  <score>/100
  Clarification:  <N/A (urd) / skipped / N questions asked (idea)>

PHASE B — Code Scan:
  Services:       <list>
  Context files:  5
  Base classes:   <count>
  Integrations:   <count>
  DTO patterns:   <count>
  Error patterns: <count>
  Confidence:     <score>/1.0
  Missing:        <list NOT DETECTED>

═══════════════════════════════════════

⏸️ REVIEW CHECKPOINT:
Review pre_openspec.md before proceeding.
When ready: /wf_openspec <feature-name>

💡 Optionally: /wf_brainstorm_openspec <name> if deeper thinking needed before apply.
```

> **Human Review**: User SHOULD review `pre_openspec.md` (FRs, Issues, Quality Score)
> before running `/wf_openspec`. This is optional but recommended for quality.
> If user wishes to skip review → proceed directly.

---

## GUARDRAILS

**Phase A:**
- DO NOT generate code
- DO NOT invent business logic not in source input
- DO NOT remove original intent
- DO NOT silently discard conflicts
- Strikethrough = deprecated
- Classification MUST use `grep_search` evidence
- Quality deductions MUST reference real FR-ID + source text
- Enriched FRs ≤ 5
- URD mode: Strikethrough = deprecated
- URD mode: If Confluence → MUST read comments
- Idea mode: Ask clarifying questions ONE AT A TIME
- Idea mode: Skip clarification if idea covers ≥4 of 5 dimensions
- Idea mode: Tag FRs as `[IDEA]` (not `[URD]`)
- Idea mode: DO NOT fabricate requirements user did not express

**Phase B:**
- ONLY scan candidate service directories
- MUST include file paths for ALL detected patterns
- MUST use `NOT DETECTED` for missing (not blank, not assumed)
- MUST NOT include template/example values as real output
- If `grep_search` returns 0 → `NOT DETECTED`
- If candidate services don't exist → STOP and ask user

**Both Phases:**
- If Confluence → MUST read comments
- If `openspec new change` fails (exists) → skip, continue
- DELTA mode: 2-phase diff algorithm
- URD mode + DELTA: read Confluence comments
