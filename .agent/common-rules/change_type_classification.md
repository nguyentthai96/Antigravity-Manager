# Change Type Classification Rules

> **Purpose:** Mandatory gate BEFORE creating any new file. Determines whether a change
> is MAINTENANCE (modify logic), EXTEND (add capability), or NEWBUILD (create new).
> Applies to: propose + apply phases of ANY project.

---

## Step 0 — Classification (MANDATORY, BEFORE any file creation)

### 0a. Classify Change Type

For **every** change/feature, answer these 4 questions in order:

| Question                                                                                                               | Yes →            | No →               |
|------------------------------------------------------------------------------------------------------------------------|------------------|--------------------|
| Q0: Is this a **modification of existing logic** (bug fix, rule change, config update, minor tweak)?                   | **MAINTENANCE**  | Continue to Q1     |
| Q1: Does a module/feature with the **same domain** already exist?                                                      | EXTEND candidate | NEWBUILD candidate |
| Q2: Does the new feature share **>50% of the same flow** (init→validate→execute→respond)?                              | EXTEND candidate | NEWBUILD candidate |
| Q3: Can the existing handler/executor/controller accommodate the new logic with **conditional branching** (flag/enum)? | EXTEND           | NEWBUILD           |

**Decision Matrix:**

| Q0 | Q1 | Q2 | Q3 | Classification       | Strategy                                                           |
|----|----|----|----|----------------------|--------------------------------------------------------------------|
| Y  | -  | -  | -  | **MAINTENANCE**      | Modify existing files directly, NO new files                       |
| N  | Y  | Y  | Y  | **EXTEND**           | Add fields + conditional logic to existing components              |
| N  | Y  | Y  | N  | **EXTEND-SPLIT**     | Extend DTOs + shared logic, new handler only if flow diverges >50% |
| N  | Y  | N  | -  | **NEWBUILD-PARTIAL** | Reuse infrastructure (controller, AGW), new handler+executor       |
| N  | N  | -  | -  | **NEWBUILD**         | Fully new components                                               |

> 🔴 **HARD RULES:**
> - Q0=Yes → MUST classify as MAINTENANCE. No new files created.
> - Q1=Yes AND Q2=Yes → MUST classify as EXTEND. Creating duplicate handler/executor/DTO is a **violation**.

### Distinguishing MAINTENANCE vs EXTEND

| Aspect      | MAINTENANCE                       | EXTEND                                            |
|-------------|-----------------------------------|---------------------------------------------------|
| **Scope**   | Change behavior of existing logic | Add NEW capability/variant                        |
| **Files**   | Modify existing only              | Modify existing + possibly new entity/service/job |
| **DTO**     | No new fields (or trivial rename) | New optional fields + discriminator flag          |
| **Handler** | Fix/adjust existing logic         | Add conditional branch for new variant            |
| **Example** | Sửa validate min amount 100k→200k | Thêm "gửi góp định kỳ" vào module tiết kiệm       |
| **Example** | Fix logic tính ngày đáo hạn       | Thêm loại sản phẩm mới vào luồng mở sổ            |
| **Example** | Đổi message lỗi                   | Thêm job chạy scheduled cho tính năng có sẵn      |

### 0b. Verify Classification via Code Evidence (SC+GKG+grep)

**Do NOT rely on intuition or handler name matching.** Verify with tools:

```
Phase 1 — Domain Discovery (parallel — low cost):
  1. grep_search("{feature domain}", Includes=["{PROJECT_EXT}"])       → Find existing module files
  2. GKG search_definitions(["{DomainHandler}", "{DomainExecutor}"])  → Find named symbols
  3. SC  codebase_search("{domain} handler executor", languageFilter="{PROJECT_LANG}", limit=5)  → Semantic match

> **{PROJECT_EXT} / {PROJECT_LANG}**: See `scan_rules.md` → LANGUAGE DETECTION section for resolution logic.

Phase 2 — Flow Comparison (if Phase 1 found handlers):
  4. GKG read_definitions([{names: ["{HandlerClass}"], file_path: "{path}"}])
     → Read actual handler methods: preHandle, aroundHandle, buildTransactionModel
  5. Compare handler flow steps with new feature requirements:
     - Count shared steps vs unique steps
     - Calculate overlap: shared_steps / total_unique_steps
     - If >50% → EXTEND candidate

Phase 3 — Accommodation Check (if Phase 2 shows >50% overlap):
  6. Evaluate: can existing handler accommodate via conditional branching?
     - Check if handler already has flags/enums for variants
     - Check if DTO can accept optional fields
     - If yes → EXTEND confirmed
     - If no (e.g., fundamentally different state machine) → EXTEND-SPLIT or NEWBUILD

Phase 4 — Classify each new requirement:
  7. For each requirement:
     - [REUSE]   — Exact same logic, already exists
     - [EXTEND]  — Same logic + new fields/conditions
     - [NEW]     — Truly unique logic, no existing equivalent
```

> 🔴 **MANDATORY:** Phase 2 (GKG read_definitions) MUST be executed if Phase 1 finds handlers.
> Do NOT skip flow comparison — it is the primary evidence for EXTEND vs NEWBUILD.

### 0c. EXTEND Strategy — What to do

When classified as EXTEND:

| Component Type           | Strategy                                                      | Example                                                           |
|--------------------------|---------------------------------------------------------------|-------------------------------------------------------------------|
| **Request/Response DTO** | Add optional fields to existing DTO                           | Add `recurringAmount`, `isRecurring` to `InitCreateSavingRequest` |
| **Handler**              | Add conditional branch in `preHandle`/`buildTransactionModel` | `if (request.getIsRecurring()) { validateRecurringFields(); }`    |
| **Executor**             | Add conditional branch in `onConfirmTransaction`              | `if (metadata.getIsRecurring()) { saveSchedule(); }`              |
| **Controller**           | Reuse existing endpoint (same DTO)                            | No new endpoint needed                                            |
| **Enum/Constant**        | Add values to existing enum                                   | Add `RECURRING_OPEN` to `TypeSaving`                              |
| **TranType**             | Reuse existing type OR add new only if service code differs   | Depends on Bank API contract                                      |
| **AGW Client**           | Add optional fields to existing request/response              | Don't create new client methods                                   |

**When to create NEW components (even in EXTEND):**

| Component Type        | Create new when...                                       |
|-----------------------|----------------------------------------------------------|
| **Entity/Repository** | New database table is needed (e.g., schedule table)      |
| **Service**           | Truly new business logic domain (e.g., date calculation) |
| **Job Module**        | New scheduled process (e.g., recurring deposit job)      |
| **Enum**              | New status domain (e.g., RecurringDepositStatus)         |
| **Config Constants**  | New configuration namespace                              |

### 0d. NEWBUILD Strategy — What to do

When classified as NEWBUILD (Q1=No):

- Create all components fresh
- Follow `reuse_rules_compact.md` Step 4 for project layout discovery
- Still check for shared infrastructure to reuse (base classes, utilities)

---

<!-- ═══════════════════════════════════════════════════════════════════════════ -->
<!-- PHASE BOUNDARY: Above = propose phase (WF1 Step 1d / WF2 Step 6c)        -->
<!-- Below = apply phase ONLY (WF3 / openspec-apply-change)                    -->
<!-- WF1 MUST STOP reading here. Loading below wastes ~2.8K tokens.           -->
<!-- ═══════════════════════════════════════════════════════════════════════════ -->

## Step 1 — DTO/Model Overlap Strategy (MANDATORY for EXTEND — apply phase only)

> ⚠️ **Code examples below are Java-specific illustrations.**
> For other languages, apply the SAME STRATEGY with language-appropriate patterns:
> Python: `@dataclass` inheritance / Pydantic `BaseModel` extension
> TypeScript: `interface` extension / `class extends`
> Kotlin: `data class` copy + additional fields

> When the change type is EXTEND, determine HOW to handle DTOs/Models.

### 1a. Measure Overlap

```
Compare fields:
  existing_fields = count fields in ExistingDTO
  new_fields      = count NEW fields needed (not in existing)
  total_fields    = existing_fields + new_fields
  overlap         = existing_fields / total_fields

Example:
  InitCreateSavingRequest:   14 fields
  Recurring needs:           14 existing + 4 new = 18 total
  Overlap = 14/18 = 78% → but new fields are all optional
  → Effective overlap = >80% (Strategy 1)
```

### 1b. Choose Strategy

| Overlap    | Strategy                | Action                                     | When to use                                  |
|------------|-------------------------|--------------------------------------------|----------------------------------------------|
| **>80%**   | **Add optional fields** | Add nullable fields + flag to existing DTO | Same flow, minor additions                   |
| **50-80%** | **Inheritance**         | Create child DTO extending existing        | Need type-safety, different validation rules |
| **<50%**   | **Separate DTO**        | Create independent DTO                     | Fundamentally different model                |

### 1c. Strategy Details

**Strategy 1: Add optional fields (>80% overlap)**

```java
// BEFORE: existing DTO
public class CreateSavingRequest extends BaseRequest {
    private String productCode;    // shared
    private BigDecimal amount;     // shared
    // ...existing fields
}

// AFTER: add optional fields (backward-compatible)
public class CreateSavingRequest extends BaseRequest {
    private String productCode;    // shared
    private BigDecimal amount;     // shared
    // ...existing fields unchanged
    
    // New optional fields — null when not applicable
    private BigDecimal recurringAmount;   // only for recurring
    private LocalDate startDate;          // only for recurring
    private Boolean isRecurring;          // discriminator flag
}
```

**Key rules:**

- New fields MUST be nullable (no @NotNull)
- Add discriminator flag (e.g., `isRecurring`, `isBatch`, `type`)
- Existing consumers are NOT affected (new fields default to null)
- Handler uses flag to conditionally validate new fields

**Strategy 2: Inheritance (50-80% overlap)**

```java
// Base — keep existing unchanged
public class CreateSavingRequest extends BaseRequest {
    // shared fields
}

// Child — extend with new fields
public class CreateRecurringSavingRequest extends CreateSavingRequest {
    private BigDecimal recurringAmount;
    private LocalDate startDate;
}
```

**Key rules:**

- Use when child needs @NotNull on new fields (type-safety)
- Use when new fields change validation contract
- Existing handler can accept base type and instanceof-check
- OR create new handler only if flow diverges >50%

**Strategy 3: Separate DTO (<50% overlap)**

- Fully independent DTO — different endpoint, handler, executor
- This is effectively NEWBUILD for the DTO layer
- Still check if shared base class can be used

### 1d. Decision Flowchart

```
Start → Count overlapping fields
  ├── >80%? → Add optional fields + flag to existing DTO
  │           └── Handler: if (flag) { validateNewFields(); }
  ├── 50-80%? → Does child need stricter validation on new fields?
  │   ├── Yes → Inheritance (child extends parent)
  │   └── No  → Add optional fields (treat as >80%)
  └── <50%? → Separate DTO (NEWBUILD for DTO layer)
```

### 1e. Verification (after choosing strategy)

- [ ] Existing API consumers are NOT broken by DTO changes
- [ ] New fields are nullable (Strategy 1) or in child class (Strategy 2)
- [ ] Discriminator flag exists for handler conditional logic
- [ ] No duplicate DTO with >80% field overlap exists in codebase

---

## Anti-Patterns (MUST AVOID)

### Anti-Pattern 1: "Clone and Specialize"

❌ Copy existing handler → rename → modify slightly
✅ Extend existing handler with conditional logic

### Anti-Pattern 2: "Parallel Endpoint"

❌ Create `/recurring/init` when `/init-create-saving` can handle both
✅ Add optional fields to existing DTO, reuse existing endpoint

### Anti-Pattern 3: "Duplicate DTO"

❌ Create `InitCreateRecurringSavingRequest` when `InitCreateSavingRequest` is 90% same
✅ Add optional fields to existing request DTO

### Anti-Pattern 4: "Separate Executor for Variant"

❌ Create `CreateRecurringSavingExecutor` when `CreateSavingExecutor` can branch
✅ Add conditional block in existing executor: `if (isRecurring) { ... }`

---

## Checklist (run BEFORE creating any [NEW] file)

- [ ] Confirmed change type classification (MAINTENANCE/EXTEND/NEWBUILD)
- [ ] Searched for existing module in same domain
- [ ] Compared existing flow with new requirements (>50% overlap = EXTEND)
- [ ] Verified no existing DTO covers >80% of required fields
- [ ] Verified no existing handler/executor can accommodate with conditional logic
- [ ] Documented classification decision with search evidence

> **If any checkbox fails → STOP and re-evaluate before proceeding.**

---

## Integration Points

This rule is referenced by:

- `openspec-apply-change/SKILL.md` — Step 6 (read classification once, before task loop)
- `reuse_rules_compact.md` — Step 0 (trigger detection)
