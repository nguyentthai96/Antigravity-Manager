# Antigravity CLI - Account Switcher

A minimal CLI tool for switching Antigravity accounts using refresh tokens.

## Features

- Load accounts from JSON file
- Refresh OAuth tokens automatically
- Generate new device fingerprints for isolation
- Inject tokens into Antigravity's state database
- Automatically restart Antigravity with the new account

## Prerequisites

- Rust toolchain installed
- Antigravity installed on your system
- Valid refresh tokens for your accounts

## Build

```bash
cd antigravity-cli
cargo build --release
```

The compiled binary will be at `target/release/antigravity-cli`

## Usage

### Basic Usage

```bash
./antigravity-cli \
  --accounts-file ./antigravity_accounts.json \
  --email pphstory@gmail.com
```

### With Enterprise Project ID

```bash
./antigravity-cli \
  --accounts-file ./antigravity_accounts.json \
  --email hoangpp@vnpay.vn \
  --project-id your-gcp-project-id
```

### Using the Helper Script

For easier usage, use the provided helper script:

```bash
# Make it executable
chmod +x switch-account.sh

# Switch to an account
./switch-account.sh pphstory@gmail.com

# With project ID
./switch-account.sh hoangpp@vnpay.vn your-gcp-project-id
```

## Account File Format

The `antigravity_accounts.json` file should contain an array of accounts:

```json
[
  {
    "email": "user@example.com",
    "refresh_token": "1//0g..."
  }
]
```

## How It Works

1. **Load Accounts**: Reads the accounts JSON file
2. **Find Target**: Locates the account by email
3. **Refresh Token**: Gets a fresh access token from Google OAuth
4. **Close Antigravity**: Gracefully shuts down the running Antigravity process
5. **Generate Fingerprint**: Creates a new device profile for isolation
6. **Inject Token**: Writes the token to Antigravity's state database
7. **Restart**: Launches Antigravity with the new account

## Platform Support

- ✅ macOS
- ✅ Windows
- ✅ Linux

## Database Locations

### macOS
- State DB: `~/Library/Application Support/Antigravity/User/globalStorage/state.vscdb`
- Storage: `~/Library/Application Support/Antigravity/User/globalStorage/storage.json`

### Windows
- State DB: `%APPDATA%\Antigravity\User\globalStorage\state.vscdb`
- Storage: `%APPDATA%\Antigravity\User\globalStorage\storage.json`

### Linux
- State DB: `~/.config/Antigravity/User/globalStorage/state.vscdb`
- Storage: `~/.config/Antigravity/User/globalStorage/storage.json`

## Troubleshooting

### "Account not found"
- Check that the email matches exactly in `antigravity_accounts.json`

### "Token refresh failed"
- Verify the refresh token is still valid
- Check your internet connection

### "Unable to close Antigravity"
- Manually close Antigravity and try again
- Check if you have permission to kill the process

### "Failed to open database"
- Ensure Antigravity is installed
- Check file permissions

## Security Notes

- Keep your `antigravity_accounts.json` file secure
- Refresh tokens provide full account access
- Never commit this file to version control
- Consider encrypting the file at rest

## Related Files

- `src/main.rs` - Main entry point
- `src/oauth.rs` - OAuth token refresh logic
- `src/db.rs` - Database injection logic
- `src/device.rs` - Device fingerprint generation
- `src/process.rs` - Process management
- `src/protobuf.rs` - Protobuf encoding/decoding
- `src/models.rs` - Data structures
