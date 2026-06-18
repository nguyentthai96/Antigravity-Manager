---
name: confluence-reader
description: Read Confluence pages recursively — detect parent/child hierarchy, discover linked pages, and ingest all related content into a single structured output.
metadata:
  author: Agent
  version: "1.0"
---

## Purpose

Nhận một URL/page_id Confluence, tự động:
1. Xác định page đó là **parent** (có child pages) hay **leaf** (không có con)
2. Nếu là parent → đệ quy đọc tất cả child pages
3. Scan nội dung tìm **link Confluence khác** được nhúng (cross-references) và gọi MCP tool để đọc
4. Đọc tất cả linked pages phát hiện được
5. Trả về **toàn bộ nội dung đã gom** dạng structured markdown

**Output**: Nội dung text đầy đủ của page + tất cả child + linked pages, sẵn sàng để workflow khác sử dụng (VD: `wf-pre-openspec`, `feat-propose`).

---

## Input

| Param | Required | Description |
|---|---|---|
| `CONFLUENCE_URL` | Có (1 trong 2) | Full URL dạng `https://{domain}/wiki/spaces/{SPACE}/pages/{PAGE_ID}/...` |
| `PAGE_ID` | Có (1 trong 2) | Page ID trực tiếp (VD: `123456789`) |
| `MAX_DEPTH` | Không | Độ sâu đệ quy child pages. Default: `3` |
| `FOLLOW_LINKS` | Không | Có đọc các page Confluence được link trong nội dung không? Default: `true` |
| `OUTPUT_VAR` | Không | Tên biến trả về. Default: `CONFLUENCE_CONTENT` |

---

## Step 1 — Parse Input & Xác định Page ID

### 1a. Nếu input là URL → extract Page ID
Confluence URL patterns:
```
https://{domain}/wiki/spaces/{SPACE_KEY}/pages/{PAGE_ID}/{PAGE_TITLE}
https://{domain}/wiki/spaces/{SPACE_KEY}/pages/{PAGE_ID}
https://{domain}/wiki/x/{SHORT_ID}
```

Extract `PAGE_ID` từ URL bằng regex:
- Pattern: `/pages/(\d+)`
- Nếu không match → thử extract `space_key` + `title` từ URL

### 1b. Gọi MCP tool đọc page gốc

```
mcp_atlassian_confluence_get_page(
    page_id = "{PAGE_ID}",
    include_metadata = true,
    convert_to_markdown = true
)
```

**Lưu kết quả**:
- `ROOT_PAGE`: nội dung + metadata
- `ROOT_TITLE`: tiêu đề page
- `ROOT_SPACE_KEY`: space key

### 1c. Đọc hình ảnh trên page gốc

> ⚠️ **BẮT BUỘC**: Nhiều tài liệu URD chứa thông tin quan trọng trong hình ảnh (sơ đồ luồng, mockup BO, bảng dữ liệu, wireframe). PHẢI đọc hình ảnh để phân tích đầy đủ.

```
mcp_atlassian_confluence_get_page_images(
    content_id = "{PAGE_ID}"
)
```

**Xử lý kết quả hình ảnh**:
- Tool trả về danh sách `ImageContent` (base64) — AI tự động phân tích nội dung ảnh
- Với mỗi ảnh, **mô tả nội dung chi tiết** và ghi nhận vào output:
  - Loại ảnh: `DIAGRAM` (sơ đồ luồng), `MOCKUP` (giao diện BO/app), `TABLE` (bảng dữ liệu trong ảnh), `ARCHITECTURE` (kiến trúc hệ thống), `OTHER`
  - Nội dung nhận diện: tên cột, giá trị mẫu, flow steps, v.v.
- **Đặc biệt với ảnh MOCKUP/TABLE**: Trích xuất chính xác tên cột, kiểu dữ liệu, giá trị mẫu → sử dụng cho thiết kế DB/Entity
- Lưu vào `PAGE_IMAGES[page_id]`

---

## Step 2 — Xác định Parent hay Leaf

### 2a. Kiểm tra có child pages không

```
mcp_atlassian_confluence_get_page_children(
    parent_id = "{PAGE_ID}",
    limit = 50,
    include_content = false
)
```

### 2b. Phân loại

| Kết quả | Loại | Hành động tiếp |
|---|---|---|
| Có children (count > 0) | **PARENT** | → Step 3 (đệ quy đọc children) |
| Không có children | **LEAF** | → Step 4 (scan links) |

**Log**: Ghi nhận `[INFO] Page "{ROOT_TITLE}" is {PARENT|LEAF} with {N} children`

---

## Step 3 — Đệ quy đọc Child Pages

### Algorithm

```
function readPageTree(pageId, currentDepth, maxDepth):
    if currentDepth > maxDepth:
        return []
    
    children = get_page_children(pageId)
    results = []
    
    for child in children:
        # Đọc nội dung child
        content = get_page(child.id, convert_to_markdown=true)
        results.append(content)
        
        # BẮT BUỘC: Đọc hình ảnh của child page
        images = get_page_images(child.id)
        if images.total_images > 0:
            PAGE_IMAGES[child.id] = analyze_images(images)
        
        # Đệ quy nếu child cũng có children
        results += readPageTree(child.id, currentDepth + 1, maxDepth)
    
    return results
```

### Implementation

Cho mỗi child page:

```
mcp_atlassian_confluence_get_page(
    page_id = "{CHILD_ID}",
    include_metadata = true,
    convert_to_markdown = true
)
```

**Tracking**: Duy trì danh sách `VISITED_PAGES` (Set of page_id) để tránh đọc trùng.

---

## Step 4 — Scan Nội Dung Tìm Linked Pages

### 4a. Tìm Confluence links trong nội dung markdown

Sau khi đọc mỗi page, scan nội dung tìm patterns:
```
Patterns to match:
1. [text](https://{domain}/wiki/spaces/{SPACE}/pages/{PAGE_ID}/...)
2. [text](/wiki/spaces/{SPACE}/pages/{PAGE_ID}/...)
3. <ac:link><ri:page ri:content-title="Page Title" ri:space-key="SPACE"/></ac:link>
4. href="/wiki/spaces/{SPACE}/pages/{PAGE_ID}"
```

Extract tất cả `PAGE_ID` từ links → thêm vào `LINKED_PAGES` queue.

> ⚠️ **Linked pages cũng PHẢI đọc hình ảnh** — nhiều trang API spec, bảng dữ liệu chỉ có hình ảnh mô tả.

### 4b. Phát hiện và đọc API documentation links

Trước khi đọc linked pages thông thường, thực hiện **API link detection riêng biệt**:

```
API_DOC_PATTERNS:
  1. Link text chứa từ khóa API: "API", "Service", "Gateway", "Endpoint", "Interface", "Swagger", "Spec"
  2. Link nằm trong bảng (table) có cột "API", "Link", "Tài liệu", "Chi tiết"
  3. Link trong section heading chứa: "API liên quan", "Tham chiếu API", "Related APIs"
  4. Link target page title chứa: "API", "Service", "v1/", "v2/"
```

**Quy tắc đọc API links**:
- Khi phát hiện link tới trang API → đánh dấu `is_api_doc = true`
- API doc pages được **ưu tiên đọc trước** linked pages thường
- API doc pages **ĐƯỢC phép follow links tiếp** (1 level nữa) nếu chúng link tới các API liên quan trong cùng flow
- Giới hạn: tối đa **10 API doc pages** mỗi lần quét

```
if FOLLOW_LINKS == true:
    # Phase 1: Read API doc links (deep follow allowed)
    for linked_page_id in API_DOC_LINKS:
        if linked_page_id NOT IN VISITED_PAGES:
            content = get_page(linked_page_id)
            VISITED_PAGES.add(linked_page_id)
            
            # Deep follow: scan API page for related API links (1 more level)
            nested_api_links = scan_for_api_links(content)
            for nested_id in nested_api_links:
                if nested_id NOT IN VISITED_PAGES and len(API_PAGES_READ) < 10:
                    nested_content = get_page(nested_id)
                    VISITED_PAGES.add(nested_id)
    
    # Phase 2: Read normal linked pages (no follow)
    for linked_page_id in NORMAL_LINKED_PAGES:
        if linked_page_id NOT IN VISITED_PAGES:
            content = get_page(linked_page_id)
            VISITED_PAGES.add(linked_page_id)
            # KHÔNG đệ quy children của linked pages thường
            # KHÔNG scan links của linked pages thường
```

> ⚠️ Linked pages thường chỉ đọc nội dung trực tiếp, KHÔNG đệ quy.
> ✅ **API doc pages được follow thêm 1 level** để đảm bảo đủ context cho toàn bộ luồng.

### 4c. Đọc Comments trên page

Comments trên Confluence thường chứa **phản hồi stakeholder, yêu cầu thay đổi, ghi chú revision** — quan trọng để xác định delta giữa các phiên bản.

```
for page_id in VISITED_PAGES:
    comments = mcp_atlassian_confluence_get_comments(page_id)
    if comments is not empty:
        PAGE_COMMENTS[page_id] = comments
```

**Rules**:
- Chỉ đọc comments của **root page** (wf-pre-openspec chỉ cần comments trên trang URD chính)
- Nếu số comments > 20 → chỉ lấy 20 comments mới nhất
- Lưu vào biến `PAGE_COMMENTS` để workflow cha sử dụng

---

## Step 5 — Tổng hợp Output

### Output format

Trả về biến `CONFLUENCE_CONTENT` chứa toàn bộ nội dung đã gom, structured:

```markdown
# Confluence Content Report

## Summary
- Root Page: {ROOT_TITLE} (ID: {PAGE_ID})
- Type: {PARENT|LEAF}
- Total pages read: {COUNT}
- Child pages: {CHILD_COUNT}
- Linked pages: {LINKED_COUNT}
- API doc pages: {API_DOC_COUNT}
- Images analyzed: {TOTAL_IMAGE_COUNT}
- Comments: {COMMENT_COUNT}

---

## 📄 Root: {ROOT_TITLE}
{Root page content in markdown}

### 🖼️ Images on this page

#### Image 1: {filename}
- **Type**: {DIAGRAM|MOCKUP|TABLE|ARCHITECTURE|OTHER}
- **Description**: {Mô tả chi tiết nội dung ảnh}
- **Extracted Data** (if TABLE/MOCKUP):
  | Column 1 | Column 2 | ... |
  |----------|----------|-----|
  | value    | value    | ... |

---

## 📁 Children

### 📄 {Child 1 Title}
{Child 1 content}

#### 🖼️ Images
{Image analysis if any}

### 📄 {Child 2 Title}
{Child 2 content}

#### 📄 {Grandchild Title}
{Grandchild content — indented by depth}

---

## 🔗 Linked Pages

### 📄 {Linked Page Title}
{Linked page content}

#### 🖼️ Images
{Image analysis if any}

---

## 📡 API Documentation (Auto-followed)

### 📄 {API Doc Page Title}
_Source: linked from {parent page title}_
{API doc content}

### 📄 {Nested API Doc Title}
_Source: linked from {API Doc Page Title} (deep follow)_
{Nested API doc content}

---

## 💬 Comments

### Comment by {Author} ({Date})
{Comment content}

### Comment by {Author} ({Date})
{Comment content}
```

### Heading depth
- Root page: `## 📄`
- Child depth 1: `### 📄`
- Child depth 2: `#### 📄`
- Linked pages: `### 📄` (under Linked Pages section)

---

## Step 6 — Trả kết quả

### Nếu gọi standalone
- In ra toàn bộ `CONFLUENCE_CONTENT`
- Lưu vào file nếu có `OUTPUT_FILE` param

### Nếu gọi từ workflow khác
- Set biến `CONFLUENCE_CONTENT` để workflow cha sử dụng
- Return summary: page count, titles list

---

## Error Handling

| Lỗi | Xử lý |
|---|---|
| Page not found (404) | Log warning, skip, continue with other pages |
| Permission denied | Log warning, ghi nhận "Access denied for page {ID}" |
| Rate limit | Wait 2s, retry 1 lần |
| Invalid URL | Return error message với URL pattern hướng dẫn |
| Quá nhiều pages (>50) | Log warning, chỉ đọc 50 pages đầu, ghi nhận còn lại |

---

## Guardrails

1. **Tránh vòng lặp**: LUÔN check `VISITED_PAGES` trước khi đọc
2. **Giới hạn đệ quy**: MAX_DEPTH mặc định = 3. Quá 3 level hiếm khi cần
3. **Linked pages không đệ quy**: Chỉ đọc nội dung, KHÔNG follow children/links tiếp
4. **Giới hạn tổng pages**: Tối đa 50 pages mỗi lần chạy
5. **Confluence MCP tools**: Dùng `mcp_atlassian_confluence_*` tools — KHÔNG dùng browser
6. **Metadata luôn bật**: `include_metadata = true` để lấy last_modified, author
7. **Markdown output**: `convert_to_markdown = true` cho đọc dễ hơn
8. **BẮT BUỘC đọc hình ảnh**: Mỗi page đọc xong text PHẢI gọi `get_page_images()` để lấy ảnh. Tài liệu URD/spec thường chứa thông tin quan trọng trong ảnh (sơ đồ BO, mockup giao diện, bảng cấu trúc DB, flow diagram) mà text content KHÔNG phản ánh được. **Bỏ qua ảnh = bỏ qua yêu cầu**.
9. **Phân tích ảnh chi tiết**: Khi nhận được ảnh từ MCP, PHẢI:
   - Nhận diện loại ảnh (DIAGRAM / MOCKUP / TABLE / ARCHITECTURE / OTHER)
   - Với ảnh **TABLE/MOCKUP**: trích xuất chính xác tên cột, kiểu dữ liệu, giá trị mẫu → phục vụ thiết kế DB/Entity
   - Với ảnh **DIAGRAM**: mô tả các bước, actors, decision points
   - Ghi nhận mô tả vào output structured markdown
10. **Giới hạn ảnh**: Tối đa 30 ảnh mỗi page. Nếu page có quá nhiều ảnh, ưu tiên ảnh có tên file chứa keyword: `mockup`, `table`, `flow`, `diagram`, `screen`, `bo`, `admin`
