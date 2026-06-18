# Usage Examples

## Quick Start (3 Steps)

```bash
# 1. Build the CLI
cd antigravity-cli
cargo build --release

# 2. List available accounts
./list-accounts.sh

# 3. Switch to an account
./switch-account.sh pphstory@gmail.com
```

## Method 1: Interactive Menu (Easiest)

```bash
./interactive-switch.sh
```

This will show you a numbered menu:

```
╔════════════════════════════════════════════╗
║   Antigravity Account Switcher (CLI)      ║
╚════════════════════════════════════════════╝

Available Accounts:

   1. thaint1@nttco.vn
   2. pphstory@gmail.com
   3. phamhoang20092000@gmail.com
   4. hoangpp@nttco.vn
   5. thinhdp@nttco.vn
   6. nguyentthai96@gmail.com
   7. trungvt3@nttco.vn
   8. lapnv@nttco.vn

Enter account number (or 'q' to quit):
```

Just type the number and press Enter!

## Method 2: Helper Script (Recommended)

### Basic Usage

```bash
# Switch to a specific account
./switch-account.sh pphstory@gmail.com
```

### With Enterprise Project ID

```bash
# For enterprise accounts that need a project ID
./switch-account.sh hoangpp@nttco.vn my-gcp-project-123
```

### List Accounts First

```bash
# See all available accounts
./list-accounts.sh

# Then switch
./switch-account.sh <email-from-list>
```

## Method 3: Direct CLI (Advanced)

### Basic Switch

```bash
./target/release/antigravity-cli \
  --accounts-file ./antigravity_accounts.json \
  --email pphstory@gmail.com
```

### With Project ID

```bash
./target/release/antigravity-cli \
  --accounts-file ./antigravity_accounts.json \
  --email hoangpp@nttco.vn \
  --project-id my-gcp-project-123
```

### Using Custom Accounts File

```bash
./target/release/antigravity-cli \
  --accounts-file /path/to/custom/accounts.json \
  --email user@example.com
```

## Method 4: Scripting & Automation

### Bash Script Example

```bash
#!/bin/bash

# Switch between multiple accounts in sequence
ACCOUNTS=(
    "pphstory@gmail.com"
    "hoangpp@nttco.vn"
    "nguyentthai96@gmail.com"
)

for email in "${ACCOUNTS[@]}"; do
    echo "Switching to $email..."
    ./switch-account.sh "$email"
    
    # Do some work with this account
    sleep 5
    
    echo "Done with $email"
done
```

### Cron Job Example

```bash
# Switch to a specific account every day at 9 AM
0 9 * * * cd /path/to/antigravity-cli && ./switch-account.sh work@company.com

# Switch to personal account at 6 PM
0 18 * * * cd /path/to/antigravity-cli && ./switch-account.sh personal@gmail.com
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Switch Antigravity Account
  run: |
    cd antigravity-cli
    ./switch-account.sh ${{ secrets.ANTIGRAVITY_EMAIL }}
```

## Windows Usage

### PowerShell

```powershell
# Basic switch
.\switch-account.ps1 -Email "pphstory@gmail.com"

# With project ID
.\switch-account.ps1 -Email "hoangpp@nttco.vn" -ProjectId "my-project-123"
```

### Command Prompt

```cmd
powershell -ExecutionPolicy Bypass -File switch-account.ps1 -Email "pphstory@gmail.com"
```

## Common Workflows

### Daily Work Routine

```bash
# Morning: Switch to work account
./switch-account.sh work@company.com

# Evening: Switch to personal account
./switch-account.sh personal@gmail.com
```

### Testing Multiple Accounts

```bash
# Test script that switches between accounts
for email in $(./list-accounts.sh | grep -oE '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}'); do
    echo "Testing with $email"
    ./switch-account.sh "$email"
    
    # Run your tests here
    # ...
    
    sleep 10
done
```

### Project-Based Switching

```bash
# Switch based on current directory/project
case "$(pwd)" in
    */project-a*)
        ./switch-account.sh team-a@company.com
        ;;
    */project-b*)
        ./switch-account.sh team-b@company.com
        ;;
    *)
        ./switch-account.sh default@company.com
        ;;
esac
```

## Troubleshooting Examples

### Check if Antigravity is Running

```bash
# Before switching
ps aux | grep -i antigravity | grep -v grep

# Switch account
./switch-account.sh pphstory@gmail.com

# Verify it restarted
ps aux | grep -i antigravity | grep -v grep
```

### Verify Token Injection

```bash
# On Linux
sqlite3 ~/.config/Antigravity/User/globalStorage/state.vscdb \
  "SELECT key FROM ItemTable WHERE key LIKE '%oauth%';"

# On macOS
sqlite3 ~/Library/Application\ Support/Antigravity/User/globalStorage/state.vscdb \
  "SELECT key FROM ItemTable WHERE key LIKE '%oauth%';"
```

### Check Device Fingerprint

```bash
# On Linux
cat ~/.config/Antigravity/User/globalStorage/storage.json | jq '.telemetry'

# On macOS
cat ~/Library/Application\ Support/Antigravity/User/globalStorage/storage.json | jq '.telemetry'
```

## Advanced Usage

### Environment Variables

```bash
# Set default accounts file
export ANTIGRAVITY_ACCOUNTS_FILE="$HOME/.antigravity/accounts.json"

# Use in script
./target/release/antigravity-cli \
  --accounts-file "${ANTIGRAVITY_ACCOUNTS_FILE:-./antigravity_accounts.json}" \
  --email "$1"
```

### Logging

```bash
# Log all switches
./switch-account.sh pphstory@gmail.com 2>&1 | tee -a switch.log

# With timestamp
echo "[$(date)] Switching to pphstory@gmail.com" >> switch.log
./switch-account.sh pphstory@gmail.com 2>&1 | tee -a switch.log
```

### Error Handling

```bash
#!/bin/bash

switch_account() {
    local email="$1"
    local max_retries=3
    local retry=0
    
    while [ $retry -lt $max_retries ]; do
        if ./switch-account.sh "$email"; then
            echo "Successfully switched to $email"
            return 0
        else
            retry=$((retry + 1))
            echo "Retry $retry/$max_retries..."
            sleep 5
        fi
    done
    
    echo "Failed to switch to $email after $max_retries attempts"
    return 1
}

switch_account "pphstory@gmail.com"
```

## Performance Tips

### Pre-build for Faster Switching

```bash
# Build once
cargo build --release

# Then switches are fast (5-10 seconds)
time ./switch-account.sh pphstory@gmail.com
```

### Parallel Account Validation

```bash
# Check multiple accounts in parallel
for email in $(./list-accounts.sh | grep '@'); do
    (
        echo "Checking $email..."
        # Your validation logic here
    ) &
done
wait
```

## Integration Examples

### With tmux

```bash
# Create tmux session with specific account
tmux new-session -d -s work
tmux send-keys -t work "./switch-account.sh work@company.com" C-m
tmux attach -t work
```

### With Docker

```dockerfile
FROM rust:latest

WORKDIR /app
COPY antigravity-cli /app/

RUN cargo build --release

CMD ["./switch-account.sh", "default@example.com"]
```

### With systemd

```ini
[Unit]
Description=Antigravity Account Switcher
After=network.target

[Service]
Type=oneshot
ExecStart=/path/to/antigravity-cli/switch-account.sh work@company.com
User=youruser

[Install]
WantedBy=multi-user.target
```
