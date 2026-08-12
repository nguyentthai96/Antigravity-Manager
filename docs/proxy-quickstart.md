# 🚀 Antigravity Ultra — Proxy Tools Quickstart Guide

Hướng dẫn khởi chạy proxy server và test API với curl.

---

## 1. Yêu cầu

- **Rust toolchain** đã cài đặt (`rustup`, `cargo`)
- File **accounts JSON** chứa danh sách tài khoản Google (mặc định: `antigravity_accounts.json`)
- Port **8045** khả dụng (hoặc tùy chỉnh)

---

## 2. Build Binary

```bash
cd src-tauri

# Build debug (nhanh hơn, dùng cho dev/test)
cargo build --bin antigravity-ultra

# Hoặc build release (tối ưu, dùng cho production)
cargo build --release --bin antigravity-ultra
```

Binary sẽ nằm tại:
- Debug: `target/debug/antigravity-ultra`
- Release: `target/release/antigravity-ultra`

---

## 3. Khởi chạy Proxy Server

### Cách 1: Chạy cơ bản (mặc định port 8045)

```bash
./target/debug/antigravity-ultra start
```

### Cách 2: Chỉ định port và accounts file

```bash
./target/debug/antigravity-ultra start \
  --port 8045 \
  --accounts /đường/dẫn/tới/antigravity_accounts.json
```

### Cách 3: Cho phép truy cập từ LAN

```bash
./target/debug/antigravity-ultra start \
  --port 8045 \
  --lan
```

### Cách 4: Tùy chỉnh log directory

```bash
./target/debug/antigravity-ultra start \
  --port 8045 \
  --log-dir /đường/dẫn/tới/logs
```

### Tất cả tùy chọn

| Flag | Mặc định | Mô tả |
|------|----------|-------|
| `--port`, `-p` | `8045` | Port lắng nghe |
| `--accounts`, `-a` | `antigravity_accounts.json` | Đường dẫn file accounts JSON |
| `--lan` | `false` | Bind `0.0.0.0` thay vì `127.0.0.1` |
| `--auto-token` | `true` | Tự tạo User Token mặc định nếu chưa có |
| `--log-dir` | `~/.antigravity_tools/logs` | Thư mục lưu log |

### Kết quả khi khởi chạy thành công

```
══════════════════════════════════════════════════
  ✅ Proxy Server Running
══════════════════════════════════════════════════
  📍 Address:  http://127.0.0.1:8045
  👥 Accounts: 3
  🔑 API Key:  sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

  📋 User Tokens (for external tools):
     🟢 sk-fbfab2cd5b69496c829964f35980e4ce (user: default, expires: never)
══════════════════════════════════════════════════
```

> **Lưu ý:** Nhấn `Ctrl+C` để dừng proxy server.

---

## 4. Test với curl

### 4.1 Claude Protocol (Anthropic Native) — Streaming

```bash
curl -N http://127.0.0.1:8045/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-fbfab2cd5b69496c829964f35980e4ce" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-6",
    "messages": [{"role": "user", "content": "Hello, xin chào!"}],
    "max_tokens": 256,
    "stream": true
  }'
```

### 4.2 Claude Protocol — Non-streaming

```bash
curl http://127.0.0.1:8045/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-fbfab2cd5b69496c829964f35980e4ce" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4-6",
    "messages": [{"role": "user", "content": "Hello, xin chào!"}],
    "max_tokens": 256
  }'
```

### 4.3 OpenAI Protocol — Chat Completions

```bash
curl http://127.0.0.1:8045/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-fbfab2cd5b69496c829964f35980e4ce" \
  -d '{
    "model": "claude-sonnet-4-6",
    "messages": [{"role": "user", "content": "Hello, xin chào!"}],
    "max_tokens": 256,
    "stream": true
  }'
```

### 4.4 Health Check

```bash
curl http://127.0.0.1:8045/healthz
```

### 4.5 List Models

```bash
curl http://127.0.0.1:8045/v1/models \
  -H "x-api-key: sk-fbfab2cd5b69496c829964f35980e4ce"
```

---

## 5. Model được hỗ trợ

### Claude Models (qua Google backend)

| Model ID | Map tới | Ghi chú |
|----------|---------|---------|
| `claude-sonnet-4-6` | `claude-sonnet-4-6` | ✅ Khuyên dùng |
| `claude-sonnet-4-6-thinking` | `claude-sonnet-4-6-thinking` | Có thinking |
| `claude-sonnet-4-5` | → `claude-sonnet-4-6` | Auto redirect |
| `claude-sonnet-4-5-thinking` | → `claude-sonnet-4-6-thinking` | Auto redirect |
| `claude-opus-4-6` | `claude-opus-4-6-thinking` | Opus + thinking |
| `claude-opus-4` | → `claude-opus-4-6-thinking` | Auto redirect |

### Gemini Models (pass-through)

| Model ID | Ghi chú |
|----------|---------|
| `gemini-2.5-flash` | Flash model |
| `gemini-3-flash` | Flash mới |
| `gemini-3-pro-preview` | Pro preview |
| `gemini-3.1-pro-low` | Pro tiết kiệm |
| `gemini-3.1-pro-high` | Pro cao cấp |
| `gemini-3-pro-image` | Tạo ảnh |

### OpenAI Models (redirect sang Gemini)

| Model ID | Map tới |
|----------|---------|
| `gpt-4`, `gpt-4-turbo` | → `gemini-2.5-flash` |
| `gpt-4o`, `gpt-4o-mini` | → `gemini-2.5-flash` |
| `gpt-3.5-turbo` | → `gemini-2.5-flash` |

> ⚠️ **Lưu ý:** Model `claude-sonnet-4-20250514` **KHÔNG** có trong bảng mapping. Sử dụng `claude-sonnet-4-6` thay thế.

---

## 6. Quản lý User Tokens (API Keys)

### Tạo token mới

```bash
./target/debug/antigravity-ultra token create \
  --username "claude-code" \
  --expires never \
  --description "Token cho Claude Code"
```

### Liệt kê tokens

```bash
./target/debug/antigravity-ultra token list
```

### Xóa token

```bash
./target/debug/antigravity-ultra token revoke <TOKEN_ID>
```

---

## 7. Xem thông tin kết nối

```bash
./target/debug/antigravity-ultra info
```

---

## 8. Tích hợp với AI Tools

### Claude Code / Antigravity IDE

Trong cấu hình Claude Code, thiết lập:

```
API Base URL: http://127.0.0.1:8045
API Key: sk-fbfab2cd5b69496c829964f35980e4ce
```

### Cursor

Trong Cursor Settings → Models:

```
OpenAI Base URL: http://127.0.0.1:8045/v1
API Key: sk-fbfab2cd5b69496c829964f35980e4ce
Model: claude-sonnet-4-6
```

---

## 9. Troubleshooting

| Lỗi | Nguyên nhân | Giải pháp |
|-----|-------------|-----------|
| `429 RESOURCE_EXHAUSTED` | Google account hết quota | Chờ quota reset hoặc thêm account |
| `401 UNAUTHORIZED` | API key sai | Kiểm tra `x-api-key` hoặc `Authorization: Bearer` |
| Model không nhận dạng | Tên model sai | Dùng đúng model ID (xem bảng trên) |
| `Connection refused` | Server chưa start | Chạy `antigravity-ultra start` trước |
| `Address already in use` | Port 8045 đã bị chiếm | Dùng `--port` khác hoặc kill process cũ |

### Kiểm tra port đã bị chiếm chưa

```bash
lsof -i :8045
# hoặc
ss -tlnp | grep 8045
```

### Xem log chi tiết

```bash
# App logs
tail -f ~/.antigravity_tools/logs/app.log

# Proxy request logs
tail -f ~/.antigravity_tools/logs/proxy_requests.log
```

---

## 10. API Endpoints Reference

| Method | Endpoint | Protocol | Mô tả |
|--------|----------|----------|--------|
| `POST` | `/v1/messages` | Anthropic | Claude Messages API |
| `POST` | `/v1/messages/count_tokens` | Anthropic | Đếm tokens |
| `GET` | `/v1/models/claude` | Anthropic | List Claude models |
| `POST` | `/v1/chat/completions` | OpenAI | Chat Completions |
| `POST` | `/v1/completions` | OpenAI | Completions |
| `GET` | `/v1/models` | OpenAI | List all models |
| `POST` | `/v1/images/generations` | OpenAI | Tạo ảnh |
| `POST` | `/v1/audio/transcriptions` | OpenAI | Chuyển giọng nói → text |
| `GET` | `/healthz` | — | Health check |
| `GET` | `/health` | — | Health check |
