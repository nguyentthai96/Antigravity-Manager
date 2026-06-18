---
name: openspec-orchestrator
description: "Meta-orchestrator for OpenSpec pipeline — routes work through brainstorming, risk assessment, design review, and apply mode selection. Integrates design-orchestration, skill-orchestration, and multi-agent optimization."
---

# OpenSpec Pipeline Orchestrator (Meta-Skill)

## Purpose

Ensure that the **OpenSpec pipeline** flows correctly through all phases:
**Explore → Analyze → Design → Gate → Implement → Archive**

This skill does NOT generate artifacts or write code.
It **controls the flow between workflows and skills**.

---

## Operating Model

This is a **routing, enforcement, and optimization skill**.

It decides:
- which workflow/skill must run next
- whether risk escalation is required
- which execution mode to use for apply phase
- whether TDD enforcement is active

---

## Pipeline Overview

```
[Required]              [Optional]                 [Required]         [Gate]              [Required]           [Parallel]
/wf_pre_openspec → /wf_brainstorm_openspec → /wf_openspec → Risk Assessment → /wf_openspec_apply → /wf_integ_test ⟂ /wf_client_doc → /wf_archive
```

---

## Controlled Workflows & Skills

| Component | Role | Phase |
|-----------|------|-------|
| `/wf_pre_openspec` | URD analysis + code scan | Phase 0 |
| `brainstorming` | Explore ideas, propose approaches | Phase 1 (optional) |
| `/wf_openspec` | Generate artifacts (proposal, design, tasks) | Phase 2 |
| `design-orchestration` (embedded) | Risk classification | Phase 2.5 |
| `multi-agent-brainstorming` | Design review (if HIGH risk) | Phase 2.5 |
| `/wf_openspec_apply` | Implement with subagent + TDD | Phase 3 |
| `subagent-driven-development` | Task execution strategy | Phase 3 |
| `test-driven-development` | Code quality enforcement | Phase 3 |
| `executing-plans` | Alternative inline execution | Phase 3 |
| `/wf_integ_test` | Integration testing | Phase 3.5 |
| `/wf_client_doc` | Client integration docs | Phase 3.5 |
| `/wf_archive` | Archive completed change | Phase 4 |
| `/wf_gen_change_doc` | Generate deployment DB scripts | Phase 5 (post-archive) |

---

## Agent ↔ Pipeline Integration

| Pipeline Phase | Primary Agent | Invoked When |
|---------------|---------------|-------------|
| Phase 0 (URD Analysis) | — (workflow only) | Always |
| Phase 1 (Brainstorm) | — (workflow only) | Optional |
| Phase 2 (Design) | `spring-boot-engineer` | Architecture validation, base class verification |
| Phase 2.5 (Risk Gate) | `code-reviewer` + `security-engineer` | HIGH risk changes |
| Phase 3 (Apply) | `spring-boot-engineer` | Implementation, code generation |
| Phase 3.5 (Test) | — (workflow only) | Integration test execution |
| Phase 3.5 (Docs) | `docs-engineer` | Client integration documentation |
| Phase 3.5 (Diagrams) | `mermaid-expert` | Sequence diagrams in client docs |
| Phase 4 (Archive) | — (workflow only) | Always |
| Phase 5 (DB Scripts) | — (workflow only) | Post-archive deployment prep |

---

## Entry Conditions & Routing

### Task Complexity Guard (from skill-orchestrator)

Before invoking this meta-skill, evaluate:

1. **Is the task simple/contained?** (bug fix, config change, typo)
   → Solve directly. Do NOT use this pipeline.

2. **Is it a new feature or significant change?**
   → Use this pipeline.

3. **Is the URD/requirement available?**
   → Start at `/wf_pre_openspec`

4. **Is the idea vague or needs exploration?**
   → Start at `/wf_pre_openspec` anyway, then use `/wf_brainstorm_openspec` to deep-think

---

## Phase 0 — URD Analysis (Required)

### When to Invoke

- Always — this is the entry point for any feature pipeline
- User has URD, requirement doc, or Confluence page

### How to Invoke

Run `/wf_pre_openspec <source>` which:
1. Analyzes URD document
2. Scans source code for related patterns
3. Outputs `pre_openspec.md` + dynamic context

### Exit Conditions

- `pre_openspec.md` generated → proceed to Phase 1 or Phase 2

---

## Phase 1 — Brainstorming (Optional)

### When to Invoke

- Requirements need deeper thinking after URD analysis
- Multiple approaches possible, need to decide before design
- User explicitly requests brainstorming
- User has free-form idea (no URD) — Mode 2

### How to Invoke

Run `/wf_brainstorm_openspec <name>` which:
1. Loads `pre_openspec.md` context (Mode 1) or accepts idea (Mode 2)
2. Explores approaches with user (The Stance)
3. Investigates codebase (GitNexus-enhanced)
4. Captures decisions
5. Outputs `brainstorm_notes.md` into change directory

### Exit Conditions

- User approves a design direction → proceed to `/wf_openspec`

### Skip Conditions

- URD is clear and straightforward
- User invokes `/wf_openspec` directly after `/wf_pre_openspec`

---

## Phase 2 — OpenSpec Artifact Generation

Run `/wf_openspec <name>` which generates:
- `proposal.md` — scope & approach
- `design.md` — architecture & patterns
- `specs/<capability>/spec.md` — per-capability specs
- `tasks.md` — implementation tasks (enriched, self-contained)
- `srs.md` — software requirements specification
- `impact_analysis.md` — reuse & blast radius (if applicable)

### Design Risk Assessment Gate (automatic after Step 8)

Evaluates `design.md` + `pre_openspec.md` → routes to Phase 3 or multi-agent review.

### Risk Classification

Evaluate `design.md` + `pre_openspec.md` against these factors:

| Factor | LOW | MODERATE | HIGH |
|--------|-----|----------|------|
| Flow type | Query / Command | Non-Financial | Financial |
| FR count | ≤ 5 | 6–10 | > 10 |
| Feature type | NEWBUILD | — | MAINTENANCE |
| External integrations | ≤ 1 | 2 | > 2 |
| DB schema changes | None | Add columns | New tables / alter existing |
| Irreversibility | Low | Medium | High (data migration, API contract) |

### Scoring

- Count factors at each level
- Overall risk = **highest level with ≥ 2 factors**
- Exception: Financial flow → minimum MODERATE regardless

### Routing

- **LOW risk** → Proceed directly to `/wf_openspec_apply`
- **MODERATE risk** → Recommend multi-agent review. User can skip with acknowledgment.
- **HIGH risk** → REQUIRE multi-agent review. Cannot skip.

### Multi-Agent Review (if triggered)

When invoked:

**Require:**
- Completed `design.md`
- Locked feature profile
- `pre_openspec.md` for FR traceability

**Allow ONLY:**
- Critique of design decisions
- Revision of implementation approach
- Decision resolution

**Do NOT allow:**
- New ideation or scope expansion
- Reopening problem definition
- Changes to FR list

**Disposition:**
- `APPROVED` → Proceed to `/wf_openspec_apply`
- `REVISE` → Return to `/wf_openspec` Step 7 (regenerate affected artifacts)
- `REJECT` → Return to brainstorming or `/wf_pre_openspec`

---

## Phase 3 — Apply Mode Selection

Before executing `/wf_openspec_apply`, determine execution mode:

### Decision Logic

```
Has tasks.md?
  └─ No → STOP: run /wf_openspec first
  └─ Yes → Evaluate task independence
       └─ Tasks mostly independent (1-2 files each)?
            └─ Yes → Subagent-Driven (default)
            └─ No (tightly coupled, shared state) → Inline Execution
```

### Subagent-Driven Mode (Default)

- Dispatch fresh subagent per task
- Each subagent receives: task spec + profile constraints + dynamic context
- Each subagent follows TDD: RED → GREEN → REFACTOR
- Two-stage review: spec compliance → code quality
- Uses `subagent-driven-development` skill prompts

### Inline Execution Mode (Fallback)

- Execute tasks sequentially in current session
- Still follows TDD per task
- Uses `executing-plans` skill approach
- Better for tightly-coupled tasks that share state

### TDD Enforcement

**Mandatory for:**
- Handler classes (extends Base*Handler)
- Service/business logic classes
- Factory classes (extends Base*DataFactory)
- Utility classes with business logic

**Exempt (configurable):**
- DTOs / Request / Response classes
- Entity classes (pure data)
- Configuration files
- Database migration scripts
- Static constants

---

## Enforcement Rules

- Do NOT allow `/wf_openspec_apply` without completed design gate
- Do NOT skip TDD for mandatory categories
- Do NOT merge design and implementation phases
- Do NOT skip spec compliance review in subagent mode
- Do NOT allow silent risk de-escalation (log all risk decisions)

---

## Performance Tracking (from multi-agent-optimize)

After each pipeline completion, log metrics:

```
Pipeline Metrics:
- Total duration: <time>
- Phase breakdown: brainstorm <t> | pre_openspec <t> | openspec <t> | gate <t> | apply <t>
- Tasks: <planned> + <added> = <total>
- TDD cycles: <RED-GREEN-REFACTOR count>
- Review iterations: <spec-review count> + <quality-review count>
- Risk level: <LOW|MODERATE|HIGH>
- Subagent dispatches: <count>
```

---

## Exit Conditions

This meta-skill exits ONLY when:
- The next step is explicitly identified, AND
- All required prior steps are complete

Possible exits:
- "Start URD analysis: `/wf_pre_openspec <source>`"
- "Start brainstorming: `/wf_brainstorm_openspec <name>`"
- "Generate artifacts: `/wf_openspec <name>`"
- "Design gate: running risk assessment..."
- "Run multi-agent review"
- "Proceed to implementation: `/wf_openspec_apply <name>`"
- "Run integration tests: `/wf_integ_test <name>`"
- "Generate client docs: `/wf_client_doc <name>`"
- "Return to design for revision"
- "Archive: `/wf_archive <name>`"

---

## Design Philosophy

This skill exists to:
- **Slow down** design decisions (risk gate)
- **Speed up** implementation (subagent parallelism + TDD confidence)
- **Prevent** costly mistakes (multi-agent review for HIGH risk)
- **Ensure** code quality (TDD + two-stage review)

Good systems fail early (in design review).
Bad systems fail in production (no review, no tests).

## When to Use

Invoke this skill when:
- Starting any feature that goes through the OpenSpec pipeline
- Unsure which pipeline phase to start from
- Need to assess risk before implementation
- Want to choose between subagent vs inline execution

## Limitations

- Do NOT use for simple bug fixes or config changes
- Do NOT use as a substitute for domain expertise (ask the user)
- Multi-agent review requires context about the codebase — ensure knowledge base is up to date
