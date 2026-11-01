---
name: game-start-server
description: >-
  Syncs workspace changes to WSL, starts or restarts the Trunk development game server
  (http://localhost:8080) for Cyber-Athletic Coliseum, and verifies health status.
  Use when asked to start, launch, serve, run, or restart the game server.
---

# Game Start Server Skill

This skill synchronizes the game workspace from Windows to native WSL 2 (ext4), starts or restarts the Trunk development server (`trunk serve`), and monitors it until the HTTP server is responsive at `http://localhost:8080`.

## Quick Execution

Execute the helper script from PowerShell via `wsl.exe`:

### 1. Start Server (Default)
Syncs latest edits and starts `trunk serve` if not already running (or confirms existing instance):
```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh start
```

### 2. Restart Server
Stops any existing Trunk process, syncs fresh workspace files, restarts `trunk serve`, and waits for HTTP 200:
```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh restart
```

### 3. Check Server Status & Health
Checks whether the process is alive, verifies the `http://localhost:8080` endpoint, and displays recent log output:
```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh status
```

### 4. View Logs
Tails the Trunk development server log:
```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh logs
```

### 5. Stop Server
Gracefully terminates the background Trunk server:
```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh stop
```

---

## Alternative Direct PowerShell Commands

If running without the helper script:

* **Start in Background:**
  ```powershell
  wsl.exe -e bash -c "source ~/.cargo/env && cd ~/repos/cyber_coliseum_koth && CARGO_TARGET_DIR=~/cargo_target nohup trunk serve --address 0.0.0.0 --port 8080 > trunk_serve.log 2>&1 &"
  ```
* **Verify Health:**
  ```powershell
  curl.exe -I http://localhost:8080
  ```
* **Stop:**
  ```powershell
  wsl.exe -e bash -c "pkill -x trunk"
  ```

---

## Key Safety & Performance Features

* **WSL Native Ext4 Compilation:** Runs on `~/repos/cyber_coliseum_koth` to avoid Windows filesystem I/O overhead and file locking.
* **Automated Rsync:** Keeps code, shaders, and assets synced while excluding `target/`, `dist/`, `.git/`, and `.cargo/`.
* **Zero Self-Matching Bugs:** Adheres to [BUILD_MONITORING_MEMO.md](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/BUILD_MONITORING_MEMO.md) by strictly using `pgrep -x` and HTTP health checks rather than generic regex loops.
