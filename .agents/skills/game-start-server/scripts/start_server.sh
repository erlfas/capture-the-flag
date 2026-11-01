#!/usr/bin/env bash
# start_server.sh - Manage, start, and monitor the Trunk dev server for Cyber-Athletic Coliseum
set -eo pipefail

ACTION="${1:-start}" # "start", "restart", "stop", "status", or "logs"
PORT="${2:-8080}"
ADDRESS="0.0.0.0"

WIN_DIR="/mnt/c/Users/ErlendHeggelund/Documents/antigravity/fervent-curie"
REPO_DIR="$HOME/repos/cyber_coliseum_koth"
TARGET_DIR="$HOME/cargo_target"
LOG_FILE="$REPO_DIR/trunk_serve.log"

sync_workspace() {
    echo "--> Syncing workspace from Windows to WSL..."
    if [ -d "$WIN_DIR" ]; then
        rsync -av --delete \
            --exclude='target' \
            --exclude='dist' \
            --exclude='.git' \
            --exclude='.cargo' \
            "$WIN_DIR/" "$REPO_DIR/"
        echo "    Workspace sync complete."
    else
        echo "    Warning: Windows directory $WIN_DIR not found; skipping rsync."
    fi
}

ensure_cargo_env() {
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
}

wait_for_server() {
    local max_retries=60
    local count=0
    echo "--> Waiting for server readiness at http://localhost:${PORT}..."

    # If compilation is actively underway, show progress
    while pgrep -x 'rustc|wasm-bindgen' > /dev/null 2>&1; do
        sleep 1
        echo "    Compiling WebAssembly / Bevy assets..."
    done

    while [ $count -lt $max_retries ]; do
        if curl -s -f "http://localhost:${PORT}" > /dev/null 2>&1; then
            echo "=========================================================="
            echo " [Game Server Status] READY"
            echo " URL: http://localhost:${PORT}"
            echo " Logs: $LOG_FILE"
            echo "=========================================================="
            return 0
        fi
        sleep 1
        count=$((count + 1))
    done

    echo "Warning: Timed out after ${max_retries}s waiting for http://localhost:${PORT}."
    echo "Check logs with: tail -n 30 $LOG_FILE"
    return 1
}

case "$ACTION" in
    start)
        echo "=========================================================="
        echo " [Game Server] Starting Trunk dev server..."
        echo "=========================================================="
        ensure_cargo_env
        cd "$REPO_DIR"

        if pgrep -x trunk > /dev/null 2>&1; then
            TRUNK_PID=$(pgrep -x trunk | head -n 1)
            echo "--> Trunk dev server is already running (PID: $TRUNK_PID)."
            if curl -s -f "http://localhost:${PORT}" > /dev/null 2>&1; then
                echo "--> Server is responsive at http://localhost:${PORT}"
            else
                echo "--> Server process is running, but port $PORT is not answering yet."
                wait_for_server
            fi
        else
            sync_workspace
            echo "--> Launching 'trunk serve' on port $PORT..."
            CARGO_TARGET_DIR="$TARGET_DIR" nohup trunk serve --address "$ADDRESS" --port "$PORT" > "$LOG_FILE" 2>&1 &
            sleep 2
            wait_for_server
        fi
        ;;

    restart)
        echo "=========================================================="
        echo " [Game Server] Restarting Trunk dev server..."
        echo "=========================================================="
        ensure_cargo_env
        cd "$REPO_DIR"

        if pgrep -x trunk > /dev/null 2>&1; then
            echo "--> Stopping existing Trunk server..."
            pkill -x trunk || true
            sleep 2
            # Force kill if still lingering
            if pgrep -x trunk > /dev/null 2>&1; then
                pkill -9 -x trunk || true
            fi
            echo "    Stopped."
        fi

        sync_workspace
        echo "--> Launching fresh 'trunk serve' on port $PORT..."
        CARGO_TARGET_DIR="$TARGET_DIR" nohup trunk serve --address "$ADDRESS" --port "$PORT" > "$LOG_FILE" 2>&1 &
        sleep 2
        wait_for_server
        ;;

    stop)
        echo "=========================================================="
        echo " [Game Server] Stopping Trunk dev server..."
        echo "=========================================================="
        if pgrep -x trunk > /dev/null 2>&1; then
            pkill -x trunk || true
            sleep 1
            if pgrep -x trunk > /dev/null 2>&1; then
                pkill -9 -x trunk || true
            fi
            echo "--> Game server stopped successfully."
        else
            echo "--> No running Trunk dev server found."
        fi
        ;;

    status)
        echo "=========================================================="
        echo " [Game Server Status Check]"
        echo "=========================================================="
        if pgrep -x trunk > /dev/null 2>&1; then
            TRUNK_PID=$(pgrep -x trunk | tr '\n' ' ')
            echo "Process: RUNNING (PID: $TRUNK_PID)"
            if curl -s -f "http://localhost:${PORT}" > /dev/null 2>&1; then
                echo "Endpoint: RESPONSIVE (http://localhost:${PORT} returned HTTP 200)"
            else
                echo "Endpoint: NOT RESPONDING on port ${PORT} (compilation may be in progress)"
            fi
        else
            echo "Process: STOPPED"
            echo "Endpoint: Inactive"
        fi

        if [ -f "$LOG_FILE" ]; then
            echo "--- Recent Logs (last 10 lines) ---"
            tail -n 10 "$LOG_FILE"
        fi
        echo "=========================================================="
        ;;

    daemon)
        echo "=========================================================="
        echo " [Game Server] Starting Persistent Foreground Daemon..."
        echo "=========================================================="
        ensure_cargo_env
        cd "$REPO_DIR"
        if pgrep -x trunk > /dev/null 2>&1; then
            pkill -x trunk || true
            sleep 1
        fi
        sync_workspace
        echo "--> Running 'trunk serve' in foreground (keep WSL session alive with IsDaemon=true)..."
        exec env CARGO_TARGET_DIR="$TARGET_DIR" trunk serve --address "$ADDRESS" --port "$PORT"
        ;;

    logs)
        if [ -f "$LOG_FILE" ]; then
            tail -n 50 "$LOG_FILE"
        else
            echo "No log file found at $LOG_FILE."
        fi
        ;;

    *)
        echo "Usage: $0 {start|restart|stop|status|logs} [port]"
        exit 1
        ;;
esac
