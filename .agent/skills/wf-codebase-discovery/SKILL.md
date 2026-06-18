---
name: wf-codebase-discovery
description: "Support skill cho workflow wf_codebase_discovery — cung cấp templates, rules, và utilities cho quá trình phân tích codebase."
risk: low
source: custom
date_added: "2026-05-26"
---

# Codebase Discovery Support Skill

Skill hỗ trợ workflow `wf_codebase_discovery.md` — cung cấp output templates, validation rules, và utility patterns cho quá trình phân tích và trích xuất kiến thức từ source code.

## Use this skill when

- Chạy `wf_codebase_discovery` workflow
- Cần tham khảo output format cho knowledge files
- Cần validation criteria cho discovery artifacts

## Do not use this skill when

- Đang implement features (dùng `openspec-apply-change`)
- Đang review architecture (dùng `architect-review`)
- Chạy pipeline chính (Openspec workflows)

## Output Artifact Registry

Discovery workflow sinh ra các artifact tại `base_knowledge/`:

| # | Artifact | Path | Description |
|---|----------|------|-------------|
| 1 | Architecture Overview | `01_architecture.md` | Kiến trúc tổng quan, tech stack, layer hierarchy |
| 2 | Database Schema | `02_database_schema.md` | Tables, JPA entities, relationship diagram |
| 3 | Business Flows | `03_business_flows.md` | Business flows cho từng nhóm tính năng |
| 4 | Security Infrastructure | `04_security_infrastructure.md` | Security, logging, error handling, observability |
| 5 | Development Conventions | `05_development_conventions.md` | Feature templates, coding patterns, anti-patterns |
| 6 | Features Index | `structures/propose/features.md` | Master feature index — endpoints, handlers, call chains |
| 7 | Code Patterns | `structures/propose/knowledge_code_patterns.md` | Auth patterns, flow styles, base class registry |
| 8 | Transaction Flows | `structures/propose/knowledge_transaction_flow.md` | Transaction flow patterns, sequence diagrams |
| 9 | Architecture Map | `structures/propose/knowledge_architecture.md` | Base classes, layers, conventions |
| 10 | System Overview | `structures/overview/overview_system.md` | Tech stack, system overview, dependencies |
| 11 | Knowledge Index | `knowledge_index.md` | Master index liệt kê tất cả artifacts |
| 12 | Coding Standard | `standards/coding_standard.md` | Coding conventions (naming, structure, patterns) |
| 13 | Logging Standard | `standards/logging_standard.md` | Logging patterns (MDC, levels, format) |
| 14 | Error Handling Standard | `standards/error_handling_standard.md` | Exception hierarchy, error codes, response format |
| 15 | Security Standard | `standards/security_standard.md` | Auth patterns, encryption, data protection |

## Standards Generation Rules

### `coding_standard.md`
Trích xuất từ source code thực tế:
- **Naming Conventions**: Phân tích class/method/field naming patterns đang dùng
- **Package Structure**: Mô tả cách tổ chức packages hiện tại
- **Code Patterns**: Init/Confirm flow, Factory pattern, Handler pattern
- **DTO Rules**: Request/Response DTO patterns
- **Annotation Usage**: Custom annotations, Spring annotations đang dùng

### `logging_standard.md`
Trích xuất từ cách log hiện tại trong codebase:
- **Log Framework**: SLF4J/Logback configuration
- **MDC Fields**: Các MDC fields đang dùng (traceId, sessionId, etc.)
- **Log Levels**: Quy tắc sử dụng DEBUG/INFO/WARN/ERROR
- **Sensitive Data**: Patterns masking dữ liệu nhạy cảm
- **Log Format**: Structured logging format (JSON/text)

### `error_handling_standard.md`
Trích xuất từ exception handling patterns:
- **Exception Hierarchy**: Base exception classes, custom exceptions
- **Error Codes**: Error code format, mapping error → HTTP status
- **Response Format**: Error response structure (RFC 7807 nếu có)
- **Global Handler**: @ControllerAdvice patterns đang dùng
- **Retry/Fallback**: Retry mechanisms, circuit breaker patterns

### `security_standard.md`
Trích xuất từ security implementation:
- **Authentication**: JWT/Session/OAuth2 patterns đang dùng
- **Authorization**: Role-based access, method-level security
- **Encryption**: Algorithms (ECDSA, AES, RSA) đang dùng cho data protection
- **Input Validation**: Validation chain, sanitization rules
- **Audit**: Audit logging patterns

## Validation Criteria

Discovery output phải thỏa mãn:

```
- [ ] knowledge_index.md liệt kê đầy đủ tất cả artifacts
- [ ] features.md có ≥1 feature với đầy đủ 12 fields
- [ ] knowledge_code_patterns.md có ≥1 auth pattern + ≥1 flow style
- [ ] knowledge_transaction_flow.md có ≥1 flow type
- [ ] standards/ chứa ≥3 documents (coding, logging, error_handling)
- [ ] Cross-references chính xác giữa các files
- [ ] Mỗi file có nội dung thực tế (không phải placeholder)
```

## Lazy-Load Strategy

Downstream workflows SHOULD NOT load tất cả knowledge files cùng lúc:

```
1. Đọc knowledge_index.md → biết có gì
2. Đọc features.md → biết features nào tồn tại
3. Load file cụ thể khi cần context cho task hiện tại
```

## Limitations
- Skill này là support — logic chính nằm trong `wf_codebase_discovery.md`
- Output quality phụ thuộc vào chất lượng index của SocratiCode + GitNexus
- Standards files chỉ phản ánh patterns đang tồn tại, không prescribe patterns mới
