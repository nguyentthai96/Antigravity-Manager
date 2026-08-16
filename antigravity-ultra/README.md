# Antigravity Ultra — Standalone Proxy

> Proxy server chuyển đổi OpenAI API format → Google Gemini backend, hỗ trợ multi-account rotation và streaming.

## 📋 Yêu cầu

- **Rust** ≥ 1.75 (cài qua [rustup](https://rustup.rs/))
- **Accounts file**: File JSON chứa danh sách accounts (`antigravity_accounts.json`)

## 🔨 Build

```bash
cd antigravity-ultra

# Build release (optimized)
cargo build --release

# Binary output tại:
# ./target/release/antigravity-ultra
```

> ⏱️ Build lần đầu mất ~2-3 phút. Các lần sau chỉ ~30 giây.

## 🚀 Start Proxy

### Cách 1: Với API key cố định (khuyến nghị)

```bash
# Xóa token DB cũ nếu muốn đổi key
rm -f ~/.antigravity_ultra/user_tokens.db

# Start proxy với key cố định
./target/release/antigravity-ultra start \
  --accounts ../antigravity-cli/antigravity_accounts.json \
  --port 8045 \
  --auto-token \
  --api-key "sk-fbfab2cd5b69496c829964f35980e4ce"
```

### Cách 2: Với key tự sinh

```bash
./target/release/antigravity-ultra start \
  --accounts ../antigravity-cli/antigravity_accounts.json \
  --port 8045 \
  --auto-token
```

> **Lưu ý**: Key được lưu persistent trong `~/.antigravity_ultra/user_tokens.db`. Restart sẽ **giữ nguyên key** — chỉ tạo mới nếu DB trống.

### CLI Flags

| Flag | Mô tả | Mặc định |
|------|--------|----------|
| `--port` / `-p` | Port lắng nghe | `8045` |
| `--accounts` / `-a` | Đường dẫn file accounts JSON | — |
| `--auto-token` | Tự tạo API key nếu chưa có | `false` |
| `--api-key` | Chỉ định API key cố định (dùng kèm `--auto-token`) | Random |
| `--lan` | Lắng nghe trên tất cả interfaces (0.0.0.0) | `false` |
| `--auto-refresh` | Tự refresh token hết hạn | `true` |
| `--healthcheck-interval` | Khoảng cách kiểm tra sức khỏe (giây, 0 = tắt) | `600` |

## ✅ Test Request

### Non-streaming

```bash
curl http://127.0.0.1:8045/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-fbfab2cd5b69496c829964f35980e4ce" \
  -d '{
    "model": "claude-sonnet-4-6",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 100
  }'
```

### Streaming

```bash
curl http://127.0.0.1:8045/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-fbfab2cd5b69496c829964f35980e4ce" \
  -d '{
    "model": "claude-sonnet-4-6",
    "messages": [{"role": "user", "content": "Hello!"}],
    "max_tokens": 256,
    "stream": true
  }'
```

## 🔑 Quản lý Token

```bash
# Liệt kê tất cả tokens
./target/release/antigravity-ultra token list

# Tạo token mới
./target/release/antigravity-ultra token create -u myuser -e never -d "For Cursor IDE"

# Xóa token
./target/release/antigravity-ultra token revoke <TOKEN_ID>
```

## 📁 File Cấu trúc

```
antigravity-ultra/
├── Cargo.toml              # Dependencies
├── README.md               # Tài liệu này
├── src/
│   ├── main.rs             # CLI & server startup
│   ├── proxy/
│   │   ├── mod.rs          # Request handler & Gemini transform
│   │   └── token_manager.rs # Account pool rotation
│   ├── models/             # Data models (Account, Token, Config)
│   ├── constants.rs        # API URLs, User-Agent, version
│   ├── config.rs           # Data directory (~/.antigravity_ultra/)
│   ├── user_token.rs       # API key management (SQLite)
│   ├── oauth.rs            # Google OAuth token refresh
│   └── logger.rs           # Structured logging
```

## 🗄️ Data Storage

| File | Đường dẫn | Mô tả |
|------|-----------|-------|
| Token DB | `~/.antigravity_ultra/user_tokens.db` | SQLite chứa API keys |
| Logs | `~/.antigravity_ultra/logs/` | Application logs |

## 🔧 Troubleshooting

### 1. Lỗi "Invalid token"
- Kiểm tra đúng API key: `./target/release/antigravity-ultra token list`
- Đảm bảo header `Authorization: Bearer <key>` đúng format

### 2. Lỗi 403 SERVICE_DISABLED
- Proxy tự động retry không gửi `x-goog-user-project` header
- Nếu vẫn lỗi → account chưa kích hoạt Cloud Code API

### 3. Lỗi 400 Bad Request
- Kiểm tra model name hợp lệ (VD: `claude-sonnet-4-6`, `gemini-2.5-pro`)
- Đảm bảo JSON body đúng format OpenAI

### 4. Port đã bị chiếm
```bash
# Tìm process đang dùng port
lsof -i :8045
# Kill process
kill -9 <PID>
```

## 🔄 Quick Restart

```bash
# Stop (Ctrl+C hoặc)
pkill -f antigravity-ultra

# Start lại (key được giữ nguyên từ DB)
./target/release/antigravity-ultra start \
  --accounts ../antigravity-cli/antigravity_accounts.json \
  --port 8045 \
  --auto-token
```
