---
description: Integration test — gọi API thật vào DB thật sau khi apply code, verify full flow request → handler → database → response
---

Chạy integration test cho một OpenSpec change đã apply xong code. Gọi API thật vào database thật, verify dữ liệu persist đúng.

**Pipeline position**: ... → `/wf_openspec_apply` → **`/wf_integ_test`** ⟂ `/wf_client_doc` → `/wf_archive` → `/wf_gen_change_doc`

> Chạy **song song** với `/wf_client_doc`. Cả hai đều dựa vào trạng thái sau apply.

---

**Input**: Argument after `/wf_integ_test` is the change name (e.g., `/wf_integ_test sound-notification`).

**Output**: `openspec/changes/<name>/integration-test-report.md`

**Prerequisites**:
- Server đang chạy (hoặc có thể start được)
- Database accessible (connection string available)
- Có session/token hợp lệ để gọi API (hoặc mock auth)

**Steps**

## 1. Load context từ change

a. **Validate** change tồn tại ở `openspec/changes/<name>/`
b. **Read** files:
   - `tasks.md` — danh sách code changes (source of truth)
   - `design.md` — technical design, MID list
   - `specs/` — delta specs nếu có
c. **Extract scope**: Từ tasks.md, xác định:
   - Danh sách MID mới/thay đổi
   - Handler classes liên quan
   - Request/Response DTOs
   - Repository methods, DB tables affected

## 2. Scan API endpoints từ source code

a. **Đọc `Mid.java`** — trích xuất MID routing:
   ```java
   // Tìm pattern: case "XX": → Handler class
   ```
b. **Đọc Handler classes** — xác định:
   - Input: Request DTO fields (required/optional)
   - Output: Response DTO fields
   - DB operations: Repository methods called
   - Cache operations: Redis keys read/write
c. **Đọc Request/Response DTO** — build JSON payload mẫu:
   - Lấy tất cả fields với @JsonProperty
   - Xác định validation rules (@NotNull, @NotBlank, enum values...)

## 3. Build test scenarios

Tạo danh sách test cases cho từng MID:

### 3a. Happy path tests

Cho mỗi MID:
```
Test: <MID> — Happy Path
1. Prepare: Seed dữ liệu test vào DB nếu cần
2. Request: Build JSON với dữ liệu hợp lệ
3. Call: Gọi API thật
4. Assert Response: code == "00", check response fields
5. Assert DB: Query DB verify dữ liệu persist đúng
6. Cleanup: Rollback dữ liệu test nếu cần
```

### 3b. Error/Edge case tests

```
Test: <MID> — Invalid Input
1. Request: Build JSON với dữ liệu KHÔNG hợp lệ
2. Call: Gọi API thật
3. Assert: code != "00", error message rõ ràng

Test: <MID> — Missing Required Fields
Test: <MID> — Invalid Enum Value
Test: <MID> — Entity Not Found
```

### 3c. Cross-MID tests (nếu có)

```
Test: Update (MID X) → Get (MID Y) consistency
1. Call Update API với dữ liệu mới
2. Call Get API
3. Assert: Get trả về đúng dữ liệu vừa Update
```

## 4. Execute tests

### 4a. Gọi API thật

Sử dụng `curl` hoặc HTTP client gọi trực tiếp:

```bash
# Template curl call
curl -s -X POST http://<host>:<port>/api \
  -H "Content-Type: application/json" \
  -d '{
    "mid": "<MID>",
    "sessionId": "<session>",
    "user": "<user>",
    "cif": "<cif>",
    "clientId": <clientId>,
    ... <test fields>
  }'
```

> **Note**: Nếu không có server chạy sẵn → fallback sang tạo `@SpringBootTest` class thay thế.

### 4b. Verify DB state

Sau mỗi API call, query database verify:

```sql
-- Template DB verification
SELECT <columns_affected> FROM <TABLE>
WHERE <PK> = '<test_value>';

-- Verify expected values
-- Compare before/after state
```

### 4c. Capture results

Cho mỗi test case, capture:
- Request payload (JSON)
- Response payload (JSON)
- Response time (ms)
- DB state before/after (nếu write operation)
- PASS / FAIL status
- Error details nếu FAIL

## 5. Generate integration-test-report.md

Tạo report tại `openspec/changes/<name>/integration-test-report.md`:

````markdown
# Integration Test Report — <change-name>

_Generated on YYYY-MM-DD_
_Server: <host>:<port>_
_Database: <db-info>_

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | N |
| Passed | N |
| Failed | N |
| Pass Rate | X% |

## Test Results

### MID <XX> — <description>

#### ✅ Happy Path
- **Request**: `<JSON>`
- **Response**: `<JSON>` (Xms)
- **DB Verify**: `SELECT ... → <result>` ✅

#### ✅ Invalid Input
- **Request**: `<JSON>`
- **Response**: `<JSON>`
- **Expected**: code != "00" ✅

### MID <YY> — <description>
...

## Failed Tests (nếu có)

| Test | MID | Error | Root Cause |
|------|-----|-------|------------|
| ... | ... | ... | ... |

## Database State After Tests

```sql
-- Cleanup queries (rollback test data)
DELETE FROM <TABLE> WHERE <condition>;
```
````

## 6. Review & Conclude

a. **Review** report — đảm bảo cover đủ happy path + error cases
b. **Nếu có test FAIL** → log issue, run 6b-gitnexus for root cause, KHÔNG block archive nhưng cảnh báo
c. **Show summary** counts

### 6b-gitnexus. Root Cause Analysis (for FAILED tests — GitNexus-enhanced)

When tests FAIL, trace the execution flow to find root cause:

1. `gitnexus query "{error message or failing endpoint}"` → find related execution flows
2. `gitnexus context "{suspect handler}"` → trace callers/callees for data flow issues
3. `READ gitnexus://repo/{name}/process/{process}` → full execution trace of the failing flow
4. Identify root cause → log in report under "Root Cause" column

> See skill `gitnexus-debugging` for detailed debugging patterns.
> If GitNexus unavailable → rely on manual log analysis and stack traces.

---

**Output On Success**

```
## Integration Test Complete

**Change:** <change-name>
**Report:** openspec/changes/<name>/integration-test-report.md

| Result | Count |
|--------|-------|
| ✅ Passed | N |
| ❌ Failed | 0 |
| Total | N |

All tests passed. Ready for archive.
```

**Output With Failures**

```
## Integration Test Complete (with failures)

**Change:** <change-name>
**Report:** openspec/changes/<name>/integration-test-report.md

| Result | Count |
|--------|-------|
| ✅ Passed | N |
| ❌ Failed | M |
| Total | N+M |

⚠️ M test(s) failed. Review report before archiving.
```

---

**Guardrails**

- MUST read `tasks.md` first — source of truth cho scope
- MUST gọi API thật, KHÔNG mock handler logic
- MUST verify DB state sau mỗi write operation
- MUST include cả happy path VÀ error cases
- MUST capture request/response JSON trong report
- MUST include cleanup/rollback queries
- DO NOT block archive nếu có test fail — chỉ cảnh báo
- DO NOT hardcode test data — derive từ Request DTO + design.md
- Nếu server không accessible → fallback tạo @SpringBootTest class + ghi note trong report
