# Skills Index — BIDC API Service

> Phân loại skills theo mức độ sử dụng trong pipeline. Giúp agent quyết định khi nào load skill nào.

## Tier 1 — Core Pipeline (dùng hàng ngày)

| Skill | Phase | Mục đích |
|-------|-------|----------|
| `openspec-orchestrator` | Meta | Meta-orchestrator điều phối toàn pipeline |
| `wf-pre-openspec` | Phase 0 | URD analysis + code scan |
| `brainstorming` | Phase 1 | Explore ideas, brainstorm approaches |
| `openspec-apply-change` | Phase 3 | Implement tasks from OpenSpec change |
| `openspec-archive-change` | Phase 4 | Archive completed change |
| `openspec-propose` | Phase 2 | Quick-propose change (simplified) |
| `openspec-explore` | Phase 1 | Free-form exploration + thinking partner |
| `sync-task` | Phase 3 | Task synchronization |

## Tier 2 — Supporting Skills (dùng khi pipeline cần)

| Skill | Pipeline Phase | Mục đích |
|-------|---------------|----------|
| `writing-plans` | Phase 2-3 | Create implementation plans |
| `subagent-driven-development` | Phase 3 | Multi-agent task execution |
| `test-driven-development` | Phase 3 | TDD enforcement |
| `dispatching-parallel-agents` | Phase 3 | Parallel agent dispatch |
| `multi-agent-brainstorming` | Phase 2.5 | Multi-perspective design review |
| `design-orchestration` | Phase 2.5 | Risk classification + gate |
| `confluence-reader` | Phase 0 | URD source reading |
| `using-superpowers` | Phase 1 | Superpowers brainstorming |
| `mermaid-diagram-agent` | Phase 3.5 | Sequence diagrams cho client docs |
| `mermaid-diagram-enterprise` | Phase 2 | Architecture diagrams |
| `codebase-exploration` | Phase 0 | Codebase navigation (SocratiCode core) |
| `codebase-management` | Phase 0 | Codebase lifecycle management |
| `wf-codebase-discovery` | Phase 0 | Discovery support — templates, standards rules |

## Tier 3 — Domain Knowledge (load khi cần context cụ thể)

| Skill | Domain | Mục đích |
|-------|--------|----------|
| `spring-boot` | Backend | Spring Boot patterns |
| `java-pro` | Language | Java best practices |
| `jpa-patterns` | Data | JPA/Hibernate patterns |
| `database-architect` | Data | DB schema design |
| `database-design` | Data | Normalization, indexing |
| `domain-driven-design` | Architecture | DDD patterns |
| `design-patterns` | Architecture | GoF + enterprise patterns |
| `architecture-patterns` | Architecture | High-level arch patterns |
| `logging-patterns` | Observability | Structured logging |
| `clean-code` | Quality | Code quality standards |
| `code-refactoring-refactor-clean` | Quality | Refactoring techniques |

## Tier 4 — Specialized (hiếm dùng, load on-demand)

| Skill | When | Mục đích |
|-------|------|----------|
| `architect-review` | Major changes | Architecture review checklist |
| `senior-architect` | Complex decisions | Senior architect toolkit |
| `software-architecture` | System design | Software architecture concepts |
| `architecture` | Quick reference | Architecture basics |
| `api-design-principles` | New APIs | REST API design |
| `api-documenter` | API docs | Swagger/OpenAPI generation |
| `docs-architect` | Documentation | Technical doc structure |
| `design-md` | Design system | DESIGN.md generation |
| `code-reviewer` | PR review | Code review patterns |
| `writing-skills` | Content | Technical writing |
| `ui-review` | Frontend | UI review (mobile apps) |

## Tier 5 — GitNexus (knowledge graph tools)

| Skill | Tool | Mục đích |
|-------|------|----------|
| `gitnexus-guide` | Reference | GitNexus overall guide |
| `gitnexus-cli` | CLI | Command-line operations |
| `gitnexus-exploring` | Exploration | Codebase exploration via graph |
| `gitnexus-impact-analysis` | Impact | Blast radius analysis |
| `gitnexus-debugging` | Debug | Root cause tracing |
| `gitnexus-refactoring` | Refactor | Safe refactoring with graph |

## Tier 6 — Potentially Unused (review periodically)

| Skill | Reason |
|-------|--------|
| `graphql-architect` | Project uses REST/gRPC, not GraphQL |
| `kotlin-tooling-agp9-migration` | Very specific Kotlin migration tool |
| `kotlin-tooling-cocoapods-spm-migration` | iOS-specific, not relevant |
| `kotlin-tooling-java-to-kotlin` | May be useful if migrating to Kotlin |
| `kotlin-backend-jpa-entity-mapping` | Kotlin-specific JPA, project is Java |

---

## Load Strategy

```
Agent lên pipeline → Load Tier 1 (orchestrator chọn workflow)
→ Workflow bắt đầu → Load Tier 2 (skills liên quan đến phase)
→ Task cụ thể → Load Tier 3 (domain knowledge on-demand)
→ Đặc biệt → Load Tier 4/5 (khi cần specialized context)
```
