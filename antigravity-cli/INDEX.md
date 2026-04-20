# Antigravity CLI - Documentation Index

Welcome to the Antigravity CLI documentation! This tool allows you to quickly switch between Antigravity accounts using refresh tokens.

## 📚 Documentation Files

### Getting Started
1. **[QUICKSTART.md](QUICKSTART.md)** - Get up and running in 3 steps
2. **[README.md](README.md)** - Complete feature overview and technical details
3. **[SUMMARY.md](SUMMARY.md)** - Architecture and comparison with full manager

### Usage Guides
4. **[USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)** - Comprehensive examples for all use cases

## 🚀 Quick Start

```bash
# 1. Build
cargo build --release

# 2. List accounts
./list-accounts.sh

# 3. Switch (choose one method)
./interactive-switch.sh              # Interactive menu
./switch-account.sh <email>          # Direct switch
```

## 📋 Available Scripts

| Script | Platform | Description |
|--------|----------|-------------|
| `interactive-switch.sh` | Linux/macOS | Interactive menu for account selection |
| `switch-account.sh` | Linux/macOS | Direct account switch with email |
| `switch-account.ps1` | Windows | PowerShell account switcher |
| `list-accounts.sh` | Linux/macOS | List all available accounts |

## 🎯 Common Use Cases

### For Daily Users
- **Quick switching**: Use `interactive-switch.sh` for a user-friendly menu
- **Frequent switches**: Use `switch-account.sh <email>` for speed

### For Developers
- **Testing**: Switch between test accounts in scripts
- **CI/CD**: Automate account switches in pipelines
- **Development**: Test features with different accounts

### For System Admins
- **Automation**: Schedule account switches with cron
- **Remote servers**: Switch accounts on headless systems
- **Batch operations**: Script multiple account switches

## 📖 Documentation Structure

```
antigravity-cli/
├── INDEX.md              ← You are here
├── QUICKSTART.md         ← Start here for basics
├── README.md             ← Full technical documentation
├── SUMMARY.md            ← Architecture overview
├── USAGE_EXAMPLES.md     ← Comprehensive examples
│
├── interactive-switch.sh ← Interactive menu (easiest)
├── switch-account.sh     ← Direct switch (fastest)
├── switch-account.ps1    ← Windows version
├── list-accounts.sh      ← List accounts
│
├── src/                  ← Source code
│   ├── main.rs          ← Entry point
│   ├── oauth.rs         ← Token refresh
│   ├── db.rs            ← Database injection
│   ├── device.rs        ← Fingerprint generation
│   ├── process.rs       ← Process management
│   ├── protobuf.rs      ← Protobuf encoding
│   └── models.rs        ← Data structures
│
└── antigravity_accounts.json  ← Your accounts (keep secure!)
```

## 🔍 Find What You Need

### I want to...

**Get started quickly**
→ Read [QUICKSTART.md](QUICKSTART.md)

**Understand how it works**
→ Read [README.md](README.md) and [SUMMARY.md](SUMMARY.md)

**See usage examples**
→ Read [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)

**Switch accounts interactively**
→ Run `./interactive-switch.sh`

**Switch accounts from command line**
→ Run `./switch-account.sh <email>`

**List available accounts**
→ Run `./list-accounts.sh`

**Use on Windows**
→ Run `.\switch-account.ps1 -Email "<email>"`

**Automate account switching**
→ See [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md) - Scripting section

**Integrate with CI/CD**
→ See [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md) - CI/CD section

**Troubleshoot issues**
→ See [README.md](README.md) - Troubleshooting section

**Understand the code**
→ See [SUMMARY.md](SUMMARY.md) - Architecture section

## 🎓 Learning Path

### Beginner
1. Read [QUICKSTART.md](QUICKSTART.md)
2. Try `./interactive-switch.sh`
3. Try `./switch-account.sh <email>`

### Intermediate
1. Read [README.md](README.md)
2. Try different usage methods from [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md)
3. Create simple automation scripts

### Advanced
1. Read [SUMMARY.md](SUMMARY.md) for architecture
2. Explore source code in `src/`
3. Integrate with your workflows
4. Contribute improvements

## 🔧 Technical Details

### Core Components
- **OAuth**: Token refresh using Google OAuth 2.0
- **Database**: SQLite state injection
- **Device**: Fingerprint generation for isolation
- **Process**: Antigravity lifecycle management
- **Protobuf**: Binary encoding for state format

### Supported Platforms
- ✅ Linux (tested on Ubuntu, Debian, Arch)
- ✅ macOS (tested on 10.15+)
- ✅ Windows (tested on Windows 10/11)

### Requirements
- Rust toolchain (for building)
- Antigravity installed
- Valid refresh tokens

## 📝 Quick Reference

### Build
```bash
cargo build --release
```

### Switch Account
```bash
# Interactive
./interactive-switch.sh

# Direct
./switch-account.sh <email>

# With project ID
./switch-account.sh <email> <project-id>
```

### List Accounts
```bash
./list-accounts.sh
```

### Windows
```powershell
.\switch-account.ps1 -Email "<email>"
.\switch-account.ps1 -Email "<email>" -ProjectId "<project-id>"
```

## 🔒 Security Notes

- Keep `antigravity_accounts.json` secure
- Never commit tokens to version control
- Consider encrypting the accounts file
- Tokens provide full account access

## 🤝 Contributing

To contribute:
1. Modify source files in `src/`
2. Test with `cargo run -- --accounts-file ... --email ...`
3. Build release with `cargo build --release`
4. Update documentation

## 📞 Support

For issues or questions:
1. Check [README.md](README.md) - Troubleshooting section
2. Review [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md) for examples
3. Examine source code in `src/`
4. Open an issue in the main repository

## 🎉 Credits

Based on the account switching logic from Antigravity-Manager:
- `src-tauri/src/modules/account.rs`
- `src-tauri/src/modules/oauth.rs`
- `src-tauri/src/modules/device.rs`

---

**Ready to start?** → [QUICKSTART.md](QUICKSTART.md)
