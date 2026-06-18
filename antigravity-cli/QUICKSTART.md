# Quick Start Guide

## 1. Build the CLI

```bash
cd antigravity-cli
cargo build --release
```

## 2. List Available Accounts

```bash
cat antigravity_accounts.json | jq -r '.[].email'
```

Or without jq:
```bash
grep '"email"' antigravity_accounts.json
```

Current accounts in your file:
- thaint1@nttco.vn
- pphstory@gmail.com
- phamhoang20092000@gmail.com
- hoangpp@nttco.vn
- thinhdp@nttco.vn
- nguyentthai96@gmail.com
- trungvt3@nttco.vn
- lapnv@nttco.vn

## 3. Switch to an Account

### Option A: Using the helper script (recommended)

```bash
./switch-account.sh pphstory@gmail.com
```

### Option B: Direct CLI usage

```bash
./target/release/antigravity-cli \
  --accounts-file ./antigravity_accounts.json \
  --email pphstory@gmail.com
```

## 4. With Enterprise Project ID (if needed)

```bash
./switch-account.sh hoangpp@nttco.vn your-project-id
```

## What Happens During Switch

1. ✓ Reads your accounts file
2. ✓ Finds the account by email
3. ✓ Refreshes the OAuth token
4. ✓ Closes Antigravity (if running)
5. ✓ Generates new device fingerprint
6. ✓ Injects token into state database
7. ✓ Restarts Antigravity

## Example Output

```
Found account: pphstory@gmail.com
Refreshing token...
Token refreshed successfully! Expires in: 3599 seconds
Closing Antigravity...
Found PIDs to kill: [12345]
Device profile written to "/home/user/.config/Antigravity/User/globalStorage/storage.json"
Starting Token injection...
Token injection successful (new format)
Starting Antigravity...
Account switch to pphstory@gmail.com completed successfully!
```

## Troubleshooting

### Build fails
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Permission denied
```bash
chmod +x switch-account.sh
```

### Account not found
Check the email spelling matches exactly in `antigravity_accounts.json`

### Token refresh fails
The refresh token might be expired. You'll need to get a new one from the main Antigravity-Manager application.
