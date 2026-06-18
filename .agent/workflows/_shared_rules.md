# Shared Workflow Rules

> Dùng chung cho các workflow trong pipeline. Import bằng reference: `> See: _shared_rules.md`

---

## CONTEXT PRIORITY (Base)

| # | Source | Override Rule |
|---|--------|--------------|
| 1 | `pre_openspec.md` | Source of truth — NEVER overridden |
| 1.5 | `brainstorm_notes.md` (if exists) | Design direction — supplements #1, NEVER overrides |
| 2 | Dynamic context (`context/generated/`) | Overrides static for: class names, packages, factories |
| 3 | Static knowledge (`base_knowledge/`) | General patterns |
| 4 | Skills | Utility references |

**Brainstorm rules:**
- `brainstorm_notes.md` is **OPTIONAL** — all workflows MUST work without it
- It SUPPLEMENTS `pre_openspec.md` (refines approach, adds design hints)
- `Selected Direction` → becomes architectural hint for design.md / tasks.md
- If brainstorm contradicts pre_openspec → **IGNORE brainstorm**, follow pre_openspec
- Downstream consumers: `wf_openspec` (Step 3d), `wf_openspec_apply` (Step 2a-bis)

**Safety:** Dynamic NEVER overrides flow type, FR meaning, feature type. If dynamic contradicts pre_openspec → IGNORE. If incomplete → fallback to static. If empty/malformed → skip file.

**Stale data:** If dynamic context `_Generated` date > 7 days old → WARN `"⚠️ Dynamic context may be stale"`. If > 30 days → treat as static priority (demote to level 3). Re-run `/wf_pre_openspec` recommended.

---

## ANTI-HALLUCINATION RULES  + UNCERTAINTY)

DO NOT invent APIs, DB tables, handlers, factories, or flow phases not in context.

| Missing Info | Action |
|-------------|--------|
| Flow / Factory / Base class | 🔴 BLOCK — ask user |
| Integration detail | 🟡 `N/A`, continue |
| Minor field | 🟢 `N/A`, continue |

ALL assumptions: `⚠️ Assumption: <what> — <reason>`

Missing → `// TODO: cần bổ sung`.

---

## TOOL PRIORITY (Hybrid Search)

> **Follows GEMINI.md Search Strategy.** Choose tool by query type, prefer cheaper tools first.
> Rule: Run exact tools (GKG + grep) in parallel first. Escalate to SC semantic ONLY if insufficient. Stop at first tier providing enough context.

| Priority | Tool | Best For | When Available |
|----------|------|----------|---------------|
| P1 | `GKG search_definitions` / `gitnexus context` | Symbol definitions, class lookup | GKG MCP / GitNexus MCP |
| P2 | `grep_search` | Exact text patterns, constants | Always |
| P3 | `gitnexus query` / `SC codebase_search` | Semantic search, process discovery | GitNexus / SocratiCode |
| P4 | `view_file` | Full file read (last resort) | Always |

---

## DIFF AWARENESS (MAINTENANCE / EXTEND)

If feature type = `MAINTENANCE` or `EXTEND`:
- MUST read existing spec files before generating (if any exist in archive or change dir)
- MUST preserve unchanged sections from previous version
- MUST highlight changes with `[CHANGED]`, `[NEW]`, `[REMOVED]` tags
- MUST NOT regenerate unchanged sections from scratch (risk of drift)
- If no previous version → generate as NEWBUILD

---