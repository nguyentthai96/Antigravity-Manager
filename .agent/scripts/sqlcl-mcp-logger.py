#!/usr/bin/env python3
"""
Transparent MCP proxy logger for SQLcl MCP Server.

Sits between Antigravity and SQLcl, forwarding all JSON-RPC messages
bidirectionally while logging every request/response to a timestamped file.

Log location: .agent/scripts/log/sqlcl_mcp_YYYYMMDD_HHMMSS.log
"""

import sys
import os
import subprocess
import threading
import datetime
import signal
import json

# --- Config ---
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
LOG_DIR = os.path.join(SCRIPT_DIR, "log")
SDKMAN_INIT = os.path.expanduser("~/.sdkman/bin/sdkman-init.sh")
SQLCL_BIN = "sqlcl/bin/sql"
MAX_LOG_FILES = 20  # Keep only the latest N log files

os.makedirs(LOG_DIR, exist_ok=True)

timestamp = datetime.datetime.now().strftime("%Y%m%d_%H")
log_path = os.path.join(LOG_DIR, f"sqlcl_mcp_{timestamp}.log")

# Thread-safe lock for writing to log file
_log_lock = threading.Lock()
# Persistent log file handle
_log_file = None


def open_log():
    """Open the log file for the session."""
    global _log_file
    _log_file = open(log_path, "a", encoding="utf-8", buffering=1)  # line-buffered


def close_log():
    """Close the log file."""
    global _log_file
    if _log_file and not _log_file.closed:
        _log_file.flush()
        _log_file.close()


def log_message(direction: str, data: str):
    """Append a timestamped log entry to the log file (thread-safe)."""
    ts = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
    with _log_lock:
        try:
            if _log_file and not _log_file.closed:
                _log_file.write(f"[{ts}] {direction} {data}\n")
                _log_file.flush()
                os.fsync(_log_file.fileno())  # Force OS flush to disk
        except IOError:
            pass


def format_json(raw: str) -> str:
    """Try to pretty-format JSON for readability in logs."""
    try:
        parsed = json.loads(raw)
        if isinstance(parsed, dict):
            method = parsed.get("method", "")
            msg_id = parsed.get("id", "")
            if method:
                # Extract SQL from tools/call for better readability
                params = parsed.get("params", {})
                args = params.get("arguments", {})
                tool_name = params.get("name", "")
                sql = args.get("sql", args.get("sqlcl", ""))
                if sql:
                    return f"[id={msg_id}] method={method} tool={tool_name} SQL=[{sql}]"
                elif tool_name:
                    return f"[id={msg_id}] method={method} tool={tool_name} args={json.dumps(args, ensure_ascii=False)[:500]}"
                return f"[id={msg_id}] method={method} | {raw[:1000]}"
            elif "result" in parsed:
                result_preview = str(parsed["result"])[:2000]
                return f"[id={msg_id}] result={result_preview}"
            elif "error" in parsed:
                return f"[id={msg_id}] ERROR={parsed['error']}"
        return raw
    except (json.JSONDecodeError, TypeError):
        return raw


def pipe_stream(src, dst, direction: str):
    """
    Read lines from src (binary stream), log them, write to dst (binary stream).
    direction: '>>>' for request (client->server), '<<<' for response (server->client)
    """
    try:
        while True:
            line = src.readline()
            if not line:
                break
            # Write to destination FIRST to not delay MCP protocol
            try:
                dst.write(line)
                dst.flush()
            except (BrokenPipeError, IOError):
                break

            # Then log asynchronously
            text = line.decode("utf-8", errors="replace").rstrip("\n\r")
            if text.strip():
                formatted = format_json(text)
                log_message(direction, formatted)
    except (BrokenPipeError, IOError, ValueError):
        pass


def log_stderr(stream):
    """Capture and log stderr from the SQLcl process."""
    try:
        for line in iter(stream.readline, b""):
            text = line.decode("utf-8", errors="replace").rstrip("\n\r")
            if text.strip():
                log_message("ERR", text)
    except (IOError, ValueError):
        pass


def cleanup_old_logs():
    """Remove old log files, keeping only the latest MAX_LOG_FILES."""
    try:
        log_files = sorted(
            [f for f in os.listdir(LOG_DIR) if f.startswith("sqlcl_mcp_") and f.endswith(".log")],
            reverse=True,
        )
        for old_file in log_files[MAX_LOG_FILES:]:
            os.remove(os.path.join(LOG_DIR, old_file))
    except OSError:
        pass


def resolve_sdkman_env() -> dict:
    """Source SDKMAN init script and capture the resulting environment."""
    env = os.environ.copy()
    if not os.path.isfile(SDKMAN_INIT):
        return env
    try:
        result = subprocess.run(
            ["bash", "-c", f"source '{SDKMAN_INIT}' && env -0"],
            capture_output=True,
            timeout=10,
        )
        if result.returncode == 0:
            for entry in result.stdout.decode("utf-8", errors="replace").split("\0"):
                if "=" in entry:
                    k, _, v = entry.partition("=")
                    env[k] = v
    except (subprocess.TimeoutExpired, OSError):
        log_message("WARN", "Failed to source SDKMAN, using current env")
    return env


def main():
    cleanup_old_logs()
    open_log()

    log_message("INFO", "=" * 60)
    log_message("INFO", "SQLcl MCP Server session started")
    log_message("INFO", f"Log file: {log_path}")
    log_message("INFO", f"SQLcl binary: {SQLCL_BIN}")
    log_message("INFO", "=" * 60)

    # Resolve Java from SDKMAN
    env = resolve_sdkman_env()
    java_home = env.get("JAVA_HOME", "NOT SET")
    log_message("INFO", f"JAVA_HOME={java_home}")

    # Launch SQLcl MCP server as subprocess
    try:
        proc = subprocess.Popen(
            [SQLCL_BIN, "-mcp"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
        )
    except FileNotFoundError:
        log_message("FATAL", f"SQLcl binary not found: {SQLCL_BIN}")
        close_log()
        sys.exit(1)

    # Wire up bidirectional proxy threads
    t_in = threading.Thread(
        target=pipe_stream,
        args=(sys.stdin.buffer, proc.stdin, ">>>"),
        daemon=True,
        name="stdin-proxy",
    )
    t_out = threading.Thread(
        target=pipe_stream,
        args=(proc.stdout, sys.stdout.buffer, "<<<"),
        daemon=True,
        name="stdout-proxy",
    )
    t_err = threading.Thread(
        target=log_stderr,
        args=(proc.stderr,),
        daemon=True,
        name="stderr-logger",
    )

    t_in.start()
    t_out.start()
    t_err.start()

    # Forward SIGTERM/SIGINT to child process
    def handle_signal(signum, frame):
        log_message("INFO", f"Received signal {signum}, forwarding to SQLcl")
        proc.send_signal(signum)

    signal.signal(signal.SIGTERM, handle_signal)
    signal.signal(signal.SIGINT, handle_signal)

    # Wait for SQLcl to exit
    exit_code = proc.wait()

    # Wait a bit for pipe threads to flush remaining data
    t_out.join(timeout=2)
    t_err.join(timeout=2)

    log_message("INFO", f"SQLcl MCP Server exited with code {exit_code}")
    log_message("INFO", "=" * 60)
    close_log()
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
