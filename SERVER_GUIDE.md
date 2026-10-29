# How to Start and Stop the Game Server (`SERVER_GUIDE.md`)

**Cyber-Athletic Coliseum** is built with Rust + Bevy (`0.15.3`, WebGL2) and bundled for the browser using **Trunk** inside WSL (Ubuntu).

---

## 1. Quick Start (From Windows PowerShell / Terminal)

Run the following command to launch the Trunk development server in WSL:

```powershell
wsl.exe -e bash -c "source ~/.cargo/env && cd ~/repos/cyber_coliseum_koth && CARGO_TARGET_DIR=~/cargo_target trunk serve --port 8080 --address 0.0.0.0"
```

Once you see `INFO ✅ success`, open your browser to:
- **http://localhost:8080**

---

## 2. Starting Inside a WSL (Ubuntu) Terminal

If you already have a WSL terminal open, run:

```bash
source ~/.cargo/env
cd ~/repos/cyber_coliseum_koth
CARGO_TARGET_DIR=~/cargo_target trunk serve --port 8080 --address 0.0.0.0
```

---

## 3. Syncing Changes from the Windows Workspace Before Starting

If you edited files in the Windows workspace (`C:\Users\ErlendHeggelund\Documents\antigravity\fervent-curie`), sync them to the native WSL repository before starting (running `cargo` and `trunk` on the native Linux filesystem is significantly faster and avoids Windows file-locking):

```powershell
wsl.exe -e bash -c "rsync -av --exclude='target' --exclude='dist' --exclude='.git' /mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie/ ~/repos/cyber_coliseum_koth/"
```

---

## 4. Stopping the Game Server

- **Interactive Terminal**: Press `Ctrl + C` in the terminal running `trunk serve`.
- **From Another Terminal (PowerShell)**:
  ```powershell
  wsl.exe -e bash -c "pkill -f 'trunk serve'"
  ```
