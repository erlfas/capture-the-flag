# Engineering Memo: Preventing Self-Matching Loops in Build Monitoring

**Date:** 2026-11-01  
**Status:** Active Reference / Best Practice  
**Applies To:** Build automation, process watchers, CI/local scripts (WSL / Linux / PowerShell)

---

## 1. Incident Summary & Root Cause

### What Happened
During a game compilation check, an automated polling script appeared to run indefinitely, leading to the false impression that the Rust / WebAssembly compilation had hung. In reality, the build had completed 10+ minutes earlier, but the monitoring script remained trapped in an infinite loop.

### The Root Cause: Self-Matching with `pgrep -f`
The monitoring loop was written as:
```bash
while pgrep -f wasm-bindgen > /dev/null; do sleep 1; done; echo 'DONE'
```

* The `-f` flag tells `pgrep` to match against the **full command line** (including all arguments), not just the executable name.
* The command line of the `bash` subshell running the loop was:
  ```text
  bash -c while pgrep -f wasm-bindgen > /dev/null; do sleep 1; done; echo 'DONE'
  ```
* Because that command line contains the literal substring `"wasm-bindgen"`, **`pgrep -f wasm-bindgen` matched its own parent process** on every iteration.
* The loop never exited because `pgrep` consistently found at least one matching process (itself) and exited with code `0`.

---

## 2. Solutions & Fixes for Future Automation

Choose the appropriate technique based on the monitoring context:

### Fix A: The Regex Bracket Trick (Easiest Inline Drop-In)
If you must use `pgrep -f`, wrap the first letter of the target name in square brackets `[...]`:

```bash
# 'w' in brackets matches the character 'w', but the command line string literally contains '[w]'
while pgrep -f '[w]asm-bindgen' > /dev/null; do sleep 1; done; echo 'DONE'
```
* **Why it works:** The regex `[w]asm-bindgen` evaluates to `"wasm-bindgen"` when matching process names, but the watcher's own argument is `[w]asm-bindgen`, so it will never match itself.

---

### Fix B: Exact Executable Name Matching (`pgrep -x` or `pidof`)
Avoid `-f` unless you strictly need to inspect command-line arguments. Instead, match only the binary's process name:

```bash
# Matches only processes where the executable itself is named wasm-bindgen
while pgrep -x wasm-bindgen > /dev/null; do sleep 1; done; echo 'DONE'
```
Or using `pidof`:
```bash
while pidof wasm-bindgen > /dev/null; do sleep 1; done; echo 'DONE'
```

---

### Fix C: PID Tracking (Deterministic Process Watching)
Do not search the process table by string pattern if you launched the process yourself. Capture the process ID directly:

```bash
# 1. Capture the PID upon launching
cargo build --target wasm32-unknown-unknown &
BUILD_PID=$!

# 2. Wait directly for completion without polling
wait $BUILD_PID
echo "Build finished with exit code $?"
```

For processes you did not start but whose PID you know:
```bash
# Check if a specific PID is alive without self-matching risk
while kill -0 "$TARGET_PID" 2>/dev/null; do
    sleep 1
done
```

---

### Fix D: Healthcheck / Artifact Readiness Polling (Best for Trunk Dev Server)
When `trunk serve` is running as a long-lived daemon, checking child processes like `wasm-bindgen` or `rustc` is brittle because:
1. They exist only intermittently during sub-stages of compilation.
2. A premature check before `rustc` spawns might falsely report the build as finished.

Instead, poll Trunk's HTTP endpoint or watch for artifact updates:

#### 1. HTTP Endpoint Check
```bash
# Wait until the web server is responding
until curl -s -f http://localhost:8080 > /dev/null; do
    sleep 1
done
echo "Trunk server is ready and serving the game!"
```

#### 2. Synchronous Release Build
When testing compilation non-interactively, run the synchronous build command instead of watching `trunk serve`:
```bash
# Exits with a clear code (0 on success, non-zero on failure)
trunk build --release
```

---

## 3. Quick Reference Card

| Goal | Don't Do | Do Instead |
| :--- | :--- | :--- |
| **Wait for an executable** | `pgrep -f wasm-bindgen` | `pgrep -x wasm-bindgen` or `pidof wasm-bindgen` |
| **Regex full-cmdline match** | `pgrep -f "pattern"` | `pgrep -f "[p]attern"` |
| **Wait on a launched job** | `pgrep -f cargo` | `wait $PID` |
| **Check dev server readiness** | Polling sub-compilers | `curl -s -f http://localhost:8080` |
| **Test build in CI/scripts** | Backgrounding `trunk serve` | `trunk build` (runs to completion) |
