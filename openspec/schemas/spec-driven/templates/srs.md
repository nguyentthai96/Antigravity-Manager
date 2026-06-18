<!-- 
  SRS Template theo format chuẩn VietABank.
  AI sẽ chọn/loại bỏ các Phase phù hợp với flow type:
  - BaseCommandHandler: chỉ có 1 phase (no OTP)
  - BaseInitFinancialHandler: Init → GenOTP → ConfirmOTP (3 phases)
  - BaseInitNonFinancialHandler: Init → Confirm (2 phases)
  - BaseQueryHandler: chỉ có 1 phase (query only)
  Fill data từ proposal.md, design.md, delta specs/ đã generate trước đó.
-->

# SRS: <Feature Name>

> **Type**: NEWBUILD | MAINTENANCE
> **Date**: <YYYY-MM-DD>
> **Module**: <service>/<module>
> **Service Code**: <code> (`<EnumClass>.<CONSTANT>`)
> **Flow Type**: <BaseClass>
> **Source**: pre_openspec.md + openspec artifacts (proposal, design, specs/)

---

## 1. Phạm vi tính năng

### 1.1 Mô tả
<!-- Mô tả ngắn gọn feature — KH làm gì, ở đâu, bằng gì, theo kiến trúc nào -->

### 1.2 Phạm vi IN-SCOPE
<!-- List các chức năng, API endpoints, tích hợp hệ thống trong scope -->

### 1.3 Phạm vi OUT-OF-SCOPE
<!-- Chức năng không thuộc phạm vi -->

---

## 2. Kiến trúc tổng quan

### 2.1 Pattern áp dụng

| Layer | Class | Mô tả |
|---|---|---|
| Controller | `<ClassName>` | REST endpoint, extends `<BaseClass>` |
| Handler/UseCase | `<ClassName>` | Business logic, extends `<BaseClass>` |
| Factory | `<ClassName>` | Data access, extends `<BaseFactory>` |
| Client | `<ClassName>` | AGW integration |

### 2.2 Cấu trúc package

```
<service>/src/main/java/<base_package>/
└── <feature>/
    ├── controller/
    │   ├── app/
    │   └── web/
    ├── handler/
    ├── factory/
    │   └── impl/
    ├── model/
    │   ├── request/
    │   └── response/
    ├── filter/
    ├── enumerate/
    ├── entity/
    └── repository/
```

### 2.3 Hệ thống vệ tinh

| Hệ thống | Mục đích | Phase sử dụng |
|---|---|---|
| Core Banking (AGW) | | Init / Confirm |
| Redis Cache | | |
| IConfigFactory | Đọc cấu hình động | |

---

<!-- ============================================================ -->
<!-- PHASE 1: INIT (bắt buộc cho mọi flow type)                   -->
<!-- Với Command flow (no OTP): đây là phase DUY NHẤT              -->
<!-- ============================================================ -->

## 3. Phase 1 — Init

### 3.1 API Specification

```
Method: POST
URL: /api/v1/{channel}/<module>/<endpoint>
Content-Type: application/json
```

### 3.2 Request — `<RequestClass>` extends `<BaseRequest>`

| Field | Type | Required | Validation | Mô tả |
|---|---|---|---|---|
| | | Yes/No | @NotBlank / @NotNull / @Size(max=N) | |

### 3.3 Response — `<ResponseClass>`

| Field | Type | Mô tả | Nguồn dữ liệu |
|---|---|---|---|
| code | String | Mã kết quả ("00" = thành công) | Logic xử lý |
| message | String | Mô tả kết quả | Logic xử lý |
| data | Object | Dữ liệu trả về | |

### 3.4 Flow xử lý tuần tự

<!-- Mô tả chi tiết từng bước xử lý: preHandle, aroundHandle, factory calls -->

1. **preHandle()**: Validate input fields
2. **aroundHandle()**: Build filter → Call factory → Map response
3. ...

### 3.5 Error Scenarios

| Scenario | Error Code | Message | Ghi chú |
|---|---|---|---|
| | | | |

### 3.6 Demo API — Init

**Request:**
```json
{
}
```

**Response (Success):**
```json
{
  "code": "00",
  "message": "Success",
  "data": {}
}
```

**Response (Error):**
```json
{
  "code": "<error_code>",
  "message": "<error_message>",
  "data": null
}
```

---

<!-- ============================================================ -->
<!-- PHASE 2: GenOTP (chỉ cho Financial / NonFinancial flow)       -->
<!-- Bỏ section này nếu flow type là Command hoặc Query           -->
<!-- ============================================================ -->

## 4. Phase 2 — GenOTP

### 4.1 API Specification

```
Method: POST
URL: /api/v1/{channel}/<module>/auth-method
Content-Type: application/json
```

### 4.2 Request

| Field | Type | Required | Validation | Mô tả |
|---|---|---|---|---|
| transId | String | Yes | | Mã giao dịch từ Phase Init |
| otpType | String | Yes | PIN / SMS / SoftOTP / Biometric | Loại xác thực |

### 4.3 Response

| Field | Type | Mô tả | Nguồn dữ liệu |
|---|---|---|---|
| otpId | String | Mã OTP session | OTP Service |

### 4.4 Error Scenarios

| Scenario | Error Code | Message | Ghi chú |
|---|---|---|---|
| | | | |

### 4.5 Demo API — GenOTP

---

<!-- ============================================================ -->
<!-- PHASE 3: ConfirmOTP (chỉ cho Financial / NonFinancial flow)   -->
<!-- Bỏ section này nếu flow type là Command hoặc Query           -->
<!-- ============================================================ -->

## 5. Phase 3 — ConfirmOTP

### 5.1 API Specification

```
Method: POST
URL: /api/v1/{channel}/<module>/confirm
Content-Type: application/json
```

### 5.2 Request

| Field | Type | Required | Validation | Mô tả |
|---|---|---|---|---|
| transId | String | Yes | | Mã giao dịch |
| otpValue | String | Yes | | Giá trị OTP |

### 5.3 Response

| Field | Type | Mô tả | Nguồn dữ liệu |
|---|---|---|---|
| | | | |

### 5.4 Flow xử lý (ConfirmOTP Logic)

<!-- Mô tả chi tiết execution logic: confirm strategy, bank call, post-processing -->

1. Verify OTP
2. Call Bank Core
3. Update transaction status
4. ...

### 5.5 Revert Chain (Rollback Logic)

<!-- Mô tả tuần tự các bước revert nếu giao dịch thất bại -->

| Step | Action | Hệ thống | Mô tả |
|---|---|---|---|
| 1 | Revert Authorization | Core Banking | |
| 2 | Void Transaction | VMMS | |

### 5.6 Error Scenarios — ConfirmOTP

| Scenario | Error Code | Message | Xử lý revert |
|---|---|---|---|
| | | | |

### 5.7 Demo API — ConfirmOTP

---

## 6. Giao diện (UI Design)

### 6.1 Tổng quan luồng UI
<!-- Mô tả flow màn hình: Home → Feature → Init → OTP → Result -->

### 6.2 Chi tiết từng màn hình

<!-- Mỗi màn hình: tên, components, labels, interaction rules -->

| Màn hình | Mô tả | Action |
|---|---|---|
| | | |

---

## 7. Tổng hợp API vệ tinh

<!-- Liệt kê tất cả API external: Core Banking, 2FA, Notification, etc. -->

| API / Endpoint | Method | Phase | Mô tả |
|---|---|---|---|
| | | | |

---

## 8. Error Code tổng hợp

| Code | Constant | Tình huống | Hành động client |
|---|---|---|---|
| 00 | SUCCESS | Thành công | Hiển thị kết quả |
| | | | |

---

<!-- Chỉ cho MAINTENANCE variant -->
## 9. Change Delta (MAINTENANCE only)

| # | FR Ref | Change Type | Current Behavior | New Behavior |
|---|---|---|---|---|
| | | ADD / MODIFY / REMOVE | | |

### Impact Analysis

| Component | File/Class | Impact Type | Description |
|---|---|---|---|
| | | New / Modified / Deleted | |

### Upgrade Strategy
- Data migration plan
- Feature toggle
- Rollback plan

---

## 10. UNCOVER Items

| Item | Mô tả | File/Interface |
|---|---|---|
| | | |

---

## 11. Open Questions & Issues
<!-- Từ pre_openspec.md -->
