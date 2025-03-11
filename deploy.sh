#!/bin/bash
echo -e "\033[36m"  # Set color to cyan
cat << "EOF"
 /$$$$$$$  /$$$$$$$$ /$$$$$$ /$$      /$$  /$$$$$$  /$$   /$$ /$$   /$$
| $$__  $$| $$_____/|_  $$_/| $$$    /$$$ /$$__  $$| $$$ | $$| $$$ | $$
| $$  \ $$| $$        | $$  | $$$$  /$$$$| $$  \ $$| $$$$| $$| $$$$| $$
| $$$$$$$/| $$$$$     | $$  | $$ $$/$$ $$| $$$$$$$$| $$ $$ $$| $$ $$ $$
| $$__  $$| $$__/     | $$  | $$  $$$| $$| $$__  $$| $$  $$$$| $$  $$$$
| $$  \ $$| $$        | $$  | $$\  $ | $$| $$  | $$| $$\  $$$| $$\  $$$
| $$  | $$| $$$$$$$$ /$$$$$$| $$ \/  | $$| $$  | $$| $$ \  $$| $$ \  $$
|__/  |__/|________/|______/|__/     |__/|__/  |__/|__/  \__/|__/  \__/
EOF
echo -e "\033[0m" 
# Store PIDs
pids=()

# Function to cleanup all processes
cleanup() {
    echo -e "\n😴 Shutting down services..."

    # kill all processes
    pkill -f da-exex
    sleep 0.5
    pkill -f da-server
    sleep 0.5
    pkill -f smt-server
    sleep 0.5
    pkill -f solver
    sleep 0.5

    pkill -9 -f da-exex
    pkill -9 -f da-server
    pkill -9 -f smt-server
    pkill -9 -f solver
    sleep 0.5
    
    rm -rf logs
    rm -rf chains
    
    echo "🧹 Cleanup complete"
}

bye() {
    exit 0
}

# Handle various exit signals
trap cleanup INT
trap bye EXIT

# Create logs directory
mkdir -p logs

echo "⬆️ Starting services..."

# Start DA server
echo "🧱 Starting DA server..."
cargo run --bin cli run da > logs/da.log 2>&1 &
pids+=($!)
echo "ℹ️ DA server started with PID: ${pids[0]}"
sleep 2

# Start SMT server
echo "🌳 Starting SMT server..."
cargo run --bin cli run smt > logs/smt.log 2>&1 &
pids+=($!)
echo "ℹ️ SMT server started with PID: ${pids[1]}"
sleep 2

# Generate genesis files
echo "📜 Generating genesis files..."
cargo run --bin cli genesis init

# Start Nexus chain
echo "🚀 Starting Nexus chain..."
cargo run --bin cli run nexus > logs/nexus.log 2>&1 &
pids+=($!)
echo "ℹ️ Nexus chain started with PID: ${pids[2]}"
sleep 1

# Start rollup1
echo "📤 Starting origin rollup..."
cargo run --bin cli run rollup --name rollup1 > logs/rollup1.log 2>&1 &
pids+=($!)
echo "ℹ️ Rollup started with PID: ${pids[3]}"
sleep 1

# Start rollup2
echo "📥 Starting destination rollup..."
cargo run --bin cli run rollup --name rollup2 --port 8547 --p2p-port 30305 --authrpc-port 8553 > logs/rollup2.log 2>&1 &
pids+=($!)
echo "ℹ️ Rollup started with PID: ${pids[4]}"
sleep 1

# Wait for chains to be ready
echo "⏳ Waiting for chains to be ready..."
sleep 7

# Run full init
echo "🏃 Running full Reimann initialization..."
cargo run --bin cli test full init

sleep 2

echo "🧮 Starting solver..."
cargo run --bin cli run solver > logs/solver.log 2>&1 &
pids+=($!)
echo "ℹ️ Solver started with PID: ${pids[5]}"

sleep 2

echo "ℹ️ Press Ctrl+C to gracefully terminate all services."

# Keep script running until user interrupts
wait
