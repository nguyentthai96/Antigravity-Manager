# Feature Planning Rules

> ⚠️ **Scope**: Only used by `openspec-propose` skill (standalone mode). **NOT loaded by core pipeline** (
`/wf_pre_openspec` → `/wf_openspec` → `/wf_openspec_apply` → `/opsx-archive`).

This document defines the mandatory rules and standards for analyzing, designing, and planning the development of a new
feature. This system of rules consists of 3 main parts (directly compatible with the `feat-propose` workflow): *
*Planning Rules** (management & structure), **Spec Rules** (business specifications), and **Design Rules** (technical
design).

---

## 1. Feature Planning Rules (Planning & Management Standards)

*These rules apply during high-level analysis, file structure planning, logic mapping, and change impact assessment.*

### 1.0. Change Type Classification (MANDATORY — FIRST STEP)

> 📌 **DELEGATES TO**: `.agent/common-rules/change_type_classification.md` — Step 0a-0d
> Read and execute that file BEFORE any file structure planning.
>
> **Output required:** `Change Type: MAINTENANCE | EXTEND | EXTEND-SPLIT | NEWBUILD-PARTIAL | NEWBUILD`
> - If EXTEND: file structure must show `[EXISTING - ENHANCE]` for shared components, NOT `[NEW]` duplicates
> - If NEWBUILD: proceed with full new component creation

### 1.1. File Structure and Changes (File Structure & Changes)

- **Create new:** Must list the file name, full path, and main purpose of the file.
- **Edit:** List the current file name, the specific method/function to be updated, and a brief summary of the expected
  changes.
- **Structure:** If there is a major structural change or moving packages/modules, a specific reason must be provided.
- **Directory tree format requirement:** File structure changes MUST be listed in tree format. Illustration:
  ```text
  omni-transfer-service/src/main/java/com/thaint/omni/transferservice/
  ├── controller/implement/interbankTransfer/      [NEW PACKAGE]
  │   └── InterbankTransactionController.java      [NEW]
  ├── service/implement/interbankTransfer/         [NEW PACKAGE]
  │   ├── InterbankInitTransactionService.java     [NEW]
  │   └── InterbankGenerateOTPService.java         [NEW]
  └── util/
      └── TransferValidationUtil.java              [EXISTING - ENHANCE]
  ```

### 1.2. Business Logic Mapping (Business Mapping)

*There must be a 1-1 mapping between the proposed logic design and the URD or functional specification document.*

| Logic/Method Name    | Detailed Behavior                                      | Associated Document Section (Ref ID/Section) | Note (Compliance / Constraint)                      |
|:---------------------|:-------------------------------------------------------|:---------------------------------------------|:----------------------------------------------------|
| Ex: `validateUser()` | Check user status (active) and role before transaction | Section 3.1 - Authentication requirements    | Comply with system security and authorization rules |

### 1.3. Configuration and Error Management (Configuration & Error Management)

- **Configs:** List new configuration keys along with: default value, description of the effect, and usage across
  environments.
- **Error Codes:** List new error codes, corresponding returning HTTP status codes, message templates displayed to the
  user, and the context triggering the error.

### 1.4. Impact Analysis (Impact Analysis)

- **Backward Compatibility:** Will this change break legacy APIs or older Mobile App versions currently consuming them?
- **Dependencies:** What other modules or services (internal or partners) will this logic depend on/call?
- **Risks:** Are there any special risk warnings (e.g., database downtime, background batch jobs) when deploying this
  feature?

### 1.5. Organization and Maintainability Mindset (Maintainability Mindset)

- Proactively design and allocate file locations aiming for long-term maintainability.
- All new files must strictly follow the current architectural pattern of the project (Service, Controller, Validator,
  DTO, Util, etc.).
- **Refactoring Rules:**
    - If there is a task to refactor legacy code, plan to thoroughly refactor one method/module first before expanding
      to other parts in the service.
    - Ensure function names, variable names, and service names are clear, consistent, and avoid ambiguous abbreviations.

---

## 2. Spec Rules (Requirements for Specifications - `specs`)

*The specification document focuses on answering the question: **"WHAT must this feature do?"** (Business Requirements &
API Contracts). Absolutely do not include specific code structures or concrete Java class names here.*

### 2.1. API Contracts (Standard Protocols)

- Must clearly describe API Endpoints, HTTP Methods (GET, POST, PUT...), and important Request Headers.
- Payload Definition: Must provide standard JSON payload structures for both the Request and Response.

### 2.2. Business Logic & Constraints (Business Logic & Constraints)

- Separately and coherently list the business validation steps.
- Clearly specify business blocking constraints. For example: maximum transfer limits per customer class, source account
  status validation.

### 2.3. Security & Authorization (Security & Authorization)

- Authorization Scope: Which audience does this API serve (Individual CA/SME customers, Large Enterprises, Internal
  Apps, or System-to-System)?
- Access Control: What specific Roles or Permissions are required to have the authority to execute the feature?

### 2.4. Edge Cases and Error Handling (Edge Cases & Error Handling)

- Rigorously think through and list all exceptional execution paths ("Sad paths").
- For each Sad path, map it directly to the Error Code and HTTP Status that the system will return.

---

## 3. Design Rules (Requirements for Design - `design.md`)

*The design document delves into the technical solution, answering the
question: **"HOW will the system build that feature?"** (Technical Architecture).*

### 3.1. Implementation File Structure (Files & Components Structure)

- Strictly apply rule 1.1: Use Tree Format to represent the package directory tree, and clearly describe the purpose of
  each Interface, Class, and Helper file that will appear. Adopt a mindset of separating interface and implementation
  logic.

### 3.2. Data Flow and Component Interaction (Component Interaction & Data Flow)

- A Sequence Diagram (using `mermaid`) is required to precisely depict the execution flow from: User -> Router/API
  Gateway -> `Controller` -> `Service` -> `Repository` -> External Service.

### 3.3. State and Data Storage (Data Persistence & State)

- Analyze and detail DDL changes for the Database: New tables to initialize, Columns to add, Indexes to create for query
  optimization.
- Caching: Define the Key structure for Redis storage (e.g., limit/OTP session cache).
- Asynchronous Events: Clearly define the Schema of messages (RabbitMQ/Kafka) intended to be published to external
  flows, if any.

### 3.4. Concurrency and Performance Optimization (Concurrency & Performance)

- **Mandatory:** Must clearly design the Blocking/Locking mechanism (Optimistic Lock blocking by version, or Distributed
  Lock preventing duplicate transaction requests) if the business logic touches: account balance deduction, batch
  allocation, OTP blocking.

### 3.5. External System Integrations (External Integrations)

- List the core bank services, internal notification services, or 3rd-party vendors involved in the communication
  stages.
- Describe along with standard load balancing configuration figures: Connect Timeout, Read Timeout, Retry logic upon
  network drops, and system protection circuit breaking scenarios (Circuit Breaker).
