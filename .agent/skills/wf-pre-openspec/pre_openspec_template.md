# Pre-OpenSpec: <feature-name>

> **Type**: <MAINTENANCE | EXTEND | NEWBUILD>
> **Flow**: <Financial | Non-Financial | Command | Query>
> **Source**: <URD (Confluence) | User Idea>
> **Classification Evidence**: <keyword> → <module> → <file>
> **Archive**: <path | N/A>
> **Quality Score**: <XX>/100

## 📋 Feature Summary

<Tóm tắt ngắn gọn tính năng — 2-5 câu mô tả mục tiêu chính và phạm vi>

| Metric | Giá trị |
|--------|---------|
| Số FR | <count> (URD: <x>, Enriched: <y>) |
| Issues | <count> (🔴: <a>, 🟡: <b>) |
| Open Questions | <count> |
| **Quality Score** | **<score>/100** |

---

## 1. Actors

- <Actor 1>: <vai trò trong luồng>
- <Actor 2>: <vai trò trong luồng>

## 2. Functional Requirements

### FR-001: <Tiêu đề Vietnamese 3-6 từ> [URD|IDEA|ENRICHED]
- **Actor**: <actor>
- **Action**: Hệ thống phải <action> khi <condition>
- **Validation**: <rules>

### FR-002: ... (continue for all FRs)

## 3. Non-functional Requirements

---

## 4. Deduplicated & Consolidated

<Merged duplicates, conflict notes. Nếu không có → ghi "Không phát hiện trùng lặp.">

## 5. Enriched Domain Requirements

<Max 5 enriched FRs tagged [ENRICHED]. Nếu không bổ sung → ghi "Không bổ sung thêm.">

### Enriched FRs

<List enriched FR-IDs with justification>

### External Integrations (from Step 2d)

| Hệ thống | Mục đích | Ghi chú |
|-----------|----------|---------|

## 6. Assumptions

---

## 7. Quality Score

| Tiêu chí | Điểm | Deduction |
|----------|-------|-----------|
| Rõ ràng (Clarity) | /25 | FR-XXX: <reason> |
| Đầy đủ (Completeness) | /25 | FR-XXX: <reason> |
| Nhất quán (Consistency) | /25 | FR-XXX: <reason> |
| Kiểm thử được (Testability) | /25 | FR-XXX: <reason> |
| **Tổng** | **/100** | |

### Chi tiết trừ điểm

| # | Tiêu chí | Điểm trừ | FR | Lý do (trích URD) | Cách cải thiện |
|---|----------|----------|-----|-------------------|---------------|

---

## 8. Issues & Risks

- 🔴 <Critical issue> — FR-XXX
- 🟡 <Warning> — FR-XXX
- 🟢 <Info> — FR-XXX

| # | Loại | Mức độ | Mô tả | FR | Đề xuất |
|---|------|--------|-------|-----|---------|

> Nếu không có issues → ghi "Không phát hiện vấn đề."

## 9. Open Questions

- <Question needing URD clarification or user confirmation>

> Nếu không có → ghi "Không có câu hỏi mở."

## 10. DETECTED SCOPE

<!-- STRUCTURED_MARKER: DO NOT MODIFY section name or sub-headers -->

### 10.1 Domain

### 10.2 Flow Type

### 10.3 Candidate Services
- <service-1>: <reason — keyword/path/module/archive evidence>
- <service-2>: <reason>

### Detection Evidence
- Keyword: <keyword> → Module: <module> → File: <file>

### 10.4 External Integrations

### 10.5 Required Modules

---
## 11. Transaction Flow Detail

| Step | Actor | Action | System |
|------|-------|--------|--------|
| 1 | | | |
| 2 | | | |


## 12. Traceability Matrix

| FR-ID | URD Section | Spec Section | Affected Class | Status |
|-------|-------------|-------------|---------------|--------|
| FR-001 | <URD ref> | <spec ref or TBD> | <class or TBD> | Mapped / Pending |

## 13. Agent Notes (Tổng hợp bổ sung)

> Phần này agent TỰ DO bổ sung thông tin phân tích ngoài template.
> Mục đích: capture insights, patterns, risks, hoặc context quan trọng mà template không cover.

### Observations
<Nhận xét tổng quan về feature — độ phức tạp, rủi ro, điểm cần lưu ý>

### Related Features / Precedents
<Các feature tương tự đã làm, có thể tham khảo. Link archive nếu có.>

### Integration Notes
<Ghi chú về tích hợp hệ thống ngoài — protocol, constraints, dependencies>

### Suggested Approach
<Gợi ý hướng triển khai — reuse service nào, base class nào, pattern nào>

### Context from Confluence Images
<Thông tin trích xuất từ hình ảnh trong Confluence (mockup, diagram, table). Nếu không có → ghi N/A>
