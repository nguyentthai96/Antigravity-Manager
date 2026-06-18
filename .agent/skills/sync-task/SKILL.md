---
name: sync-task
description: Sync documentation artifacts of a change/task with current source code. Detects drift and auto-corrects documents to match implementation.
compatibility: Requires openspec CLI.
metadata:
  author: anhdt8
  version: "1.0"
---

Synchronize a change's documentation with the current source code. Detects drift between docs and code, auto-corrects documentation, and handles archived changes.

> **Pipeline boundary:** This skill is for **post-implementation maintenance** — use it AFTER the pipeline
> (`/wf_openspec_apply`) completes, or anytime during ongoing development to detect doc-code drift.
> The pipeline's built-in sync (WF3 Step 5) handles in-pipeline task tracking.
> This skill handles deeper structural sync: API endpoints, service methods, DTOs, configs, error codes.

**Input**: Task name (kebab-case) or metadata ID (e.g., `VCBDIGIBIZ260323A1b2C`).

**Steps**

## 1. Locate the Change

Search for the change using the provided input (name or ID):

### 1a. Search Active Changes

Scan `openspec/changes/` (excluding `archive/`) for a match:
- Check directory name against the provided task name
- Read `.openspec.yaml` in each change directory and match against `name` or `id` field
  > **Fallback:** If `.openspec.yaml` not found → try `metadata.yaml` (legacy format)

### 1b. Search Archived Changes

If not found in active changes, scan `openspec/changes/archive/`:
- Read `.openspec.yaml` in each archived change directory (fallback: `metadata.yaml`)
- Match against `name` or `id` field

### 1c. Handle Results

- **Found in active changes**: Proceed to Step 2.
- **Found in archive**: Mark as `archived = true`, proceed to Step 2.
- **Not found**: Report error — "Change not found. Available changes:" and list all active + archived changes with their IDs and names. Stop.
- **Multiple matches**: List all matches and ask user to select.

Always announce: "Found change: `<name>` (ID: `<id>`) — Status: active / archived"

## 2. Read Change Documentation

Read all documentation artifacts in the change directory:

| Artifact | Feature Change | CR Change |
|----------|---------------|-----------|
| `proposal.md` or `cr-proposal.md` | [V] | [V] |
| `design.md` or `cr-design.md` | [V] | [V] |
| `tasks.md` or `cr-tasks.md` | [V] | [V] |
| `specs/*/spec.md` | [V] | — |
| `srs.md` | [V] | — |
| `current-code-logic.md` | — | [V] |
| `compare-logic.md` | — | [V] |
| `.openspec.yaml` (or `metadata.yaml`) | [V] | [V] |

Read whichever artifacts exist. Skip any that don't exist.

From the documentation, extract:
- API endpoints described (paths, methods, request/response schemas)
- Service methods and business logic described
- Entity/DTO structures described
- Config keys described
- Error codes described
- Processing flows described

## 3. Analyze Current Source Code

Based on the documentation scope (which services, endpoints, classes are mentioned), scan the current source code:

a. **Scan documented endpoints**: Find actual controller/route handler classes and verify path mappings match
   > Java: `@RestController`, `@Controller` | Python: `@app.route`, `class *View(APIView)` | TypeScript: `@Controller()`, `router.*`

b. **Scan documented services**: Find actual service implementations and verify method signatures and logic

c. **Scan documented models**: Find actual DTOs, entities, and verify field definitions

d. **Scan documented configs**: Find actual `getConfig()` calls and verify key names and defaults

e. **Scan documented error codes**: Find actual `Constants.ResCode` usage and verify codes

Build a comprehensive comparison of what docs say vs what code does.

## 4. Compare Documentation vs Source Code

For each documentable item, determine sync status:

```
┌─────────────────────────────────────────────────┐
│              SYNC STATUS REPORT                 │
├─────────────────────────────────────────────────┤
│ [V] In Sync:    [count] items                     │
│ [!]  Drifted:   [count] items                     │
│ [NEW] In Code Only: [count] items (undocumented)    │
│ [DEL]  In Docs Only: [count] items (code removed)   │
└─────────────────────────────────────────────────┘
```

Create a detailed comparison table:

| # | Item | Document | Doc Description | Code Reality | Status |
|---|------|----------|----------------|-------------|--------|
| 1 | `POST /v1/payment` | design.md | 3 request params | 5 request params | [!] Drifted |
| 2 | `PaymentService.init()` | design.md | Validates amount only | Validates amount + currency | [!] Drifted |
| 3 | Config `TIMEOUT` | design.md | Default 30s | Default 60s | [!] Drifted |
| 4 | `GET /v1/status` | — | Not documented | Exists in code | [NEW] Code Only |

## 5. Handle Sync Results

### 5a. If ALL IN SYNC (no drift)

```
[V] Documentation is in sync with source code.
No corrections needed.

Change: <name> (ID: <id>)
Items checked: N
Last sync: YYYY-MM-DDTHH:mm:ss
```

Update `.openspec.yaml` with `last-sync` (Step 7), then **STOP**.

### 5b. If DRIFT DETECTED

Display the sync report from Step 4, then proceed to Step 6.

### 5c. If Change is ARCHIVED and DRIFTED

a. **Move the change back to active**:
```bash
mv openspec/changes/archive/<archived-dir-name> openspec/changes/<original-name>
```

b. **Announce**: "Change was archived but documentation has drifted. Moved back to active changes for sync."

c. Proceed to Step 6.

## 6. Auto-Correct Documentation

For each drifted item, edit the relevant documentation file to match the current source code.

### 6a. Correction Process

For each drift:
1. Open the relevant document
2. Find the section describing the drifted item
3. Update the description to match current code
4. Add a warning marker:

```markdown
> [!WARNING]
> **Auto-corrected during sync (YYYY-MM-DD HH:mm:ss)**
> Original: [what the document previously stated]
> Corrected to: [what the code actually does]
```

### 6b. Handle "Code Only" Items (undocumented)

For items that exist in code but not in docs:
- Add a new section in the most appropriate document (design.md for architecture, specs for requirements)
- Mark as newly added:

```markdown
> [!NOTE]
> **Added during sync (YYYY-MM-DD HH:mm:ss)**
> This item was found in source code but not previously documented.
```

### 6c. Handle "Docs Only" Items (code removed)

For items that exist in docs but not in code:
- Do NOT remove from documentation
- Add a notice:

```markdown
> [!CAUTION]
> **Code not found during sync (YYYY-MM-DD HH:mm:ss)**
> This item is documented but was not found in current source code.
> It may have been removed, renamed, or moved. Please verify.
```

### 6d. Generate Sync Report

After all corrections:

```
## Sync Report: <change-name>

**Date**: YYYY-MM-DD HH:mm:ss
**Change ID**: <id>

### Corrections Applied

| # | Document | Section | Change Type | Before | After |
|---|----------|---------|------------|--------|-------|
| 1 | design.md | POST /v1/payment | [!] Updated | 3 params | 5 params |
| 2 | design.md | PaymentService.init() | [!] Updated | Validates amount | Validates amount + currency |
| 3 | specs/payment/spec.md | — | [NEW] Added | Not documented | GET /v1/status documented |

### Summary
- Documents corrected: N
- Items updated: M
- Items added: K
- Items flagged (code not found): J
```

## 7. Update `.openspec.yaml`

Add or update the `last-sync` field in the change's `.openspec.yaml` (or `metadata.yaml` if legacy):

```yaml
last-sync: "YYYY-MM-DDTHH:mm:ss+07:00"
```

Use the exact current timestamp at the moment of completion.

## 8. Post-Sync Actions

### If Change Was Active
```
[V] Sync complete. Documentation updated to match source code.
Review the corrected documents in: openspec/changes/<name>/
```

### If Change Was Unarchived
```
[!] Sync complete. Documentation updated to match source code.

This change was moved back from archive because documentation had drifted.
Location: openspec/changes/<name>/

**Action Required**: Review the corrections, then run `/wf_archive` to re-archive.
```

---

**Guardrails**
- **MUST** only edit documentation to match code — NEVER edit code to match documentation
- **MUST** add `> [!WARNING]` markers on every corrected section with timestamp and before/after details
- **MUST** search BOTH active and archived changes when locating a task
- **MUST** unarchive a change before modifying its documents (do NOT edit files in `archive/` directory)
- **MUST** update `last-sync` in `.openspec.yaml` after every sync operation (even if no drift found)
- **MUST** prompt user to re-archive after syncing a previously archived change
- Do NOT remove documented items that are missing from code — only add a `[!CAUTION]` notice
- Do NOT auto-archive after sync — always let the user decide
- Do NOT guess code behavior — scan actual source files
- If a document references code that cannot be found at all (e.g., entire service deleted), flag it and ask user for guidance
