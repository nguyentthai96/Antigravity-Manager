# Antigravity Ultra CLI — Hướng dẫn Build & Sử dụng

Antigravity Ultra là CLI tool chạy headless proxy server, cho phép proxy API token cho các tools bên ngoài (Claude Code, Cursor, Windsurf, v.v.) mà không cần mở GUI.

---

## 1. Yêu cầu hệ thống

| Yêu cầu | Chi tiết |
|----------|----------|
| **Rust** | >= 1.75 (cài qua [rustup.rs](https://rustup.rs)) |
| **OS** | Linux (Ubuntu 20.04+), macOS, Windows |
| **Linux deps** | `sudo apt install -y build-essential libgtk-3-dev libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev` |

---

## 2. Build Binary

### Debug build (nhanh, dùng khi dev)

```bash
cd src-tauri
cargo build --bin antigravity-ultra
```

Binary output: `src-tauri/target/debug/antigravity-ultra`

### Release build (tối ưu, dùng khi deploy)

```bash
cd src-tauri
cargo build --bin antigravity-ultra --release
```

Binary output: `src-tauri/target/release/antigravity-ultra`

> **Tip:** Release build chạy nhanh hơn ~10x so với debug, nên dùng cho production.

---

## 3. Chuẩn bị file Accounts

Tạo file JSON chứa danh sách tài khoản Google với `email` và `refresh_token`:

```json
[
  {
    "email": "user1@gmail.com",
    "refresh_token": "1//0gHBj72JgELiH..."
  },
  {
    "email": "user2@gmail.com",
    "refresh_token": "1//0eMoQSJ0UEY3O..."
  }
]
```

Lưu file tại bất kỳ đâu, ví dụ: `~/antigravity_accounts.json`

---

## 4. Các lệnh CLI

### 4.1 Start Proxy Server

```bash
antigravity-ultra start [OPTIONS]
```

| Option | Mô tả | Mặc định |
|--------|--------|----------|
| `-p, --port <PORT>` | Port lắng nghe | `8045` |
| `-a, --accounts <FILE>` | Path đến file JSON accounts | `antigravity_accounts.json` |
| `--lan` | Cho phép truy cập từ LAN (bind `0.0.0.0`) | `false` (chỉ `127.0.0.1`) |
| `--auto-token` | Tự tạo User Token mặc định nếu chưa có | `true` |
| `--log-dir <DIR>` | Thư mục lưu log proxy request/response | `~/.antigravity_tools/logs/` |

**Ví dụ:**

```bash
# Chạy cơ bản — chỉ localhost, port 8045
./antigravity-ultra start --accounts ~/antigravity_accounts.json

# Chạy mở LAN, port tuỳ chọn
./antigravity-ultra start --port 9090 --lan --accounts ~/antigravity_accounts.json

# Chạy ở background (Linux/macOS)
nohup ./antigravity-ultra start --port 8045 --lan \
  --accounts ~/antigravity_accounts.json > proxy.log 2>&1 &

# Chạy với log directory tuỳ chỉnh
./antigravity-ultra start --accounts ~/accounts.json --log-dir /var/log/antigravity
```

Khi server start thành công, output sẽ hiển thị:

```
══════════════════════════════════════════════════
  ✅ Proxy Server Running
══════════════════════════════════════════════════
  📍 Address:  http://0.0.0.0:8045
  👥 Accounts: 8
  🔑 API Key:  sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
  📝 App logs: /home/user/.antigravity_tools/logs/app.log.*
  📋 Request logs: /home/user/.antigravity_tools/logs/proxy_requests.log.*
  📋 User Tokens (for external tools):
     🟢 sk-yyyyyy... (user: default, expires: never)
══════════════════════════════════════════════════
```

Nhấn `Ctrl+C` để dừng server.

---

### 4.2 Quản lý User Tokens (API Keys)

User Tokens là các `sk-...` API key cấp cho từng người dùng/tool bên ngoài.

#### Tạo token mới

```bash
antigravity-ultra token create --username <TÊN> [OPTIONS]
```

| Option | Mô tả | Mặc định |
|--------|--------|----------|
| `-u, --username <TÊN>` | Tên user/tool (bắt buộc) | — |
| `-e, --expires <TYPE>` | Thời hạn: `day`, `week`, `month`, `never` | `never` |
| `--max-ips <N>` | Giới hạn số IP (0 = unlimited) | `0` |
| `-d, --description <MÔ_TẢ>` | Ghi chú | — |

**Ví dụ:**

```bash
# Tạo token vĩnh viễn cho Claude Code
./antigravity-ultra token create --username claude-code --description "Token cho Claude Code"

# Tạo token 1 tháng, giới hạn 3 IP
./antigravity-ultra token create --username cursor --expires month --max-ips 3
```

#### Liệt kê tất cả tokens

```bash
./antigravity-ultra token list
```

#### Xoá (revoke) token

```bash
./antigravity-ultra token revoke <TOKEN_ID>
```

---

### 4.3 Xem thông tin kết nối

```bash
./antigravity-ultra info
```

Hiển thị: Base URL, API Key, danh sách User Tokens, và ví dụ curl.

---

## 5. Sử dụng API

### Gọi API qua curl

```bash
curl http://localhost:8045/v1/chat/completions \
  -H "Authorization: Bearer sk-YOUR_TOKEN_HERE" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### Cấu hình cho Claude Code

```bash
export ANTHROPIC_BASE_URL=http://localhost:8045
export ANTHROPIC_API_KEY=sk-YOUR_TOKEN_HERE
```

### Cấu hình cho Cursor / Continue

Trong settings, set:
- **API Base URL:** `http://localhost:8045/v1`
- **API Key:** `sk-YOUR_TOKEN_HERE`

### Health check

```bash
curl http://localhost:8045/healthz
# => {"status":"ok","version":"4.4.9"}
```

### Liệt kê models

```bash
curl http://localhost:8045/v1/models \
  -H "Authorization: Bearer sk-YOUR_TOKEN_HERE"
```

---

## 6. Bulk Import Tokens (API)

Import hàng loạt user + refresh_token và tự động tạo API key cho mỗi user:

```bash
curl -X POST http://localhost:8045/api/bulk-import-tokens \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-ADMIN_API_KEY" \
  -d '[
    {"username": "user1", "refresh_token": "1//0abc..."},
    {"username": "user2", "refresh_token": "1//0def..."}
  ]'
```

Response:

```json
{
  "total": 2,
  "success_count": 2,
  "failed_count": 0,
  "results": [
    {
      "username": "user1",
      "email": "user1@gmail.com",
      "api_key": "sk-generated-key-1",
      "account_id": "uuid-1",
      "status": "success",
      "error": null
    },
    ...
  ]
}
```

---

## 7. Cấu trúc dữ liệu

Tất cả dữ liệu được lưu tại `~/.antigravity_tools/`:

```
~/.antigravity_tools/
├── accounts.json        # Danh sách tài khoản đã import
├── accounts/            # Thông tin chi tiết từng account
├── gui_config.json      # Cấu hình proxy (port, auth mode, ...)
├── user_tokens.db       # SQLite database chứa User Tokens
├── token_stats.db       # Thống kê sử dụng token
├── proxy_logs.db        # Log các request qua proxy (DB)
├── security.db          # IP blacklist/whitelist
└── logs/
    ├── app.log.2026-08-10          # App logs (daily rolling)
    ├── app.log.2026-08-09
    ├── proxy_requests.log.2026-08-10  # Request/Response logs (JSONL, daily rolling)
    └── proxy_requests.log.2026-08-09
```

---

## 8. Hệ thống Logging

### 8.1 App Logs (`app.log.*`)

Log chương trình chung: khởi động, kết nối, lỗi hệ thống, refresh token, v.v.

- **Format:** Text, mỗi dòng kèm timestamp + level
- **Rolling:** Daily (1 file/ngày)
- **Auto-cleanup:** Xoá file > 7 ngày, tổng size > 1GB
- **Vị trí:** `~/.antigravity_tools/logs/app.log.YYYY-MM-DD`

```
2026-08-10T16:40:51+07:00  INFO 反代服务器启动在 http://0.0.0.0:8045
2026-08-10T16:40:52+07:00  INFO Token refreshed successfully! Expires in: 3599s
2026-08-10T16:41:30+07:00  INFO Request: POST /v1/chat/completions
```

### 8.2 Proxy Request Logs (`proxy_requests.log.*`)

Log chi tiết từng request/response qua proxy, bao gồm body, token usage, timing.

- **Format:** JSONL (1 JSON object/dòng)
- **Rolling:** Daily (1 file/ngày)
- **Auto-cleanup:** Xoá file > 7 ngày
- **Vị trí:** `~/.antigravity_tools/logs/proxy_requests.log.YYYY-MM-DD`
- **Tuỳ chỉnh:** `--log-dir /path/to/dir`

**Mỗi dòng chứa:**

```json
{
  "timestamp": "2026-08-10T16:45:30+07:00",
  "id": "uuid",
  "method": "POST",
  "url": "/v1/chat/completions",
  "status": 200,
  "duration_ms": 1523,
  "model": "claude-sonnet-4-20250514",
  "mapped_model": "claude-sonnet-4-20250514",
  "protocol": "openai",
  "account": "user@gmail.com",
  "client_ip": "127.0.0.1",
  "username": "claude-code",
  "tokens": { "input": 150, "output": 320, "cached": 0 },
  "request_body": "{\"model\":\"claude-sonnet-4-20250514\",...}",
  "response_body": "{\"thinking\":\"...\",\"content\":\"...\",\"tool_calls\":[...]}"
}
```

### 8.3 Xem logs

```bash
# Xem app logs realtime
tail -f ~/.antigravity_tools/logs/app.log.$(date +%Y-%m-%d)

# Xem proxy request logs realtime
tail -f ~/.antigravity_tools/logs/proxy_requests.log.$(date +%Y-%m-%d)

# Lọc request theo model
grep '"model":"claude' ~/.antigravity_tools/logs/proxy_requests.log.* | jq .

# Lọc request lỗi
grep -v '"status":200' ~/.antigravity_tools/logs/proxy_requests.log.* | jq .

# Thống kê token usage
cat ~/.antigravity_tools/logs/proxy_requests.log.$(date +%Y-%m-%d) | \
  jq -r '[.username, .model, .tokens.input, .tokens.output] | @tsv'
```

---

## 9. Chạy như systemd service (Linux)

Tạo file `/etc/systemd/system/antigravity-proxy.service`:

```ini
[Unit]
Description=Antigravity Proxy Server
After=network.target

[Service]
Type=simple
User=your-username
WorkingDirectory=/home/your-username
ExecStart=/path/to/antigravity-ultra start --port 8045 --lan --accounts /path/to/antigravity_accounts.json
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable antigravity-proxy
sudo systemctl start antigravity-proxy
sudo systemctl status antigravity-proxy
```
