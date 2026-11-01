# How to Start and Stop the Game Server (`SERVER_GUIDE.md`)

**Cyber-Athletic Coliseum** is built with Rust + Bevy (`0.15.3`, WebGL2) and bundled for the browser using **Trunk** inside WSL (Ubuntu).

---

## 1. Quick Start (Using the `game-start-server` Skill / Helper Script)

Run the automated starter script via PowerShell:

```powershell
wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh start
```
This automatically syncs Windows workspace edits into WSL, launches `trunk serve` if not running, and validates that `http://localhost:8080` is healthy.

### Other Useful Commands
* **Restart Server:** `wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh restart`
* **Check Status:** `wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh status`
* **Stop Server:** `wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh stop`
* **View Logs:** `wsl.exe -e bash /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/.agents/skills/game-start-server/scripts/start_server.sh logs`

---

## 2. Manual Start (From Windows PowerShell / Terminal)

Run the following command to launch the Trunk development server in WSL manually:

```powershell
wsl.exe -e bash -c "source ~/.cargo/env && cd ~/repos/cyber_coliseum_koth && CARGO_TARGET_DIR=~/cargo_target trunk serve --port 8080 --address 0.0.0.0"
```

Once you see `INFO ✅ success`, open your browser to:
- **http://localhost:8080**

---

## 3. Starting Inside a WSL (Ubuntu) Terminal

If you already have a WSL terminal open, run:

```bash
source ~/.cargo/env
cd ~/repos/cyber_coliseum_koth
CARGO_TARGET_DIR=~/cargo_target trunk serve --port 8080 --address 0.0.0.0
```

---

## 4. Syncing Changes from the Windows Workspace Before Starting

If you edited files in the Windows workspace (`C:\Users\ErlendHeggelund\Documents\antigravity\fervent-curie`), sync them to the native WSL repository before starting (running `cargo` and `trunk` on the native Linux filesystem is significantly faster and avoids Windows file-locking):

```powershell
wsl.exe -e bash -c "rsync -av --exclude='target' --exclude='dist' --exclude='.git' /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/ ~/repos/cyber_coliseum_koth/"
```

---

## 5. Stopping the Game Server

- **Interactive Terminal**: Press `Ctrl + C` in the terminal running `trunk serve`.
- **From Another Terminal (PowerShell)**:
  ```powershell
  wsl.exe -e bash -c "pkill -x trunk"
  ```

---

## 6. Troubleshooting & Process Monitoring

- If checking whether compilation has finished or waiting for server readiness, avoid `while pgrep -f wasm-bindgen; do sleep 1; done` as it matches itself.
- See [`BUILD_MONITORING_MEMO.md`](file:///c:/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/BUILD_MONITORING_MEMO.md) for solutions, robust watcher patterns, and healthcheck polling.
