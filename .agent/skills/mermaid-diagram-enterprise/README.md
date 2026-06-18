# mermaid-diagram-enterprise

**Enterprise Mermaid Diagram Generator** — Professional-grade Mermaid diagram skill cho enterprise systems.

## Tổng quan

Skill này kết hợp khả năng toàn diện của **mermaid-expert** (global) với các template enterprise-specific để tạo diagram chuyên nghiệp cho mọi loại hệ thống phần mềm.

## Scope

- ✅ API Request Flow (REST, GraphQL, gRPC)
- ✅ Database ERD (Entity-Relationship)
- ✅ Deployment Architecture (K8s, Docker, Cloud)
- ✅ Microservices Communication
- ✅ Event-Driven Architecture (CQRS, Event Sourcing)
- ✅ Business Process Flowcharts
- ✅ State Machines (Order lifecycle, User states)
- ✅ CI/CD Pipeline Visualization
- ✅ Class Diagrams (Domain models, DTOs)
- ✅ Gantt Charts, User Journeys, Mind Maps

## Không dùng khi

- ❌ Cần diagram cho AI Agent Systems → dùng `mermaid-diagram-agent/`
- ❌ Cần chọn architecture pattern → dùng `architecture-patterns/`
- ❌ Cần tài liệu chi tiết (ebook, manual) → dùng `docs-architect/`

## Templates có sẵn (8 templates)

1. **API Request Flow** — Spring Boot: Controller → Service → Repository
2. **Database ERD** — Entity relationships với cardinality
3. **Deployment Architecture** — Kubernetes topology
4. **Microservices Communication** — Service mesh, REST, gRPC, Kafka
5. **Event-Driven Architecture** — CQRS, Command/Query separation
6. **Business Process Flow** — Approval chains, decision trees
7. **State Machine** — Order lifecycle, FSM
8. **CI/CD Pipeline** — Build → Test → Deploy stages

## Capabilities (merged from mermaid-expert)

- Tất cả Mermaid diagram types: flowchart, sequence, class, state, ERD, gantt, pie, gitGraph, journey, quadrant, timeline, mindmap
- Syntax Guard với self-repair protocol
- Light & Dark theme presets
- Professional classDef palette
- Quality Gate validation

## References

| File | Mô tả |
|------|-------|
| `references/syntax-guard.md` | Mermaid syntax pitfall guide, escaping rules, self-repair protocol |
| `references/real-example.md` | Worked example: Spring Boot Order Management → complete enterprise diagram package (quality benchmark) |

## Related Skills

| Skill | Quan hệ |
|-------|---------|
| `mermaid-diagram-agent/` | Agent Systems — cho LangGraph, AutoGen, CrewAI |
| `software-architecture/` | C4 Model, ADR templates |
| `docs-architect/` | Technical documentation orchestration |
| `design-orchestration/` | Meta-skill routing |

---

> **Tip:** Skill này phù hợp với mọi project enterprise, không giới hạn Spring Boot. Có thể dùng cho Node.js, Python, Go, .NET hoặc bất kỳ tech stack nào.
