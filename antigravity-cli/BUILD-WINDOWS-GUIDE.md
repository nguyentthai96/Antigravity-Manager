# Hướng Dẫn Build cho Windows từ Ubuntu

## Phương Pháp 1: Cross-Compile (Khuyến nghị)

### Bước 1: Cài đặt công cụ cần thiết

```bash
# Cài đặt MinGW (cross-compiler cho Windows)
sudo apt-get update
sudo apt-get install -y mingw-w64

# Thêm target Windows vào Rust
rustup target add x86_64-pc-windows-gnu
```

### Bước 2: Cấu hình Cargo

```bash
# Tạo file cấu hình
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
EOF
```

### Bước 3: Build cho Windows

```bash
cd antigravity-cli
cargo build --release --target x86_64-pc-windows-gnu
```

### Bước 4: Lấy file .exe

File executable sẽ ở:
```
target/x86_64-pc-windows-gnu/release/antigravity-cli.exe
```

### Hoặc dùng script tự động:

```bash
cd antigravity-cli
./build-windows.sh
```

Script này sẽ:
- ✅ Cài đặt tất cả dependencies
- ✅ Build executable cho Windows
- ✅ Tạo package hoàn chỉnh với scripts và docs
- ✅ Tạo file ZIP để transfer sang Windows

---

## Phương Pháp 2: Docker (Nếu gặp vấn đề)

### Tạo Dockerfile

```dockerfile
FROM rust:latest

# Install MinGW
RUN apt-get update && \
    apt-get install -y mingw-w64 && \
    rustup target add x86_64-pc-windows-gnu

# Configure cargo
RUN mkdir -p /root/.cargo && \
    echo '[target.x86_64-pc-windows-gnu]' > /root/.cargo/config.toml && \
    echo 'linker = "x86_64-w64-mingw32-gcc"' >> /root/.cargo/config.toml && \
    echo 'ar = "x86_64-w64-mingw32-ar"' >> /root/.cargo/config.toml

WORKDIR /app
COPY . .

RUN cargo build --release --target x86_64-pc-windows-gnu

CMD ["cp", "target/x86_64-pc-windows-gnu/release/antigravity-cli.exe", "/output/"]
```

### Build với Docker

```bash
cd antigravity-cli

# Build Docker image
docker build -t antigravity-cli-builder .

# Run và copy file ra
docker run -v $(pwd)/output:/output antigravity-cli-builder

# File .exe sẽ ở thư mục output/
```

---

## Phương Pháp 3: GitHub Actions (CI/CD)

Tạo file `.github/workflows/build-windows.yml`:

```yaml
name: Build Windows

on:
  push:
    branches: [ main ]
  workflow_dispatch:

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        target: x86_64-pc-windows-gnu
        override: true
    
    - name: Install MinGW
      run: |
        sudo apt-get update
        sudo apt-get install -y mingw-w64
    
    - name: Configure Cargo
      run: |
        mkdir -p ~/.cargo
        cat > ~/.cargo/config.toml << 'EOF'
        [target.x86_64-pc-windows-gnu]
        linker = "x86_64-w64-mingw32-gcc"
        ar = "x86_64-w64-mingw32-ar"
        EOF
    
    - name: Build
      working-directory: antigravity-cli
      run: cargo build --release --target x86_64-pc-windows-gnu
    
    - name: Upload artifact
      uses: actions/upload-artifact@v3
      with:
        name: antigravity-cli-windows
        path: antigravity-cli/target/x86_64-pc-windows-gnu/release/antigravity-cli.exe
```

Sau đó download artifact từ GitHub Actions.

---

## Kiểm Tra Build

### Trên Ubuntu (trước khi chuyển sang Windows)

```bash
# Kiểm tra file đã được tạo
ls -lh target/x86_64-pc-windows-gnu/release/antigravity-cli.exe

# Kiểm tra file type
file target/x86_64-pc-windows-gnu/release/antigravity-cli.exe
# Output: PE32+ executable (console) x86-64, for MS Windows
```

### Trên Windows (sau khi chuyển file)

```powershell
# Kiểm tra version
.\antigravity-cli.exe --version

# Test chạy
.\antigravity-cli.exe --help
```

---

## Troubleshooting

### Lỗi: "linker `x86_64-w64-mingw32-gcc` not found"

```bash
# Cài lại MinGW
sudo apt-get install --reinstall mingw-w64
```

### Lỗi: "error: linking with `x86_64-w64-mingw32-gcc` failed"

```bash
# Kiểm tra MinGW đã cài đúng
which x86_64-w64-mingw32-gcc

# Nếu không có, cài lại
sudo apt-get purge mingw-w64
sudo apt-get install mingw-w64
```

### Lỗi: "cannot find -lwindows"

Thêm vào `Cargo.toml`:

```toml
[target.x86_64-pc-windows-gnu.dependencies]
winapi = { version = "0.3", features = ["winuser"] }
```

### Build chậm

```bash
# Sử dụng nhiều CPU cores
cargo build --release --target x86_64-pc-windows-gnu -j $(nproc)
```

---

## Package cho Windows

### Tạo package hoàn chỉnh

```bash
# Tạo thư mục distribution
mkdir -p antigravity-cli-windows
cd antigravity-cli-windows

# Copy executable
cp ../target/x86_64-pc-windows-gnu/release/antigravity-cli.exe .

# Copy scripts
cp ../switch-account.ps1 .

# Copy accounts file
cp ../antigravity_accounts.json .

# Copy docs
cp ../README.md .
cp ../QUICKSTART.md .

# Tạo file hướng dẫn Windows
cat > README-WINDOWS.txt << 'EOF'
Antigravity CLI for Windows
============================

Cách sử dụng:

1. Mở PowerShell trong thư mục này
2. Chạy: .\switch-account.ps1 -Email "email@example.com"

Hoặc dùng trực tiếp:
.\antigravity-cli.exe --accounts-file .\antigravity_accounts.json --email "email@example.com"

Xem thêm trong README.md
EOF

# Tạo file ZIP
cd ..
zip -r antigravity-cli-windows.zip antigravity-cli-windows/
```

### Transfer sang Windows

**Cách 1: USB/Network Share**
```bash
# Copy folder hoặc file ZIP sang Windows
```

**Cách 2: SCP (nếu Windows có SSH)**
```bash
scp antigravity-cli-windows.zip user@windows-pc:/path/to/destination/
```

**Cách 3: Cloud Storage**
```bash
# Upload lên Google Drive, Dropbox, etc.
# Rồi download trên Windows
```

---

## Chạy trên Windows

### PowerShell

```powershell
# Di chuyển vào thư mục
cd antigravity-cli-windows

# List accounts
Get-Content .\antigravity_accounts.json | ConvertFrom-Json | Select-Object -ExpandProperty email

# Switch account
.\switch-account.ps1 -Email "pphstory@gmail.com"

# Hoặc dùng trực tiếp
.\antigravity-cli.exe --accounts-file .\antigravity_accounts.json --email "pphstory@gmail.com"
```

### Command Prompt

```cmd
cd antigravity-cli-windows

antigravity-cli.exe --accounts-file antigravity_accounts.json --email "pphstory@gmail.com"
```

---

## Tóm Tắt Nhanh

```bash
# Trên Ubuntu - Build cho Windows
cd antigravity-cli
./build-windows.sh

# File output:
# - antigravity-cli-windows/antigravity-cli.exe
# - antigravity-cli-windows.zip

# Transfer sang Windows và chạy:
# PowerShell> .\switch-account.ps1 -Email "email@example.com"
```

---

## Kích Thước File

- **antigravity-cli.exe**: ~5-7 MB
- **Package đầy đủ**: ~6-8 MB
- **ZIP file**: ~2-3 MB (compressed)

---

## Lưu Ý Bảo Mật

⚠️ File `antigravity_accounts.json` chứa refresh tokens
- Không share file này
- Không commit vào Git
- Giữ an toàn khi transfer

---

## Hỗ Trợ

Nếu gặp vấn đề:
1. Kiểm tra MinGW: `x86_64-w64-mingw32-gcc --version`
2. Kiểm tra Rust target: `rustup target list | grep windows`
3. Xem log build chi tiết: `cargo build --release --target x86_64-pc-windows-gnu --verbose`
