# Code Scan Rules (Phase B)

> Shared rules for ALL pre-openspec/explore skills (wf-pre-openspec, wf-idea-to-spec, etc.)
> Mỗi skill PHẢI tham chiếu file này cho Phase B thay vì inline lại.

---

## ANTI-HALLUCINATION RULES

DO NOT:

- Invent API names not in source material
- Invent bank systems not mentioned
- Invent modules not found in `features.md` (`base_knowledge/structures/propose/features.md`)
- Assume integrations not explicitly referenced
- Generate fake quality deductions
- Assume CQRS if no CommandHandler/QueryHandler found in code
- Assume Base* pattern if no Base* classes found in scan
- Assume AGW gateway if no IAgw*Client interfaces found
- Include predefined patterns in output unless detected in code

If information is missing → write `N/A`.
If pattern not found in code → write `NOT DETECTED`.

---

## EVIDENCE REQUIREMENT

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

## LANGUAGE DETECTION (Multi-Project Support)

> All search filters in this document use `{PROJECT_LANG}` and `{PROJECT_EXT}` variables.
> These MUST be resolved BEFORE Phase B scanning.

**Detection order:**

1. Read `artifact_context.yml` → `environment.language` (if set)
2. If not set → scan project root:
    - `pom.xml` or `build.gradle` → `language=java`, `ext=*.java`
    - `build.gradle.kts` → `language=kotlin`, `ext=*.kt`
    - `package.json` → `language=typescript`, `ext=*.ts,*.js`
    - `requirements.txt` or `pyproject.toml` → `language=python`, `ext=*.py`
3. Default: `language=java`, `ext=*.java` — ⚠️ WARN:
   `"Language auto-detected as Java (no config found). Verify or set environment.language in artifact_context.yml."`

**Usage in scan commands:**

- `grep_search("{pattern}", Includes=["{PROJECT_EXT}"])` instead of `Includes=["*.java"]`
- `codebase_search("{query}", languageFilter="{PROJECT_LANG}")` instead of `languageFilter="java"`

---

## CANDIDATE SERVICE DETECTION RULES

Candidate Services in Section 10.3 are determined by these rules (in order):

| # | Rule                         | Method                                                                                                                            | Example                                        |
|---|------------------------------|-----------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------|
| 1 | **Domain keyword match**     | Extract keywords from source → `grep_search` in `features.md` (`base_knowledge/structures/propose/features.md`) → matched service | "chuyển tiền" → `transfer-service`             |
| 2 | **API path pattern**         | Detect API paths (e.g., `/api/v1/transfer`) → map to service                                                                      | `/api/v1/alias` → `alias-service`              |
| 3 | **Module naming convention** | Match feature name to `module/` directory names                                                                                   | `tat-toan-tiet-kiem` → `saving/module/closure` |
| 4 | **Archive reference**        | If archive found, use its `tasks.md` to identify services                                                                         | Archive tasks show files in `gateway-service`  |
| 5 | **Integration reference**    | If source mentions external system, include its gateway service                                                                   | "gửi Napas" → `client_gateway`                 |

If NO candidate service resolved → STOP and ask user:
> "Cannot detect candidate services. Please specify which service/module to scan."

**Fallback if `features.md` missing:** `list_dir` project root → detect service/module directories by name → match
keywords from source against directory names. If project uses monorepo → scan `settings.gradle` / `pom.xml` /
`package.json` workspaces for module list.

---

## SCOPE ENFORCEMENT (APPLIES TO ALL STEPS)

**ONLY scan directories belonging to resolved Candidate Services:**

> **Project-agnostic paths:** DO NOT hardcode `src/main/java`. Detect source root from project structure.

| Project Type        | Source Root Pattern                                        |
|---------------------|------------------------------------------------------------|
| Maven/Gradle (Java) | `<service>/src/main/java/**`                               |
| Gradle (Kotlin)     | `<service>/src/main/kotlin/**`                             |
| Node.js             | `<service>/src/**`                                         |
| Python              | `<service>/**/*.py`                                        |
| Multi-module        | Detect from `pom.xml` / `build.gradle` / `settings.gradle` |

For EACH file scanned:

- MUST belong to candidate service directories
- If outside scope → IGNORE

If candidate services don't resolve to real directories → STOP and ask user.
