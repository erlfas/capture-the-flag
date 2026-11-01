---
name: game-compile-and-wait
description: >-
  Syncs workspace changes to WSL, compiles the Cyber-Athletic Coliseum Bevy/Wasm project using Trunk,
  and safely waits for compilation/dev-server completion without self-matching process bugs.
  Use when asked to compile, build, package, or verify build status for the game.
---

# Game Compile and Wait Skill

This skill synchronizes the game workspace from Windows to native WSL 2 (ext4), compiles the Rust/Bevy codebase to WebAssembly using Trunk, and reliably waits for completion without encountering `pgrep` self-matching bugs.

## How to Run

Execute the helper script through PowerShell via `wsl.exe`:

### 1. Standard Development Build (Synchronous)
Syncs files and runs `trunk build` to completion:
```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-compile-and-wait/scripts/build_and_wait.sh build
```

### 2. Production Release Build
Syncs files and runs `trunk build --release` (with thin LTO and size optimization):
```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-compile-and-wait/scripts/build_and_wait.sh release
```

### 3. Sync & Wait for Dev Server (`trunk serve`)
Syncs files and monitors the running dev server until compilation finishes and the HTTP server (`http://localhost:8080`) is ready:
```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-compile-and-wait/scripts/build_and_wait.sh wait-dev
```

## Safety Mechanisms Built In
* **No `pgrep -f` traps:** Uses exact binary matching (`pgrep -x 'rustc|wasm-bindgen'`) so the monitoring loop will never match its own process arguments.
* **HTTP Healthcheck:** Verifies that port 8080 responds with HTTP 200 before concluding.
* **Automated Sync:** Ensures edits in the Windows workspace are reflected in the WSL build directory before compilation starts.
* **Artifact Reporting:** Automatically prints the size and timestamp of generated `.wasm` and `.js` bundles in `dist/`.
