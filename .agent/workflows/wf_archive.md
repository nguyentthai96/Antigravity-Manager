---
description: Archive a completed change in the experimental workflow
---

Archive a completed change in the experimental workflow.

**Pipeline position**: ... → `/wf_integ_test` ⟂ `/wf_client_doc` → **`/wf_archive`** → `/wf_gen_change_doc`

## EXECUTION CONTRACT

Steps 1→7 sequential. Each step with user prompt: wait for reply before proceeding.

**DO NOT:** auto-select a change, skip task completion check, modify source code, delete archive without confirmation.

**Input**: Optionally specify a change name after `/wf_archive` (e.g., `/wf_archive add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Steps**

1. **If no change name provided, prompt for selection**

   Run `openspec list --json` to get available changes. Ask the user directly to select and STOP. Wait for reply.

   Show only active changes (not already archived).
   Include the schema used for each change if available.

   **IMPORTANT**: Do NOT guess or auto-select a change. Always let the user choose.

2. **Check artifact completion status**

   Run `openspec status --change "<name>" --json` to check artifact completion.

   Parse the JSON to understand:
   - `schemaName`: The workflow being used
   - `artifacts`: List of artifacts with their status (`done` or other)

   **If any artifacts are not `done`:**
   - Display warning listing incomplete artifacts
   - Ask user for confirmation to continue in the response and STOP. Wait for reply.
   - Proceed if user confirms

3. **Check task completion status**

   Read the tasks file (typically `tasks.md`) to check for incomplete tasks.

   Count tasks marked with `- [ ]` (incomplete) vs `- [x]` (complete).

   **If incomplete tasks found:**
   - Display warning showing count of incomplete tasks
   - Ask user for confirmation to continue in the response and STOP. Wait for reply.
   - Proceed if user confirms

   **If no tasks file exists:** Proceed without task-related warning.

4. **Assess delta spec sync state**

   Check for delta specs at `openspec/changes/<name>/specs/`. If none exist, proceed without sync prompt.

   **If delta specs exist:**
   - Compare each delta spec with its corresponding main spec at `openspec/specs/<capability>/spec.md`
   - Determine what changes would be applied (adds, modifications, removals, renames)
   - Show a combined summary before prompting

   **Prompt options:**
   - If changes needed: "Sync now (recommended)", "Archive without syncing"
   - If already synced: "Archive now", "Sync anyway", "Cancel"

   If user chooses sync, use Task tool (subagent_type: "general-purpose", prompt: "Use Skill tool to invoke openspec-sync-specs for change '<name>'. Delta spec analysis: <include the analyzed delta spec summary>"). Proceed to archive regardless of choice.

5. **Perform the archive**

   > ⚠️ **Cross-platform:** Agent MUST detect OS from system metadata and use appropriate commands.

   Create the archive directory if it doesn't exist:

   **Linux/Mac (bash):**
   ```bash
   mkdir -p openspec/changes/archive
   ```

   **Windows (PowerShell):**
   ```powershell
   New-Item -ItemType Directory -Force -Path "openspec/changes/archive" | Out-Null
   ```

   Generate target name using current date: `YYYY-MM-DD-<change-name>`

   **Check if target already exists:**
   - If yes: Fail with error, suggest renaming existing archive or using different date
   - If no: Move the change directory to archive

   **Linux/Mac (bash):**
   ```bash
   mv openspec/changes/<name> openspec/changes/archive/YYYY-MM-DD-<name>
   ```

   **Windows (PowerShell):**
   ```powershell
   Move-Item -Path "openspec/changes/<name>" -Destination "openspec/changes/archive/YYYY-MM-DD-<name>"
   ```

6. **Display summary**

   Show archive completion summary including:
   - Change name
   - Schema that was used
   - Archive location
   - Spec sync status (synced / sync skipped / no delta specs)
   - Note about any warnings (incomplete artifacts/tasks)

**Output On Success**

```
## Archive Complete

**Change:** <change-name>
**Schema:** <schema-name>
**Archived to:** openspec/changes/archive/YYYY-MM-DD-<name>/
**Specs:** ✓ Synced to main specs

All artifacts complete. All tasks complete.
```

**Output On Success (No Delta Specs)**

```
## Archive Complete

**Change:** <change-name>
**Schema:** <schema-name>
**Archived to:** openspec/changes/archive/YYYY-MM-DD-<name>/
**Specs:** No delta specs

All artifacts complete. All tasks complete.
```

**Output On Success With Warnings**

```
## Archive Complete (with warnings)

**Change:** <change-name>
**Schema:** <schema-name>
**Archived to:** openspec/changes/archive/YYYY-MM-DD-<name>/
**Specs:** Sync skipped (user chose to skip)

**Warnings:**
- Archived with 2 incomplete artifacts
- Archived with 3 incomplete tasks
- Delta spec sync was skipped (user chose to skip)

Review the archive if this was not intentional.
```

**Output On Error (Archive Exists)**

```
## Archive Failed

**Change:** <change-name>
**Target:** openspec/changes/archive/YYYY-MM-DD-<name>/

Target archive directory already exists.

**Options:**
1. Rename the existing archive
2. Delete the existing archive if it's a duplicate
3. Wait until a different date to archive
```

**Guardrails**
- Always prompt for change selection if not provided
- Use artifact graph (openspec status --json) for completion checking
- Don't block archive on warnings - just inform and confirm
- Preserve .openspec.yaml when moving to archive (it moves with the directory)
- Show clear summary of what happened
- If sync is requested, use the Skill tool to invoke `openspec-sync-specs` (agent-driven)
- If delta specs exist, always run the sync assessment and show the combined summary before prompting

---

## 7. SocratiCode Sync (Post-Archive)

After archive is complete, ensure SocratiCode indexes reflect code changes.

**This step is AUTOMATIC — no user prompt needed.**

### 7a — Quick status check

Run `codebase_status` once:
- **FILE_WATCHER active** → source code auto-synced ✓, context artifacts auto-detected as stale on next search ✓ → **SKIP all further sync steps**
- **FILE_WATCHER inactive** → run `codebase_update` to sync source code

### 7b — Context re-index (only if knowledge changed AND watcher inactive)

If files in `base_knowledge/` were modified during this change AND FILE_WATCHER is inactive:
- Run `codebase_context_index`
- Otherwise: auto-detected on next `codebase_context_search` → skip

### 7c — Graph rebuild (conditional)

If ≥5 source files changed AND FILE_WATCHER is inactive → run `codebase_graph_build`
Otherwise → skip (graph rebuilds lazily on next `codebase_graph_query`)

### Console output:
```
SocratiCode Sync: {auto-synced ✓ | updated} | context={auto|refreshed} | graph={auto|rebuilt}
```
