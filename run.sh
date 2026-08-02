#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cleanup() {
    echo "Shutting down..."
    kill $BACKEND_PID 2>/dev/null
    kill $FRONTEND_PID 2>/dev/null
    exit 0
}

trap cleanup SIGINT SIGTERM

echo "Starting backend..."
cd "$SCRIPT_DIR/backend"
cargo run --release &
BACKEND_PID=$!

sleep 2

echo "Starting frontend..."
cd "$SCRIPT_DIR/frontend"
bun run dev &
FRONTEND_PID=$!

echo ""
echo "payme is running:"
echo "  Frontend: http://localhost:3000"
echo "  Backend:  http://localhost:3001"
echo ""
echo "Press Ctrl+C to stop"

wait
