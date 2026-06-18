---
description: Pre-process URD into structured specification AND scan source code for dynamic context
---

**[WORKFLOW]** Orchestrate URD analysis + Code scan pipeline → `pre_openspec.md` + context files.

- **Primary skill**: `wf-pre-openspec` (with `source_type=urd`) — Phase A (URD analysis) + Phase B (code scan) in one pass
- **Support skill**: `confluence-reader` — reads Confluence content when input is Confluence source

**Pipeline position**: **`/wf_pre_openspec`** → `/wf_brainstorm_openspec` → `/wf_openspec` → `/wf_openspec_apply` → `/wf_integ_test` ⟂ `/wf_client_doc` → `/wf_archive`

> ℹ️ This workflow uses `wf-pre-openspec` skill with `source_type=urd`.
> Code scanning is now embedded in Phase B of the skill — no separate explore step needed.

**Input**: Argument after the command is the URD source (Confluence URL, page ID, file path, or raw text).

**Steps**

1. **Set source type**
   Set `source_type = urd` for the skill invocation.

2. **If input is a Confluence URL or Page ID**:
   Invoke the `confluence-reader` skill to read the page content (including sub-pages if any).
   Read and rigorously follow: `.agent/skills/confluence-reader/SKILL.md`
   > Comment reading + retry logic is handled by `wf-pre-openspec` skill Step 1a — do not duplicate here.

3. **Invoke wf-pre-openspec skill**
   Read and rigorously follow the instructions in `.agent/skills/wf-pre-openspec/SKILL.md`.

   The skill executes 2 phases:

   **Phase A — URD Analysis (Steps 1–7):**
   - Scaffold OpenSpec change directory via CLI
   - Classify feature type (MAINTENANCE/EXTEND/NEWBUILD) with SC+GKG+grep evidence
   - Classify transaction flow
   - Analyze URD → normalize FR-xxx + quality scoring + issue detection
   - Output: `openspec/changes/<feature-name>/pre_openspec.md`

   **Phase B — Code Scan (Steps 8–11) + GitNexus Enhancement:**
   - Read DETECTED SCOPE from pre_openspec.md Section 10
   - Scan ONLY candidate service code (scope-enforced)
   - Extract: base classes, package structure, integrations, DTOs, error patterns
   - Output: `openspec/context/generated/<feature-name>/` (5 context files)
   - Calculate detection confidence score

   **Step 8.5 — GitNexus Graph Search** (runs PARALLEL with Steps 8–9):
   - `gitnexus query "{feature keywords from URD}"` → discover related processes + symbols
   - Map processes → candidate services (cross-validate with keyword detection)
   - If GitNexus confirms a candidate found by grep → confidence +15%
   - Output: enriched detection results (merged into context files)

   **Step 9.5 — GitNexus Verification** (for each candidate handler):
   - `gitnexus context "{handler}"` → verify integration points, callers, callees
   - Cross-validate with grep-based detection

   > GitNexus runs in parallel with keyword/module scan — does NOT block.
   > If GitNexus index unavailable → fallback to pure grep/SC. No degradation.

   **Enforced patterns:**
   - Execution Contract — 11 steps sequential, no skip
   - Evidence Requirement — class + file path for every detection
   - Anti-Hallucination — no invented modules/APIs/patterns
   - Scope Enforcement — only scan candidate services (5 detection rules)
   - Candidate Detection — keyword match, API path, module naming, archive ref, integration ref
   - 2-Phase DELTA — deterministic diff for EXTEND
   - Enrichment Limit — max(5, top 20% of FR count) whichever smaller
   - Evidence-Based Scoring — deductions reference real FR-ID + URD text
   - Confidence Formula — weighted (service 30%, base 25%, integration 20%, DTO 15%, error 10%)
   - Traceability Matrix — FR → URD → Spec → Class mapping
   - Change Impact Map — FR → affected classes for MAINTENANCE/EXTEND (with [MODIFY]/[ADD]/[REUSE] tags)
   - Review Checkpoint — optional human review before /wf_openspec

When ready to proceed:
- Brainstorm/deep thinking: `/wf_brainstorm_openspec <feature-name>`
- Or directly generate artifacts: `/wf_openspec <feature-name>`