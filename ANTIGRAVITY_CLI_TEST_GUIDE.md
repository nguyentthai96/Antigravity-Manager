# Testing Guide - Antigravity CLI macOS Account Switch

## Quick Test: Verify Process Detection Works

### Step 1: Start Antigravity IDE
```bash
open -a "Antigravity IDE"
# Wait 2-3 seconds for it to fully launch
```

### Step 2: Run the CLI with debug output
```bash
cd /Users/nguyenthanhthai/Desktop/workspace_research/Antigravity-Manager/antigravity-cli

# Test process detection (it will try to close and restart)
./target/release/antigravity-cli \
  --accounts-file ~/accounts.json \
  --email your-test-email@gmail.com \
  --target ide
```

### Step 3: Check Debug Output
Look for these lines (indicating successful detection and fix):
```
[DEBUG] Scanning for 'ide' processes on macOS...
[DEBUG] Found Antigravity (ide) process: PID=XXXXX, Name='electron', Exe='...Antigravity IDE.app...'
[DEBUG] ✓ Matched PID=XXXXX
Found 1 main Antigravity process(es):
  PID=XXXXX, Name=Electron, Exe=/Applications/Antigravity IDE.app/Contents/MacOS/Electron
```

If you see these, the fix is working! ✅

## Expected Behavior After Fix

### ✅ Process Detection Now Works
- Detects main "Electron" process inside `.app` bundle
- Filters out helper processes (Renderer, GPU, etc.)
- Works with both "Antigravity IDE" and "Antigravity" apps

### ✅ Account Switch Complete Flow
1. Finds and closes running app
2. Refreshes OAuth token
3. Injected new token into database
4. Restarts app with old account logged out
5. App opens fresh with new account

### ✅ Proper Error Handling
- If app refuses to close, shows clear error
- If token injection fails, reports which format failed
- Includes system information in error messages

## Comparison: Before vs After

### Before (Broken on macOS)
```
$ ./antigravity-cli --accounts-file ~/accounts.json --email test@gmail.com --target ide

Target: Antigravity IDE
Found account: test@gmail.com
Refreshing token...
Token refreshed successfully!
Closing Antigravity IDE...
Antigravity IDE is not running.  ← ❌ WRONG - process was running!
[Skips closing, token injection, restart]
Account switch to test@gmail.com completed successfully!  ← ❌ MISLEADING
```

### After (Fixed on macOS)
```
$ ./target/release/antigravity-cli --accounts-file ~/accounts.json --email test@gmail.com --target ide

Target: Antigravity IDE
Found account: test@gmail.com
Refreshing token...
Token refreshed successfully! Expires in: 3600 seconds
Closing Antigravity IDE...
[DEBUG] Scanning for 'ide' processes on macOS...
[DEBUG] Found Antigravity (ide) process: PID=7026, Name='electron', Exe='/Applications/Antigravity IDE.app/Contents/MacOS/Electron'
[DEBUG] ✓ Matched PID=7026
Found 1 main Antigravity process(es):
  PID=7026, Name=Electron, Exe=/Applications/Antigravity IDE.app/Contents/MacOS/Electron
Sending SIGTERM to PID=7026
Antigravity IDE closed successfully (graceful).
Device profile written to /Users/USERNAME/Library/Application Support/Antigravity IDE/User/globalStorage/storage.json
Starting Token injection...
Token injection successful (new format)
Starting Antigravity IDE...
Sent start command to Antigravity IDE. Waiting for app to launch...
[DEBUG] Scanning for 'ide' processes on macOS...
[DEBUG] Found Antigravity (ide) process: PID=7234, Name='electron', Exe='/Applications/Antigravity IDE.app/Contents/MacOS/Electron'
[DEBUG] ✓ Matched PID=7234
Antigravity IDE started successfully (verified running).
Account switch to test@gmail.com completed successfully! (target: ide)  ← ✅ WORKS NOW!
```

## Troubleshooting

### Issue: "Antigravity IDE is not running" but it is running
**Cause:** Process detection not working
**Solution:** 
1. Check debug output with `[DEBUG]` lines
2. Run `ps aux | grep -i antigravity` to see actual processes
3. Verify `/Applications/Antigravity IDE.app` exists

### Issue: "Unable to close Antigravity process"
**Cause:** App not responding to SIGTERM
**Solution:**
1. Manually force-quit the app
2. Try running the CLI again

### Issue: "Token injection failed"
**Cause:** 
- Database file is locked (Antigravity still has it open)
- Different Antigravity version format
**Solution:**
1. Ensure Antigravity was properly closed
2. Check if new keyring format is needed (newer versions)

## Verification Checklist

- [ ] Binary compiles: `cargo build --release`
- [ ] Process detection finds Antigravity: See `[DEBUG] Found Antigravity (ide) process`
- [ ] App closes properly: See "Antigravity IDE closed successfully"
- [ ] Token injected: See "Token injection successful"
- [ ] App restarts: See "Antigravity IDE started successfully (verified running)"
- [ ] Account switched: Log into Antigravity with new account

## Performance Notes

- Process scanning: ~100-200ms (querying all running processes)
- App close timeout: ~20 seconds (waits for graceful exit, then forces kill)
- Startup verification: Up to 8 seconds wait
- Total operation: Typically 30-40 seconds on macOS

## Known Limitations

1. Requires Antigravity.app to be in `/Applications/` folder
2. User data must be in standard `~/Library/Application Support/` location
3. Works better if Antigravity was already running
4. No support for portable/custom installations (future improvement)

## Next Steps (Optional Improvements)

1. Load custom Antigravity path from config file
2. Support for portable installations
3. Version detection for newer token formats
4. Optional system keyring integration

