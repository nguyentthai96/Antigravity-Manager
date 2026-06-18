---
trigger: always_on
---

# AI Rules chuẩn hóa cho Agentic Dev trong Antigravity IDE
- Use Vietnamese for chat and documentation; English for code and comments.
- Always check for existing implementations before creating new ones (reuse-first).
- Follow project architecture strictly (e.g., Clean Architecture).
- Understand context before coding; never assume.
- Prefer small, safe, incremental changes.
- Avoid duplicate logic and unnecessary abstractions.
- Keep code clean, readable, and maintainable.
- Do not refactor broadly unless explicitly requested.
- Maintain backward compatibility.
- Tài liệu spec và phân tích nghiệp vụ business sẽ được lưu trữ một folder riêng biệt *_docs (docs/*), source code sẽ gồm fronend/app và backend

#
# Ngôn ngữ & Quy ước giao tiếp
- Chat / Trao đổi / Giải thích: sử dụng Tiếng Việt (Human communication → dễ hiểu (Vietnamese))
- Documentation (Markdown artifacts): sử dụng Tiếng Việt
- Source code & code comments: sử dụng Tiếng Anh (Codebase → chuẩn quốc tế (English))
# Reuse-first Principle (Nguyên tắc tái sử dụng)
- Quy tắc:
Luôn kiểm tra:
Function đã tồn tại?
Class / module đã có?
Service / util có thể reuse?
- Ưu tiên:
Reuse trực tiếp
Extend / Refactor nhẹ
Tạo mới (chỉ khi bắt buộc)
- Checklist trước khi code:
[ ] Đã search toàn repo?
[ ] Có util/service tương tự?
[ ] Có thể generic hóa không?
[ ] Có duplicate logic không?
# Architecture Consistency (Tuân thủ kiến trúc)
Phải tuân theo kiến trúc project (Clean Architecture)
Nguyên tắc:
- Không bypass layer
- Domain phải: Pure (không phụ thuộc framework)
# Code Understanding Before Action
Không được code khi chưa hiểu context
Bắt buộc:
Đọc: File liên quan; Dependency chain
Hiểu: Data flow; Business logic; Side effects
Nếu chưa rõ: Phải ask clarification; Không được assume sai
# Incremental Change (Thay đổi nhỏ, an toàn)
- Không refactor lớn nếu không được yêu cầu
- Mỗi change phải: Scoped rõ ràng; Không phá vỡ behavior cũ
- Ưu tiên: Small PR mindset; Atomic changes
# Naming & Code Style
- Naming: Rõ nghĩa, không viết tắt mơ hồ; Theo domain language (DDD nếu có);
- Code:
# Clean, readable > clever
Avoid over-engineering
erformance & Optimization Awareness
Không optimize sớm (premature optimization)
Nhưng phải:
Tránh O(n²) nếu không cần thiết
Tránh redundant calls (API, DB)

# Safety & Stability
Không: Xóa code quan trọng nếu chưa xác minh; Thay đổi config hệ thống nếu không rõ impact
Khi sửa: Giữ backward compatibility

# Agent Behavior (Quan trọng cho Agentic System)
Agent phải:
- Chủ động: Suggest improvement (nếu rõ ràng)
- Nhưng không:
  + Tự ý refactor lớn
  + Tự ý đổi architecture
  + Decision hierarchy: Reuse > Refactor nhẹ > Tạo mới

# Documentation Rule
Khi tạo artifact: Dùng Markdown và Viết bằng tiếng Việt
Nội dung cần có: Context -> Decision -> Trade-offs

# 🚫 Strict Prohibitions
❌ Không đoán business logic
❌ Không tạo duplicate code
❌ Không bypass architecture
❌ Không đổi naming convention tùy ý