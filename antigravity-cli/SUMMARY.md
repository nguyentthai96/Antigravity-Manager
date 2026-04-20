# Antigravity CLI - Summary

## What is this?

A lightweight command-line tool that allows you to quickly switch between Antigravity accounts using refresh tokens. It's a simplified version of the full Antigravity-Manager, focused solely on account switching.

## Key Differences from Full Manager

| Feature | Antigravity-Manager | Antigravity-CLI |
|---------|-------------------|-----------------|
| GUI | ✅ Full Tauri UI | ❌ Command-line only |
| Account Management | ✅ Add/Edit/Delete | ❌ Read-only from JSON |
| Quota Monitoring | ✅ Real-time | ❌ Not included |
| Proxy Server | ✅ Built-in | ❌ Not included |
| Account Switching | ✅ Via UI | ✅ Via command-line |
| Device Fingerprints | ✅ Full management | ✅ Auto-generated |
| Token Refresh | ✅ Automatic | ✅ On-demand |
| Size | ~100MB+ | ~5MB |

## Use Cases

1. **Quick Switching**: Rapidly switch between accounts without opening the full UI
2. **Automation**: Script account switches in CI/CD or testing workflows
3. **Headless Servers**: Switch accounts on remote machines without GUI
4. **Development**: Test different accounts during development

## Architecture

```
antigravity-cli
├── src/
│   ├── main.rs          # Entry point & CLI argument parsing
│   ├── oauth.rs         # Token refresh logic
│   ├── db.rs            # State database injection
│   ├── device.rs        # Device fingerprint generation
│   ├── process.rs       # Process management (start/stop)
│   ├── protobuf.rs      # Protobuf encoding/decoding
│   └── models.rs        # Data structures
├── antigravity_accounts.json  # Account list (refresh tokens)
├── switch-account.sh    # Helper script (Linux/macOS)
├── switch-account.ps1   # Helper script (Windows)
└── list-accounts.sh     # List available accounts
```

## How It Works

The CLI replicates the core account switching logic from the full manager:

1. **Token Refresh**: Uses Google OAuth to get fresh access tokens
2. **Process Management**: Gracefully closes and restarts Antigravity
3. **Device Isolation**: Generates unique device fingerprints per switch
4. **State Injection**: Writes tokens directly to Antigravity's SQLite database
5. **Format Support**: Handles both old and new Antigravity state formats

## Security Considerations

- Refresh tokens provide full account access
- Store `antigravity_accounts.json` securely
- Consider encrypting the file or using environment variables
- Never commit tokens to version control
- Tokens are written to Antigravity's local database only

## Performance

- Build time: ~1-2 minutes (first time)
- Switch time: ~5-10 seconds
- Binary size: ~5MB
- Memory usage: <50MB during execution

## Limitations

1. No GUI - command-line only
2. No account quota monitoring
3. No proxy server functionality
4. No account creation/deletion (read-only from JSON)
5. Requires manual refresh token management

## Future Enhancements

Possible improvements:
- Interactive account selection menu
- Encrypted token storage
- Account validation before switch
- Rollback to previous account on failure
- Integration with password managers
- Support for multiple account files

## Related Code

The CLI is based on the account switching logic from:
- `src-tauri/src/modules/account.rs::switch_account()`
- `src-tauri/src/modules/oauth.rs`
- `src-tauri/src/modules/device.rs`

## Building from Source

```bash
# Clone the repository
git clone <repo-url>
cd Antigravity-Manager/antigravity-cli

# Build release binary
cargo build --release

# Binary location
./target/release/antigravity-cli
```

## Contributing

To add features:
1. Modify source files in `src/`
2. Test with `cargo run -- --accounts-file ... --email ...`
3. Build release with `cargo build --release`
4. Update documentation

## License

Same as parent project (Antigravity-Manager)
