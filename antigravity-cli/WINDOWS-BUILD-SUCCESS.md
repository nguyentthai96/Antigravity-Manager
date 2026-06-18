# ✅ Build Windows Thành Công!

## 📦 Package đã được tạo

### File ZIP (Khuyến nghị - Dễ transfer)
```
antigravity-cli-windows-x64.zip  (4.7 MB)
```

### Thư mục đầy đủ
```
antigravity-cli-windows/
├── antigravity-cli.exe          (12 MB) - Executable chính
├── switch-account.ps1           - PowerShell helper script
├── antigravity_accounts.json    - Danh sách 8 accounts
├── README-WINDOWS.txt           - Hướng dẫn nhanh cho Windows
├── README.md                    - Tài liệu đầy đủ
├── QUICKSTART.md                - Hướng dẫn bắt đầu
└── USAGE_EXAMPLES.md            - Ví dụ sử dụng
```

---

## 🚀 Cách Chuyển sang Windows

### Phương Pháp 1: USB Drive (Đơn giản nhất)
```bash
# Trên Ubuntu - Copy file ZIP vào USB
cp antigravity-cli-windows-x64.zip /media/usb/

# Trên Windows - Extract và sử dụng
```

### Phương Pháp 2: Network Share (Nếu cùng mạng)
```bash
# Trên Ubuntu - Share qua Samba hoặc copy qua mạng
scp antigravity-cli-windows-x64.zip user@windows-pc:/path/
```

### Phương Pháp 3: Cloud Storage
```bash
# Upload lên Google Drive, Dropbox, OneDrive
# Rồi download trên Windows
```

### Phương Pháp 4: Email (File nhỏ)
```bash
# Gửi file ZIP qua email cho chính mình
# Download trên Windows
```

---

## 💻 Cách Sử Dụng trên Windows

### Bước 1: Extract file ZIP
```
Right-click antigravity-cli-windows-x64.zip
→ Extract All...
→ Chọn thư mục đích
```

### Bước 2: Mở PowerShell
```
Shift + Right-click trong thư mục antigravity-cli-windows
→ "Open PowerShell window here"
```

### Bước 3: Chạy lệnh

**Cách 1: Dùng PowerShell Script (Dễ nhất)**
```powershell
# List accounts
Get-Content .\antigravity_accounts.json | ConvertFrom-Json | Select-Object -ExpandProperty email

# Switch account
.\switch-account.ps1 -Email "pphstory@gmail.com"

# Với project ID
.\switch-account.ps1 -Email "hoangpp@nttco.vn" -ProjectId "my-project-123"
```

**Cách 2: Dùng trực tiếp executable**
```powershell
.\antigravity-cli.exe --accounts-file .\antigravity_accounts.json --email "pphstory@gmail.com"
```

**Cách 3: Command Prompt**
```cmd
antigravity-cli.exe --accounts-file antigravity_accounts.json --email "pphstory@gmail.com"
```

---

## 📋 Danh Sách Accounts Có Sẵn

1. thaint1@nttco.vn
2. pphstory@gmail.com
3. phamhoang20092000@gmail.com
4. hoangpp@nttco.vn
5. thinhdp@nttco.vn
6. nguyentthai96@gmail.com
7. trungvt3@nttco.vn
8. lapnv@nttco.vn

---

## 🔍 Kiểm Tra Hoạt Động

### Test 1: Xem version
```powershell
.\antigravity-cli.exe --version
```

### Test 2: Xem help
```powershell
.\antigravity-cli.exe --help
```

### Test 3: List accounts
```powershell
Get-Content .\antigravity_accounts.json | ConvertFrom-Json | ForEach-Object { $_.email }
```

### Test 4: Switch account (thật)
```powershell
.\switch-account.ps1 -Email "pphstory@gmail.com"
```

---

## ⚠️ Troubleshooting trên Windows

### Lỗi: "cannot be loaded because running scripts is disabled"

**Giải pháp:**
```powershell
# Chạy PowerShell as Administrator
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Hoặc bypass cho lần chạy này
powershell -ExecutionPolicy Bypass -File .\switch-account.ps1 -Email "email@example.com"
```

### Lỗi: "Windows protected your PC"

**Giải pháp:**
```
Click "More info"
→ Click "Run anyway"
```

Hoặc:
```powershell
# Unblock file
Unblock-File .\antigravity-cli.exe
```

### Lỗi: "The system cannot find the path specified"

**Giải pháp:**
```powershell
# Đảm bảo đang ở đúng thư mục
cd path\to\antigravity-cli-windows

# Kiểm tra file có tồn tại
dir antigravity-cli.exe
```

### Lỗi: "Access is denied"

**Giải pháp:**
```powershell
# Chạy PowerShell as Administrator
# Right-click PowerShell → Run as Administrator
```

---

## 🎯 Ví Dụ Sử Dụng Thực Tế

### Ví dụ 1: Switch sang account công việc
```powershell
PS> .\switch-account.ps1 -Email "hoangpp@nttco.vn"

Found account: hoangpp@nttco.vn
Refreshing token...
Token refreshed successfully! Expires in: 3599 seconds
Closing Antigravity...
Device profile written...
Starting Token injection...
Token injection successful (new format)
Starting Antigravity...
Account switch to hoangpp@nttco.vn completed successfully!
```

### Ví dụ 2: Switch sang account cá nhân
```powershell
PS> .\switch-account.ps1 -Email "pphstory@gmail.com"
```

### Ví dụ 3: Tạo batch script để switch nhanh
Tạo file `switch-work.bat`:
```batch
@echo off
powershell -ExecutionPolicy Bypass -File switch-account.ps1 -Email "hoangpp@nttco.vn"
pause
```

Tạo file `switch-personal.bat`:
```batch
@echo off
powershell -ExecutionPolicy Bypass -File switch-account.ps1 -Email "pphstory@gmail.com"
pause
```

Sau đó chỉ cần double-click file .bat để switch!

---

## 📊 Thông Tin Kỹ Thuật

### Build Information
- **Platform**: Windows x86_64
- **Compiler**: MinGW-w64 (cross-compiled from Ubuntu)
- **Rust Version**: Latest stable
- **Build Type**: Release (optimized)
- **File Size**: 12 MB (executable), 4.7 MB (ZIP)

### System Requirements
- **OS**: Windows 10/11 (64-bit)
- **RAM**: 50 MB minimum
- **Disk**: 20 MB free space
- **Network**: Internet connection (for token refresh)

### Dependencies
- Antigravity phải được cài đặt trên Windows
- Không cần cài thêm runtime (statically linked)

---

## 🔒 Bảo Mật

### ⚠️ Quan Trọng
- File `antigravity_accounts.json` chứa **refresh tokens**
- Refresh tokens có quyền **full access** vào accounts
- **KHÔNG** share file này với ai
- **KHÔNG** commit vào Git
- **KHÔNG** upload lên public cloud

### Khuyến Nghị
1. Giữ file trong thư mục được bảo vệ
2. Xóa file sau khi không dùng nữa
3. Sử dụng BitLocker để mã hóa ổ đĩa
4. Định kỳ rotate refresh tokens

---

## 📚 Tài Liệu Thêm

Trong package có các file:
- `README-WINDOWS.txt` - Hướng dẫn nhanh
- `README.md` - Tài liệu đầy đủ
- `QUICKSTART.md` - Bắt đầu nhanh
- `USAGE_EXAMPLES.md` - Ví dụ chi tiết

---

## 🎉 Hoàn Thành!

Bạn đã có:
✅ File executable cho Windows (antigravity-cli.exe)
✅ PowerShell helper script
✅ Danh sách 8 accounts sẵn sàng
✅ Tài liệu đầy đủ
✅ Package ZIP dễ transfer

**Bước tiếp theo:**
1. Copy file `antigravity-cli-windows-x64.zip` sang Windows
2. Extract và chạy thử
3. Enjoy! 🚀

---

## 📞 Hỗ Trợ

Nếu gặp vấn đề:
1. Đọc phần Troubleshooting ở trên
2. Kiểm tra file README.md
3. Xem USAGE_EXAMPLES.md cho ví dụ chi tiết
