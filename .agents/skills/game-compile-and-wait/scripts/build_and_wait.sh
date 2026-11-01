#!/usr/bin/env bash
# build_and_wait.sh - Reliable sync, compilation, and wait script for Cyber-Athletic Coliseum
set -eo pipefail

MODE="${1:-build}" # "build", "release", or "wait-dev"
WIN_DIR="/mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie"
REPO_DIR="$HOME/repos/cyber_coliseum_koth"
TARGET_DIR="$HOME/cargo_target"

echo "=========================================================="
echo " [Game Build Pipeline] Starting at $(date)"
echo " Mode: $MODE"
echo "=========================================================="

# 1. Sync Windows edits to WSL native filesystem
echo "--> 1. Syncing workspace from Windows to WSL..."
if [ -d "$WIN_DIR" ]; then
    rsync -av --delete \
        --exclude='target' \
        --exclude='dist' \
        --exclude='.git' \
        --exclude='.cargo' \
        "$WIN_DIR/" "$REPO_DIR/"
    echo "    Sync complete."
else
    echo "    Warning: Windows directory $WIN_DIR not found; skipping rsync."
fi

# 2. Source cargo environment
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

cd "$REPO_DIR"

# 3. Execute according to mode
if [ "$MODE" = "wait-dev" ]; then
    echo "--> 2. Monitoring dev server (trunk serve) build completion..."
    # Check if trunk serve is active
    if ! pgrep -x trunk > /dev/null; then
        echo "    Notice: trunk serve not running. Starting trunk serve in background..."
        CARGO_TARGET_DIR="$TARGET_DIR" nohup trunk serve --address 0.0.0.0 --port 8080 > "$REPO_DIR/trunk_serve.log" 2>&1 &
        sleep 2
    fi

    # Wait while compilers are active (using exact process matching to avoid self-match bugs)
    echo "    Waiting for active rustc/wasm-bindgen tasks to finish..."
    while pgrep -x 'rustc|wasm-bindgen' > /dev/null; do
        sleep 1
    done

    # Wait for HTTP endpoint readiness
    echo "    Verifying web server responsiveness at http://localhost:8080..."
    for i in {1..30}; do
        if curl -s -f http://localhost:8080 > /dev/null 2>&1; then
            echo "    Server responded successfully."
            break
        fi
        sleep 1
    done

elif [ "$MODE" = "release" ]; then
    echo "--> 2. Executing production release build (trunk build --release)..."
    CARGO_TARGET_DIR="$TARGET_DIR" trunk build --release

else # default: "build"
    echo "--> 2. Executing standard development build (trunk build)..."
    CARGO_TARGET_DIR="$TARGET_DIR" trunk build
fi

echo "=========================================================="
echo " [Game Build Pipeline] Finished successfully at $(date)"
echo " Generated artifacts in $REPO_DIR/dist/:"
ls -lh "$REPO_DIR/dist/"*.wasm "$REPO_DIR/dist/"*.js "$REPO_DIR/dist/index.html" 2>/dev/null || true
echo "=========================================================="
