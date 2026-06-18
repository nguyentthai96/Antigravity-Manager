---
description: Brainstorm & deep thinking after URD analysis — explore ideas, analyze requirements, decide approach, or propose a new change from an idea
---

**[WORKFLOW]** Brainstorm → deep thinking → design direction → feed into `/wf_openspec`.

- **Primary skill**: `brainstorming` — collaborative design exploration
- **Support skills**: `openspec-orchestrator` (pipeline routing), `openspec-explore` (thinking stance), `gitnexus-exploring` (codebase investigation)

**Pipeline position**: `/wf_pre_openspec` → **`/wf_brainstorm_openspec`** → `/wf_openspec` → `/wf_openspec_apply` → `/wf_integ_test` ⟂ `/wf_client_doc` → `/wf_archive`

> ℹ️ This workflow is **OPTIONAL** when URD/requirements are already crystal clear.
> Use it when: the idea is vague, multiple approaches need evaluation, or you want deeper analysis after URD.
> It can also accept **free-form ideas** (like opsx-propose) — not just URD-derived changes.

---

**Input modes**:
- **Mode 1** (after `/wf_pre_openspec`): Change name → reads existing `pre_openspec.md` for context
- **Mode 2** (standalone): Ý tưởng/description tự do → creates new change directory, brainstorms from scratch

---

## The Stance

> Adapted from opsx-explore: This is a **thinking partner**, not a script runner.

- **Curious, not prescriptive** — Ask questions that emerge naturally, don't follow a rigid script
- **Open threads, not interrogations** — Surface multiple interesting directions and let the user follow what resonates
- **Visual** — Use ASCII diagrams liberally when they'd help clarify thinking
- **Adaptive** — Follow interesting threads, pivot when new information emerges
- **Patient** — Don't rush to conclusions, let the shape of the problem emerge
- **Grounded** — Explore the actual codebase when relevant, don't just theorize

---

## Steps

### 1. Load context & resolve input mode

**1a. Check existing changes:**
```bash
openspec list --json
```

**1b. Resolve input mode:**

- **Mode 1** (change name provided, `pre_openspec.md` exists):
  - Read `openspec/changes/<name>/pre_openspec.md` → extract FR list, scope, type
  - Read `openspec/changes/<name>/brainstorm_notes.md` if exists (resume previous brainstorm)

- **Mode 2** (description/idea provided, no `pre_openspec.md`):
  - Derive kebab-case name from description (e.g., "add user authentication" → `add-user-auth`)
  - Create change directory: `openspec new change "<name>"`
  - Note: No `pre_openspec.md` available — brainstorm from user's idea directly

**1c. OpenSpec awareness** (from opsx-explore):
- If active changes exist → reference them naturally in conversation
- If user mentions a change → read its artifacts for context

---

### 2. Invoke brainstorming skill (adapted for OpenSpec)

Read and follow `.agent/skills/brainstorming/SKILL.md` with these adaptations:

---

### 3. Explore & Analyze

> No fixed sequence — adapt based on what the user brings.

**2a. Understanding Phase:**
- Ask clarifying questions **ONE AT A TIME**
- Focus on: purpose, constraints, affected modules, success criteria
- Prefer multiple choice when possible

**2b. Approach Exploration:**
- Propose 2-3 approaches with trade-offs
- Consider project-specific factors:
  - Flow type implications (Query vs Command vs Non-Financial vs Financial)
  - Factory type implications (BaseCrudDataFactory vs BaseClientDataFactory)
  - NEWBUILD vs MAINTENANCE classification
  - Integration points with existing services

**2c. Codebase Investigation (optional — GitNexus-enhanced):**

When the discussion benefits from grounding in actual code:

| Need | Tool | Example |
|------|------|---------|
| Find related execution flows | `gitnexus query "{topic}"` | Discover how similar features are implemented |
| 360-degree view of symbol | `gitnexus context "{symbol}"` | See callers, callees, processes |
| Architecture overview | `READ gitnexus://repo/{name}/clusters` | Functional areas with cohesion scores |
| Full execution trace | `READ gitnexus://repo/{name}/process/{name}` | Step-by-step flow trace |
| Find file/class by name | `grep_search` | Simple text matching |

> **Prerequisite**: Index must be up-to-date. Check: `READ gitnexus://repo/{name}/context`.
> If stale → `npx gitnexus analyze` (see skill `gitnexus-cli`).
> If GitNexus unavailable → fallback to grep/SC (không block brainstorm).

**2d. Visualization:**
```
Use ASCII diagrams liberally:

    ┌────────┐         ┌────────┐
    │ State  │────────▶│ State  │
    │   A    │         │   B    │
    └────────┘         └────────┘

System diagrams, state machines, data flows,
architecture sketches, dependency graphs,
comparison tables — all welcome.
```

**2e. Surface risks and unknowns:**
- Identify what could go wrong
- Find gaps in understanding
- Suggest spikes or investigations

---

### 4. Decision Capture

> From opsx-explore: Offer to capture decisions — don't auto-capture.

When insights crystallize during conversation:

| Insight Type | Where to Capture |
|-------------|-----------------|
| New requirement discovered | `specs/<capability>/spec.md` |
| Requirement changed | `specs/<capability>/spec.md` |
| Design decision made | `design.md` |
| Scope changed | `proposal.md` |
| New work identified | `tasks.md` |
| Assumption invalidated | Relevant artifact |

**Example offers:**
- "That's a design decision. Want me to capture it in design.md?"
- "This is a new requirement. Add it to specs?"
- "This changes scope. Update the proposal?"

**The user decides** — Offer and move on. Don't pressure. Don't auto-capture.

---

### 5. Create brainstorm notes

If a change directory exists:
```bash
# Write to existing change directory
openspec/changes/<name>/brainstorm_notes.md
```

**Brainstorm notes format:**
```markdown
---
type: brainstorm_notes
change: <name>
date: YYYY-MM-DD
selected_direction: "<approach name>"
pre_flow: "<flow type or TBD>"
pre_feature_type: "<NEWBUILD/MAINTENANCE/EXTEND or TBD>"
status: complete
---

# Brainstorm Notes: <topic>

## Date
YYYY-MM-DD

## Context
<What prompted this exploration — URD summary or user idea>

## Questions Asked & Answers
- Q1: ... → A: ...
- Q2: ... → A: ...

## Approaches Considered
### Approach 1: <name>
- Pros: ...
- Cons: ...

### Approach 2: <name>
- Pros: ...
- Cons: ...

## Selected Direction
<Chosen approach with reasoning>

## Pre-classifications (preliminary)
- Feature type: NEWBUILD / MAINTENANCE (if determinable)
- Flow type: Query / Command / Non-Financial / Financial (if determinable)
- Affected modules: <list>

## GitNexus Findings (if explored)
- Related processes: <list>
- Key symbols: <list>
- Architecture insights: <summary>

## Open Questions for Design Phase
- [OPEN] <Question that needs `/wf_openspec` to answer>
- [RESOLVED] <Question already answered during brainstorm>

## Open Questions for URD Analysis
- <Questions that need formal URD to answer>
```

---

### 6. Transition

```
═══════════════════════════════════════
BRAINSTORM COMPLETE
═══════════════════════════════════════
Change:     <name>
Direction:  <selected approach summary>
Pre-class:  <flow> | <type> (preliminary)
Open Qs:    <count>
GitNexus:   <explored / not explored>
═══════════════════════════════════════
Ready: /wf_openspec <name>
```

If Mode 2 was used (no `pre_openspec.md`):
```
💡 Note: No URD analysis was performed.
   Consider running /wf_pre_openspec <URD-source> for formal analysis,
   or proceed directly to /wf_openspec <name> if brainstorm is sufficient.
```

---

## What You Don't Have To Do

> From opsx-explore: These are liberties, not obligations.

- Follow the steps above in strict order
- Ask the same questions every time
- Produce a specific artifact
- Reach a conclusion
- Stay on topic if a tangent is valuable
- Be brief (this is thinking time)

---

## Guardrails

- **Do NOT write implementation code** during brainstorming
- **Do NOT generate OpenSpec artifacts** (proposal, design, tasks) — that's `/wf_openspec`'s job
- **Do NOT skip user approval** of design direction
- **Do NOT auto-capture** — offer to save insights, don't just do it
- Brainstorm notes are **SUPPLEMENTARY** — they do NOT replace URD or `pre_openspec.md`
- If user provides clear URD + clear requirements → suggest skipping brainstorm, go directly to `/wf_openspec`
- **Do visualize** — a good diagram is worth many paragraphs
- **Do explore the codebase** — ground discussions in reality (GitNexus or grep)
- **Do question assumptions** — including the user's and your own
